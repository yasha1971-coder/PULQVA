mod arti_materialize;
mod deno_zip;
mod deno_stage;
mod deno_source;
mod deno_environment;
#[cfg(test)]
mod deno_archive_compat;
mod ffmpeg_archive_policy;
mod ffmpeg_zip;
mod ffmpeg_tar_xz;
mod ffmpeg_stage;
mod ffmpeg_source;
#[cfg(test)]
mod ffmpeg_archive_compat;
mod ytdlp_materialize;

use pulqva_core::{
    CORE_CRATE_READY, SearchCandidate, SearchCandidateError, SearchIntent, SearchIntentError,
};
use pulqva_privacy::{
    ArtiConfigMaterializeError, ArtiProcessError, ArtiRuntimePlan, CompletedDownloadResult,
    CompletedFfmpegRemuxResult, FfmpegCompletionError, FfmpegProcessError, FfmpegRemuxContainer,
    FfmpegRemuxPlan, FfmpegRemuxPlanError,
    PreparedArtiRuntime, ReadyTorTransport, RunningArti, RunningFfmpeg, RunningYtDlp,
    TorReadinessError, TorSocksEndpoint, TorSocksEndpointError, YtDlpCompletionError,
    YtDlpMediaRequestError, YtDlpMediaRequestPlan, YtDlpMediaSourceError, YtDlpMediaSourceUrl,
    YtDlpProcessError, launch_ffmpeg_remux, launch_prepared_arti, launch_ytdlp_request,
    prepare_arti_runtime, verify_tor_readiness,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::Manager;
use std::{
    ffi::OsString,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
    time::Duration,
};

const PRODUCT_NAME: &str = "PULQVA";
const KERNEL_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../kernel/KERNEL_VERSION"));
const T024_MEDIA_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/4d21a042353022326071acb0251ab75cd6bae114/big_buck_bunny.mp4";
const T024_PROOF_LOCATOR: &str = "local:test:candidate:official-live";
const DEFAULT_TOR_SOCKS_PORT: u16 = 19050;
const DEFAULT_TOR_READY_TIMEOUT_SECS: u64 = 30;
const ARTI_SIDECAR_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/arti/VERSION"));
const ARTI_SIDECAR_SHA256SUMS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/arti/SHA256SUMS"));
const YTDLP_SIDECAR_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/yt-dlp/VERSION"));
const YTDLP_SIDECAR_SHA256SUMS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/yt-dlp/SHA256SUMS"));
const FFMPEG_SIDECAR_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/ffmpeg/VERSION"));
const FFMPEG_SIDECAR_SHA256SUMS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../sidecars/ffmpeg/SHA256SUMS"));

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStatus {
    product_name: &'static str,
    app_version: &'static str,
    kernel_version: &'static str,
    core_ready: bool,
    privacy_mode: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct IntentSubmission {
    query: String,
    stage: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct IntentSubmissionError {
    code: &'static str,
    message: String,
}

impl From<SearchIntentError> for IntentSubmissionError {
    fn from(source: SearchIntentError) -> Self {
        Self {
            code: "invalid-search-intent",
            message: source.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DesktopCandidate {
    title: String,
    locator: String,
}

impl From<&SearchCandidate> for DesktopCandidate {
    fn from(candidate: &SearchCandidate) -> Self {
        Self {
            title: candidate.title().to_owned(),
            locator: candidate.locator().to_owned(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CandidateList {
    intent_query: String,
    stage: &'static str,
    candidates: Vec<DesktopCandidate>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CandidateListError {
    code: &'static str,
    message: String,
}

impl From<SearchIntentError> for CandidateListError {
    fn from(source: SearchIntentError) -> Self {
        Self {
            code: "invalid-search-intent",
            message: source.to_string(),
        }
    }
}

impl From<SearchCandidateError> for CandidateListError {
    fn from(source: SearchCandidateError) -> Self {
        Self {
            code: "invalid-local-candidate",
            message: source.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CandidateSelection {
    intent_query: String,
    title: String,
    locator: String,
    stage: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CandidateSelectionError {
    code: &'static str,
    message: String,
}

impl From<SearchIntentError> for CandidateSelectionError {
    fn from(source: SearchIntentError) -> Self {
        Self {
            code: "invalid-search-intent",
            message: source.to_string(),
        }
    }
}

impl From<SearchCandidateError> for CandidateSelectionError {
    fn from(source: SearchCandidateError) -> Self {
        Self {
            code: "invalid-local-candidate",
            message: source.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DownloadAction {
    intent_query: String,
    title: String,
    locator: String,
    action: &'static str,
    stage: &'static str,
    media_source_ready: bool,
    media_source_stage: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DownloadActionError {
    code: &'static str,
    message: String,
}

impl From<SearchIntentError> for DownloadActionError {
    fn from(source: SearchIntentError) -> Self {
        Self {
            code: "invalid-search-intent",
            message: source.to_string(),
        }
    }
}

impl From<SearchCandidateError> for DownloadActionError {
    fn from(source: SearchCandidateError) -> Self {
        Self {
            code: "invalid-local-candidate",
            message: source.to_string(),
        }
    }
}

impl From<YtDlpMediaSourceError> for DownloadActionError {
    fn from(source: YtDlpMediaSourceError) -> Self {
        Self {
            code: "invalid-media-source",
            message: source.to_string(),
        }
    }
}

impl From<TorSocksEndpointError> for DownloadActionError {
    fn from(source: TorSocksEndpointError) -> Self {
        Self {
            code: "invalid-tor-socks-endpoint",
            message: source.to_string(),
        }
    }
}

impl From<ArtiConfigMaterializeError> for DownloadActionError {
    fn from(source: ArtiConfigMaterializeError) -> Self {
        Self {
            code: "arti-runtime-preparation-failed",
            message: source.to_string(),
        }
    }
}

impl From<ArtiProcessError> for DownloadActionError {
    fn from(source: ArtiProcessError) -> Self {
        Self {
            code: "arti-process-failed",
            message: source.to_string(),
        }
    }
}

impl From<YtDlpMediaRequestError> for DownloadActionError {
    fn from(source: YtDlpMediaRequestError) -> Self {
        Self {
            code: "media-request-plan-failed",
            message: source.to_string(),
        }
    }
}

impl From<YtDlpProcessError> for DownloadActionError {
    fn from(source: YtDlpProcessError) -> Self {
        Self {
            code: "ytdlp-process-failed",
            message: source.to_string(),
        }
    }
}

impl From<YtDlpCompletionError> for DownloadActionError {
    fn from(source: YtDlpCompletionError) -> Self {
        Self {
            code: "ytdlp-completion-failed",
            message: source.to_string(),
        }
    }
}

impl From<FfmpegRemuxPlanError> for DownloadActionError {
    fn from(source: FfmpegRemuxPlanError) -> Self {
        Self {
            code: "ffmpeg-remux-plan-failed",
            message: source.to_string(),
        }
    }
}

impl From<FfmpegProcessError> for DownloadActionError {
    fn from(source: FfmpegProcessError) -> Self {
        Self {
            code: "ffmpeg-process-failed",
            message: source.to_string(),
        }
    }
}

impl From<FfmpegCompletionError> for DownloadActionError {
    fn from(source: FfmpegCompletionError) -> Self {
        Self {
            code: "ffmpeg-completion-failed",
            message: source.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CompletedFileView {
    file_name: String,
    byte_size: u64,
    stage: &'static str,
}

fn sanitized_completed_file_view(
    completed: &CompletedFfmpegRemuxResult,
) -> Result<CompletedFileView, DownloadActionError> {
    let file_name = completed
        .output()
        .file_name()
        .ok_or_else(|| DownloadActionError {
            code: "completed-file-name-missing",
            message: "validated completed remux output must include a file name".to_owned(),
        })?
        .to_string_lossy()
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '/' | '\\') {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();

    if file_name.is_empty() {
        return Err(DownloadActionError {
            code: "completed-file-name-missing",
            message: "validated completed remux output must include a file name".to_owned(),
        });
    }

    Ok(CompletedFileView {
        file_name,
        byte_size: completed.byte_size(),
        stage: "completed-file-ready",
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadPreflightInputs {
    arti_executable: PathBuf,
    ytdlp_executable: PathBuf,
    tor_config_file: PathBuf,
    tor_cache_dir: PathBuf,
    tor_state_dir: PathBuf,
    output_root: PathBuf,
    socks_port: u16,
}

impl DownloadPreflightInputs {
    fn new(
        arti_executable: impl Into<PathBuf>,
        ytdlp_executable: impl Into<PathBuf>,
        tor_config_file: impl Into<PathBuf>,
        tor_cache_dir: impl Into<PathBuf>,
        tor_state_dir: impl Into<PathBuf>,
        output_root: impl Into<PathBuf>,
        socks_port: u16,
    ) -> Result<Self, DownloadActionError> {
        let inputs = Self {
            arti_executable: arti_executable.into(),
            ytdlp_executable: ytdlp_executable.into(),
            tor_config_file: tor_config_file.into(),
            tor_cache_dir: tor_cache_dir.into(),
            tor_state_dir: tor_state_dir.into(),
            output_root: output_root.into(),
            socks_port,
        };

        for (name, path) in [
            ("arti executable", inputs.arti_executable.as_path()),
            ("yt-dlp executable", inputs.ytdlp_executable.as_path()),
            ("Tor config file", inputs.tor_config_file.as_path()),
            ("Tor cache directory", inputs.tor_cache_dir.as_path()),
            ("Tor state directory", inputs.tor_state_dir.as_path()),
            ("download output root", inputs.output_root.as_path()),
        ] {
            if path.as_os_str().is_empty() {
                return Err(DownloadActionError {
                    code: "missing-preflight-path",
                    message: format!("{name} must not be empty"),
                });
            }
        }

        TorSocksEndpoint::new(inputs.socks_port).map_err(DownloadActionError::from)?;
        Ok(inputs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppRuntimeLayout {
    root: PathBuf,
}

impl AppRuntimeLayout {
    fn new(root: impl Into<PathBuf>) -> Result<Self, DownloadActionError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(DownloadActionError {
                code: "runtime-layout-root-missing",
                message: "application runtime root must not be empty".to_owned(),
            });
        }
        if root
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(DownloadActionError {
                code: "runtime-layout-root-traversal",
                message: "application runtime root must not contain parent-directory traversal"
                    .to_owned(),
            });
        }
        Ok(Self { root })
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn derive(&self, relative: &str) -> Result<PathBuf, DownloadActionError> {
        let relative = Path::new(relative);
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(DownloadActionError {
                code: "runtime-layout-derived-path-invalid",
                message: "runtime layout derived path must be a non-empty relative child path"
                    .to_owned(),
            });
        }

        let derived = self.root.join(relative);
        if !derived.starts_with(&self.root)
            || derived
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(DownloadActionError {
                code: "runtime-layout-derived-path-escaped",
                message: "runtime layout derived path must remain beneath the application root"
                    .to_owned(),
            });
        }

        Ok(derived)
    }

    fn bin_root(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("bin")
    }

    fn arti_executable(&self) -> Result<PathBuf, DownloadActionError> {
        #[cfg(target_os = "windows")]
        {
            return self.derive("bin/arti.exe");
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.derive("bin/arti")
        }
    }

    fn ytdlp_executable(&self) -> Result<PathBuf, DownloadActionError> {
        #[cfg(target_os = "windows")]
        {
            return self.derive("bin/yt-dlp.exe");
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.derive("bin/yt-dlp")
        }
    }

    fn ffmpeg_executable(&self) -> Result<PathBuf, DownloadActionError> {
        #[cfg(target_os = "windows")]
        {
            return self.derive("bin/ffmpeg.exe");
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.derive("bin/ffmpeg")
        }
    }

    fn tor_config_file(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("arti/config/pulqva.toml")
    }

    fn tor_cache_dir(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("arti/cache")
    }

    fn tor_state_dir(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("arti/state")
    }

    fn output_root(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("downloads")
    }

    fn download_preflight_inputs(&self) -> Result<DownloadPreflightInputs, DownloadActionError> {
        DownloadPreflightInputs::new(
            self.arti_executable()?,
            self.ytdlp_executable()?,
            self.tor_config_file()?,
            self.tor_cache_dir()?,
            self.tor_state_dir()?,
            self.output_root()?,
            DEFAULT_TOR_SOCKS_PORT,
        )
    }

    fn completed_file_pipeline_inputs(
        &self,
    ) -> Result<CompletedFilePipelineInputs, DownloadActionError> {
        CompletedFilePipelineInputs::new(
            self.download_preflight_inputs()?,
            self.ffmpeg_executable()?,
            Duration::from_secs(DEFAULT_TOR_READY_TIMEOUT_SECS),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundledSidecarKind {
    Arti,
    YtDlp,
    Ffmpeg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BundledSidecarIdentity {
    version: String,
    pinned_source_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BundledSidecarMaterializationItem {
    kind: BundledSidecarKind,
    source_resource: PathBuf,
    destination: PathBuf,
    identity: BundledSidecarIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BundledSidecarMaterializationPlan {
    platform: &'static str,
    bin_root: PathBuf,
    items: Vec<BundledSidecarMaterializationItem>,
}

#[derive(Debug, Clone, Copy)]
struct SidecarPlatformSpec {
    platform: &'static str,
    arti_resource: &'static str,
    arti_digest_asset: &'static str,
    ytdlp_resource: &'static str,
    ytdlp_digest_asset: &'static str,
    ffmpeg_resource: &'static str,
    ffmpeg_digest_asset: &'static str,
}

fn current_sidecar_platform_spec() -> Result<SidecarPlatformSpec, DownloadActionError> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Ok(SidecarPlatformSpec {
            platform: "linux-x86_64",
            arti_resource: "sidecars/arti/linux-x86_64/arti",
            arti_digest_asset: "linux-x86_64/arti",
            ytdlp_resource: "sidecars/yt-dlp/linux-x86_64/yt-dlp",
            ytdlp_digest_asset: "yt-dlp_linux",
            ffmpeg_resource: "sidecars/ffmpeg/linux-x86_64/ffmpeg-n9.0.2-3-ga5923073bf-linux64-gpl-9.0.tar.xz",
            ffmpeg_digest_asset: "ffmpeg-n9.0.2-3-ga5923073bf-linux64-gpl-9.0.tar.xz",
        });
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return Ok(SidecarPlatformSpec {
            platform: "windows-x86_64",
            arti_resource: "sidecars/arti/windows-x86_64/arti.exe",
            arti_digest_asset: "windows-x86_64/arti.exe",
            ytdlp_resource: "sidecars/yt-dlp/windows-x86_64/yt-dlp.exe",
            ytdlp_digest_asset: "yt-dlp.exe",
            ffmpeg_resource: "sidecars/ffmpeg/windows-x86_64/ffmpeg-n9.0.2-3-ga5923073bf-win64-gpl-9.0.zip",
            ffmpeg_digest_asset: "ffmpeg-n9.0.2-3-ga5923073bf-win64-gpl-9.0.zip",
        });
    }

    #[allow(unreachable_code)]
    Err(DownloadActionError {
        code: "unsupported-sidecar-platform",
        message: "bundled sidecar plan supports Windows x86_64 and Linux x86_64".to_owned(),
    })
}

fn pinned_sha256_for_asset(
    manifest: &str,
    asset: &str,
) -> Result<String, DownloadActionError> {
    for line in manifest.lines() {
        let mut fields = line.split_whitespace();
        let Some(digest) = fields.next() else {
            continue;
        };
        let Some(name) = fields.next() else {
            continue;
        };

        if name == asset
            && fields.next().is_none()
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(digest.to_ascii_lowercase());
        }
    }

    Err(DownloadActionError {
        code: "sidecar-source-identity-missing",
        message: format!("no pinned SHA-256 identity found for sidecar source asset {asset}"),
    })
}


fn pinned_arti_sha256_for_platform(
    manifest: &str,
    platform: &str,
    asset: &str,
) -> Result<String, DownloadActionError> {
    let expected_prefix = format!("{platform}/");
    if !asset.starts_with(&expected_prefix) {
        return Err(DownloadActionError {
            code: "arti-source-identity-platform-mismatch",
            message: format!(
                "Arti identity asset {asset} does not belong to platform {platform}"
            ),
        });
    }

    let mut found = None;
    for (line_index, line) in manifest.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let fields = trimmed.split_whitespace().collect::<Vec<_>>();
        let names_requested_asset = fields.get(1).copied() == Some(asset)
            || fields.last().copied() == Some(asset);
        if !names_requested_asset {
            continue;
        }

        if fields.len() != 2 {
            return Err(DownloadActionError {
                code: "arti-source-identity-malformed",
                message: format!(
                    "Arti SHA-256 identity for {asset} is malformed on line {}",
                    line_index + 1
                ),
            });
        }

        let digest = fields[0];
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(DownloadActionError {
                code: "arti-source-identity-malformed",
                message: format!(
                    "Arti SHA-256 identity for {asset} must be exactly 64 hexadecimal characters"
                ),
            });
        }

        if found.is_some() {
            return Err(DownloadActionError {
                code: "arti-source-identity-duplicate",
                message: format!("Arti SHA-256 identity for {asset} appears more than once"),
            });
        }
        found = Some(digest.to_ascii_lowercase());
    }

    found.ok_or_else(|| DownloadActionError {
        code: "arti-source-identity-missing",
        message: format!("no pinned Arti SHA-256 identity found for platform asset {asset}"),
    })
}

fn validate_bundled_sidecar_item(
    bin_root: &Path,
    item: &BundledSidecarMaterializationItem,
) -> Result<(), DownloadActionError> {
    if item.source_resource.as_os_str().is_empty()
        || item
            .source_resource
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(DownloadActionError {
            code: "sidecar-resource-path-invalid",
            message: "bundled sidecar resource must be a non-empty relative resource path".to_owned(),
        });
    }

    if !item.destination.starts_with(bin_root) || item.destination.parent() != Some(bin_root) {
        return Err(DownloadActionError {
            code: "sidecar-destination-escaped",
            message: "sidecar runtime destination must be a direct child of runtime/bin".to_owned(),
        });
    }

    if item.source_resource == item.destination {
        return Err(DownloadActionError {
            code: "sidecar-source-destination-collision",
            message: "bundled sidecar source and runtime destination must differ".to_owned(),
        });
    }

    if item.identity.version.is_empty() {
        return Err(DownloadActionError {
            code: "sidecar-source-identity-missing",
            message: "bundled sidecar identity must include a pinned version".to_owned(),
        });
    }

    Ok(())
}

fn build_bundled_sidecar_materialization_plan(
    layout: &AppRuntimeLayout,
) -> Result<BundledSidecarMaterializationPlan, DownloadActionError> {
    let platform = current_sidecar_platform_spec()?;
    let bin_root = layout.bin_root()?;

    let items = vec![
        BundledSidecarMaterializationItem {
            kind: BundledSidecarKind::Arti,
            source_resource: PathBuf::from(platform.arti_resource),
            destination: layout.arti_executable()?,
            identity: BundledSidecarIdentity {
                version: ARTI_SIDECAR_VERSION.trim().to_owned(),
                pinned_source_sha256: Some(pinned_arti_sha256_for_platform(
                    ARTI_SIDECAR_SHA256SUMS,
                    platform.platform,
                    platform.arti_digest_asset,
                )?),
            },
        },
        BundledSidecarMaterializationItem {
            kind: BundledSidecarKind::YtDlp,
            source_resource: PathBuf::from(platform.ytdlp_resource),
            destination: layout.ytdlp_executable()?,
            identity: BundledSidecarIdentity {
                version: YTDLP_SIDECAR_VERSION.trim().to_owned(),
                pinned_source_sha256: Some(pinned_sha256_for_asset(
                    YTDLP_SIDECAR_SHA256SUMS,
                    platform.ytdlp_digest_asset,
                )?),
            },
        },
        BundledSidecarMaterializationItem {
            kind: BundledSidecarKind::Ffmpeg,
            source_resource: PathBuf::from(platform.ffmpeg_resource),
            destination: layout.ffmpeg_executable()?,
            identity: BundledSidecarIdentity {
                version: FFMPEG_SIDECAR_VERSION.trim().to_owned(),
                pinned_source_sha256: Some(pinned_sha256_for_asset(
                    FFMPEG_SIDECAR_SHA256SUMS,
                    platform.ffmpeg_digest_asset,
                )?),
            },
        },
    ];

    for item in &items {
        validate_bundled_sidecar_item(&bin_root, item)?;
    }

    Ok(BundledSidecarMaterializationPlan {
        platform: platform.platform,
        bin_root,
        items,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedBundledSidecarMaterializationItem {
    kind: BundledSidecarKind,
    source: PathBuf,
    destination: PathBuf,
    identity: BundledSidecarIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedBundledSidecarMaterializationPlan {
    platform: &'static str,
    resource_root: PathBuf,
    bin_root: PathBuf,
    items: Vec<ResolvedBundledSidecarMaterializationItem>,
}

fn resolve_bundled_sidecar_sources(
    resource_root: impl Into<PathBuf>,
    plan: BundledSidecarMaterializationPlan,
) -> Result<ResolvedBundledSidecarMaterializationPlan, DownloadActionError> {
    let resource_root = resource_root.into();

    if resource_root.as_os_str().is_empty() {
        return Err(DownloadActionError {
            code: "package-resource-root-missing",
            message: "application package resource root must not be empty".to_owned(),
        });
    }

    if resource_root
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(DownloadActionError {
            code: "package-resource-root-traversal",
            message: "application package resource root must not contain parent-directory traversal"
                .to_owned(),
        });
    }

    let mut resolved_items = Vec::with_capacity(plan.items.len());
    for item in plan.items {
        if item.source_resource.as_os_str().is_empty()
            || item
                .source_resource
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(DownloadActionError {
                code: "sidecar-resource-path-invalid",
                message:
                    "bundled sidecar resource must be a non-empty relative resource path".to_owned(),
            });
        }

        let source = resource_root.join(&item.source_resource);
        if !source.starts_with(&resource_root)
            || source
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(DownloadActionError {
                code: "sidecar-resource-path-escaped",
                message: "resolved bundled sidecar source must remain beneath the package resource root"
                    .to_owned(),
            });
        }

        if source == item.destination {
            return Err(DownloadActionError {
                code: "sidecar-source-destination-collision",
                message: "resolved bundled sidecar source and runtime destination must differ"
                    .to_owned(),
            });
        }

        resolved_items.push(ResolvedBundledSidecarMaterializationItem {
            kind: item.kind,
            source,
            destination: item.destination,
            identity: item.identity,
        });
    }

    Ok(ResolvedBundledSidecarMaterializationPlan {
        platform: plan.platform,
        resource_root,
        bin_root: plan.bin_root,
        items: resolved_items,
    })
}

fn resolve_packaged_sidecar_materialization_plan<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    layout: &AppRuntimeLayout,
) -> Result<ResolvedBundledSidecarMaterializationPlan, DownloadActionError> {
    let resource_root = app.path().resource_dir().map_err(|source| DownloadActionError {
        code: "package-resource-path-resolution-failed",
        message: format!("failed to resolve application package resource directory: {source}"),
    })?;

    let plan = build_bundled_sidecar_materialization_plan(layout)?;
    resolve_bundled_sidecar_sources(resource_root, plan)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedBundledSidecarArtifact {
    kind: BundledSidecarKind,
    source: PathBuf,
    destination: PathBuf,
    identity: BundledSidecarIdentity,
    byte_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedBundledSidecarMaterializationPlan {
    platform: &'static str,
    resource_root: PathBuf,
    bin_root: PathBuf,
    items: Vec<VerifiedBundledSidecarArtifact>,
}

fn sha256_file(path: &Path) -> Result<String, DownloadActionError> {
    let mut file = fs::File::open(path).map_err(|source| DownloadActionError {
        code: "sidecar-source-read-failed",
        message: format!("failed to open packaged sidecar source {}: {source}", path.display()),
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer).map_err(|source| DownloadActionError {
            code: "sidecar-source-read-failed",
            message: format!("failed to read packaged sidecar source {}: {source}", path.display()),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_packaged_sidecar_sources(
    plan: ResolvedBundledSidecarMaterializationPlan,
) -> Result<VerifiedBundledSidecarMaterializationPlan, DownloadActionError> {
    let root_metadata = fs::symlink_metadata(&plan.resource_root).map_err(|source| {
        DownloadActionError {
            code: "package-resource-root-inspection-failed",
            message: format!(
                "failed to inspect application package resource root {}: {source}",
                plan.resource_root.display()
            ),
        }
    })?;

    if root_metadata.file_type().is_symlink() {
        return Err(DownloadActionError {
            code: "package-resource-root-symlink",
            message: "application package resource root must not be a symlink".to_owned(),
        });
    }
    if !root_metadata.is_dir() {
        return Err(DownloadActionError {
            code: "package-resource-root-not-directory",
            message: "application package resource root must be a directory".to_owned(),
        });
    }

    let canonical_root = fs::canonicalize(&plan.resource_root).map_err(|source| {
        DownloadActionError {
            code: "package-resource-root-canonicalization-failed",
            message: format!(
                "failed to canonicalize application package resource root {}: {source}",
                plan.resource_root.display()
            ),
        }
    })?;

    if plan.items.len() != 3 {
        return Err(DownloadActionError {
            code: "sidecar-source-plan-invalid",
            message: "packaged sidecar validation plan must contain exactly Arti, yt-dlp, and FFmpeg"
                .to_owned(),
        });
    }

    let mut saw_arti = false;
    let mut saw_ytdlp = false;
    let mut saw_ffmpeg = false;
    let mut verified = Vec::with_capacity(plan.items.len());

    for item in plan.items {
        match item.kind {
            BundledSidecarKind::Arti if !saw_arti => saw_arti = true,
            BundledSidecarKind::YtDlp if !saw_ytdlp => saw_ytdlp = true,
            BundledSidecarKind::Ffmpeg if !saw_ffmpeg => saw_ffmpeg = true,
            _ => {
                return Err(DownloadActionError {
                    code: "sidecar-source-plan-invalid",
                    message:
                        "packaged sidecar validation plan must contain each supported sidecar once"
                            .to_owned(),
                });
            }
        }

        if !item.source.starts_with(&plan.resource_root) {
            return Err(DownloadActionError {
                code: "sidecar-source-path-escaped",
                message: "packaged sidecar source must remain beneath the package resource root"
                    .to_owned(),
            });
        }

        let relative = item
            .source
            .strip_prefix(&plan.resource_root)
            .map_err(|source| DownloadActionError {
                code: "sidecar-source-path-escaped",
                message: format!("failed to verify packaged sidecar source path: {source}"),
            })?;

        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(DownloadActionError {
                code: "sidecar-source-path-invalid",
                message: "packaged sidecar source must be a non-empty normal child path".to_owned(),
            });
        }

        let mut cursor = plan.resource_root.clone();
        let components = relative.components().collect::<Vec<_>>();
        for (index, component) in components.iter().enumerate() {
            let Component::Normal(part) = component else {
                return Err(DownloadActionError {
                    code: "sidecar-source-path-invalid",
                    message: "packaged sidecar source contains a non-normal path component".to_owned(),
                });
            };
            cursor.push(part);
            let is_final = index + 1 == components.len();

            let metadata = match fs::symlink_metadata(&cursor) {
                Ok(metadata) => metadata,
                Err(source) if source.kind() == io::ErrorKind::NotFound => {
                    return Err(DownloadActionError {
                        code: "sidecar-source-missing",
                        message: format!("packaged sidecar source is missing: {}", cursor.display()),
                    });
                }
                Err(source) => {
                    return Err(DownloadActionError {
                        code: "sidecar-source-inspection-failed",
                        message: format!(
                            "failed to inspect packaged sidecar source {}: {source}",
                            cursor.display()
                        ),
                    });
                }
            };

            if metadata.file_type().is_symlink() {
                return Err(DownloadActionError {
                    code: "sidecar-source-symlink",
                    message: format!(
                        "packaged sidecar source path must not traverse a symlink: {}",
                        cursor.display()
                    ),
                });
            }

            if is_final {
                if !metadata.is_file() {
                    return Err(DownloadActionError {
                        code: "sidecar-source-not-file",
                        message: format!(
                            "packaged sidecar source must be a regular file: {}",
                            cursor.display()
                        ),
                    });
                }
            } else if !metadata.is_dir() {
                return Err(DownloadActionError {
                    code: "sidecar-source-parent-not-directory",
                    message: format!(
                        "packaged sidecar source parent must be a directory: {}",
                        cursor.display()
                    ),
                });
            }
        }

        let canonical_source = fs::canonicalize(&item.source).map_err(|source| {
            DownloadActionError {
                code: "sidecar-source-canonicalization-failed",
                message: format!(
                    "failed to canonicalize packaged sidecar source {}: {source}",
                    item.source.display()
                ),
            }
        })?;

        if !canonical_source.starts_with(&canonical_root) {
            return Err(DownloadActionError {
                code: "sidecar-source-path-escaped",
                message: "canonical packaged sidecar source escaped the package resource root"
                    .to_owned(),
            });
        }

        let expected = item
            .identity
            .pinned_source_sha256
            .as_deref()
            .ok_or_else(|| DownloadActionError {
                code: "sidecar-source-hash-missing",
                message: "every packaged sidecar source requires a pinned SHA-256 identity"
                    .to_owned(),
            })?;
        let actual = sha256_file(&canonical_source)?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(DownloadActionError {
                code: "sidecar-source-hash-mismatch",
                message: format!(
                    "packaged sidecar source SHA-256 mismatch for {}",
                    canonical_source.display()
                ),
            });
        }

        let byte_size = fs::metadata(&canonical_source)
            .map_err(|source| DownloadActionError {
                code: "sidecar-source-inspection-failed",
                message: format!(
                    "failed to read packaged sidecar source metadata {}: {source}",
                    canonical_source.display()
                ),
            })?
            .len();

        verified.push(VerifiedBundledSidecarArtifact {
            kind: item.kind,
            source: canonical_source,
            destination: item.destination,
            identity: item.identity,
            byte_size,
        });
    }

    if !(saw_arti && saw_ytdlp && saw_ffmpeg) {
        return Err(DownloadActionError {
            code: "sidecar-source-plan-invalid",
            message: "packaged sidecar validation plan must contain Arti, yt-dlp, and FFmpeg"
                .to_owned(),
        });
    }

    Ok(VerifiedBundledSidecarMaterializationPlan {
        platform: plan.platform,
        resource_root: canonical_root,
        bin_root: plan.bin_root,
        items: verified,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedAppRuntimeDirectories {
    layout: AppRuntimeLayout,
}

impl PreparedAppRuntimeDirectories {
    fn layout(&self) -> &AppRuntimeLayout {
        &self.layout
    }

    fn into_layout(self) -> AppRuntimeLayout {
        self.layout
    }
}

fn runtime_directory_error(
    code: &'static str,
    operation: &str,
    path: &Path,
    source: impl std::fmt::Display,
) -> DownloadActionError {
    DownloadActionError {
        code,
        message: format!("{operation} {}: {source}", path.display()),
    }
}

fn reject_existing_runtime_path_hazards(
    layout: &AppRuntimeLayout,
    target: &Path,
) -> Result<(), DownloadActionError> {
    if !target.starts_with(layout.root()) {
        return Err(DownloadActionError {
            code: "runtime-directory-path-escaped",
            message: format!(
                "runtime directory target escaped the verified root: {}",
                target.display()
            ),
        });
    }

    let mut cursor = layout.root().to_path_buf();
    let relative = target.strip_prefix(layout.root()).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-path-escaped",
            "failed to verify runtime directory target",
            target,
            source,
        )
    })?;

    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(DownloadActionError {
                code: "runtime-directory-path-invalid",
                message: format!(
                    "runtime directory target contains a non-normal path component: {}",
                    target.display()
                ),
            });
        };
        cursor.push(name);

        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(DownloadActionError {
                    code: "runtime-directory-symlink",
                    message: format!(
                        "runtime directory preparation refuses symlink path: {}",
                        cursor.display()
                    ),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(DownloadActionError {
                    code: "runtime-directory-not-directory",
                    message: format!(
                        "runtime directory preparation found a non-directory path: {}",
                        cursor.display()
                    ),
                });
            }
            Ok(_) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => break,
            Err(source) => {
                return Err(runtime_directory_error(
                    "runtime-directory-inspection-failed",
                    "failed to inspect runtime directory path",
                    &cursor,
                    source,
                ));
            }
        }
    }

    Ok(())
}

fn create_verified_runtime_directory(
    layout: &AppRuntimeLayout,
    target: &Path,
    canonical_root: &Path,
) -> Result<(), DownloadActionError> {
    reject_existing_runtime_path_hazards(layout, target)?;

    fs::create_dir_all(target).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-preparation-failed",
            "failed to create runtime directory",
            target,
            source,
        )
    })?;

    let metadata = fs::symlink_metadata(target).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-inspection-failed",
            "failed to inspect prepared runtime directory",
            target,
            source,
        )
    })?;

    if metadata.file_type().is_symlink() {
        return Err(DownloadActionError {
            code: "runtime-directory-symlink",
            message: format!(
                "runtime directory preparation refuses symlink path: {}",
                target.display()
            ),
        });
    }

    if !metadata.is_dir() {
        return Err(DownloadActionError {
            code: "runtime-directory-not-directory",
            message: format!(
                "runtime directory preparation requires a directory: {}",
                target.display()
            ),
        });
    }

    let canonical_target = fs::canonicalize(target).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-canonicalization-failed",
            "failed to canonicalize prepared runtime directory",
            target,
            source,
        )
    })?;

    if !canonical_target.starts_with(canonical_root) {
        return Err(DownloadActionError {
            code: "runtime-directory-path-escaped",
            message: format!(
                "prepared runtime directory escaped the verified runtime root: {}",
                target.display()
            ),
        });
    }

    Ok(())
}

fn prepare_app_runtime_directories(
    layout: AppRuntimeLayout,
) -> Result<PreparedAppRuntimeDirectories, DownloadActionError> {
    match fs::symlink_metadata(layout.root()) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(DownloadActionError {
                code: "runtime-directory-symlink",
                message: format!(
                    "runtime directory preparation refuses symlink root: {}",
                    layout.root().display()
                ),
            });
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(DownloadActionError {
                code: "runtime-directory-not-directory",
                message: format!(
                    "runtime directory root is not a directory: {}",
                    layout.root().display()
                ),
            });
        }
        Ok(_) => {}
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(runtime_directory_error(
                "runtime-directory-inspection-failed",
                "failed to inspect runtime root",
                layout.root(),
                source,
            ));
        }
    }

    fs::create_dir_all(layout.root()).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-preparation-failed",
            "failed to create runtime root",
            layout.root(),
            source,
        )
    })?;

    let canonical_root = fs::canonicalize(layout.root()).map_err(|source| {
        runtime_directory_error(
            "runtime-directory-canonicalization-failed",
            "failed to canonicalize runtime root",
            layout.root(),
            source,
        )
    })?;

    let tor_config_parent = layout
        .tor_config_file()?
        .parent()
        .ok_or_else(|| DownloadActionError {
            code: "runtime-directory-path-invalid",
            message: "Tor config path must have a parent directory".to_owned(),
        })?
        .to_path_buf();

    for target in [
        tor_config_parent,
        layout.tor_cache_dir()?,
        layout.tor_state_dir()?,
        layout.output_root()?,
    ] {
        create_verified_runtime_directory(&layout, &target, &canonical_root)?;
    }

    Ok(PreparedAppRuntimeDirectories { layout })
}

fn select_verified_ytdlp_artifact(
    plan: &VerifiedBundledSidecarMaterializationPlan,
) -> Result<&VerifiedBundledSidecarArtifact, DownloadActionError> {
    let mut matches = plan
        .items
        .iter()
        .filter(|item| item.kind == BundledSidecarKind::YtDlp);
    let artifact = matches.next().ok_or_else(|| DownloadActionError {
        code: "verified-ytdlp-artifact-missing",
        message: "validated packaged sidecar plan must contain exactly one yt-dlp artifact".to_owned(),
    })?;

    if matches.next().is_some() {
        return Err(DownloadActionError {
            code: "verified-ytdlp-artifact-duplicate",
            message: "validated packaged sidecar plan contains more than one yt-dlp artifact"
                .to_owned(),
        });
    }

    Ok(artifact)
}

fn prepare_materialized_ytdlp_prelaunch<F>(
    layout: AppRuntimeLayout,
    plan: &VerifiedBundledSidecarMaterializationPlan,
    materialize: F,
) -> Result<(AppRuntimeLayout, PathBuf), DownloadActionError>
where
    F: FnOnce(
        &PreparedAppRuntimeDirectories,
        &Path,
        &VerifiedBundledSidecarArtifact,
    ) -> Result<PathBuf, DownloadActionError>,
{
    let artifact = select_verified_ytdlp_artifact(plan)?;
    let prepared = prepare_app_runtime_directories(layout)?;
    let expected = prepared.layout().ytdlp_executable()?;
    let materialized = materialize(&prepared, &plan.resource_root, artifact)?;

    if materialized != expected {
        return Err(DownloadActionError {
            code: "materialized-ytdlp-path-mismatch",
            message:
                "materialized yt-dlp path must equal the app-owned runtime layout destination"
                    .to_owned(),
        });
    }

    Ok((prepared.into_layout(), materialized))
}

fn select_verified_arti_artifact(
    plan: &VerifiedBundledSidecarMaterializationPlan,
) -> Result<&VerifiedBundledSidecarArtifact, DownloadActionError> {
    let mut matches = plan
        .items
        .iter()
        .filter(|item| item.kind == BundledSidecarKind::Arti);
    let artifact = matches.next().ok_or_else(|| DownloadActionError {
        code: "verified-arti-artifact-missing",
        message: "validated packaged sidecar plan must contain exactly one Arti artifact".to_owned(),
    })?;

    if matches.next().is_some() {
        return Err(DownloadActionError {
            code: "verified-arti-artifact-duplicate",
            message: "validated packaged sidecar plan contains more than one Arti artifact".to_owned(),
        });
    }

    Ok(artifact)
}

fn prepare_materialized_arti_prelaunch<F>(
    layout: AppRuntimeLayout,
    plan: &VerifiedBundledSidecarMaterializationPlan,
    materialize: F,
) -> Result<(AppRuntimeLayout, PathBuf), DownloadActionError>
where
    F: FnOnce(
        &PreparedAppRuntimeDirectories,
        &Path,
        &VerifiedBundledSidecarArtifact,
    ) -> Result<PathBuf, DownloadActionError>,
{
    let artifact = select_verified_arti_artifact(plan)?;
    let prepared = prepare_app_runtime_directories(layout)?;
    let expected = prepared.layout().arti_executable()?;
    let materialized = materialize(&prepared, &plan.resource_root, artifact)?;

    if materialized != expected {
        return Err(DownloadActionError {
            code: "materialized-arti-path-mismatch",
            message: "materialized Arti path must equal the app-owned runtime layout destination"
                .to_owned(),
        });
    }

    Ok((prepared.into_layout(), materialized))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedFilePipelineInputs {
    download: DownloadPreflightInputs,
    ffmpeg_executable: PathBuf,
    tor_ready_timeout: Duration,
}

impl CompletedFilePipelineInputs {
    fn new(
        download: DownloadPreflightInputs,
        ffmpeg_executable: impl Into<PathBuf>,
        tor_ready_timeout: Duration,
    ) -> Result<Self, DownloadActionError> {
        let ffmpeg_executable = ffmpeg_executable.into();
        if ffmpeg_executable.as_os_str().is_empty() {
            return Err(DownloadActionError {
                code: "missing-ffmpeg-executable",
                message: "FFmpeg executable must not be empty".to_owned(),
            });
        }

        Ok(Self {
            download,
            ffmpeg_executable,
            tor_ready_timeout,
        })
    }
}

fn run_completed_file_with_materialized_sidecars<FY, FA, FP>(
    layout: AppRuntimeLayout,
    verified_sidecar_plan: &VerifiedBundledSidecarMaterializationPlan,
    candidate: &SearchCandidate,
    inputs: CompletedFilePipelineInputs,
    materialize_ytdlp: FY,
    materialize_arti: FA,
    run_pipeline: FP,
) -> Result<CompletedFileView, DownloadActionError>
where
    FY: FnOnce(
        &PreparedAppRuntimeDirectories,
        &Path,
        &VerifiedBundledSidecarArtifact,
    ) -> Result<PathBuf, DownloadActionError>,
    FA: FnOnce(
        &PreparedAppRuntimeDirectories,
        &Path,
        &VerifiedBundledSidecarArtifact,
    ) -> Result<PathBuf, DownloadActionError>,
    FP: FnOnce(
        &SearchCandidate,
        CompletedFilePipelineInputs,
    ) -> Result<CompletedFileView, DownloadActionError>,
{
    let (verified_layout, materialized_ytdlp) =
        prepare_materialized_ytdlp_prelaunch(layout, verified_sidecar_plan, materialize_ytdlp)?;

    if materialized_ytdlp != inputs.download.ytdlp_executable
        || materialized_ytdlp != verified_layout.ytdlp_executable()?
    {
        return Err(DownloadActionError {
            code: "completed-file-ytdlp-path-mismatch",
            message: "completed-file pipeline yt-dlp path must equal the verified materialized path"
                .to_owned(),
        });
    }

    let (verified_layout, materialized_arti) =
        prepare_materialized_arti_prelaunch(verified_layout, verified_sidecar_plan, materialize_arti)?;

    if materialized_arti != inputs.download.arti_executable
        || materialized_arti != verified_layout.arti_executable()?
    {
        return Err(DownloadActionError {
            code: "completed-file-arti-path-mismatch",
            message: "completed-file pipeline Arti path must equal the verified materialized path"
                .to_owned(),
        });
    }

    run_pipeline(candidate, inputs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadPreflightSpec {
    arti_runtime: ArtiRuntimePlan,
    ytdlp_executable: PathBuf,
    media_source: YtDlpMediaSourceUrl,
    output_root: PathBuf,
}

impl DownloadPreflightSpec {
    fn arti_runtime(&self) -> &ArtiRuntimePlan {
        &self.arti_runtime
    }

    fn ytdlp_executable(&self) -> &Path {
        &self.ytdlp_executable
    }

    fn media_source(&self) -> &YtDlpMediaSourceUrl {
        &self.media_source
    }

    fn output_root(&self) -> &Path {
        &self.output_root
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedDownloadRuntime {
    arti_runtime: PreparedArtiRuntime,
    ytdlp_executable: PathBuf,    media_source: YtDlpMediaSourceUrl,
    output_root: PathBuf,
}

impl PreparedDownloadRuntime {
    fn arti_runtime(&self) -> &PreparedArtiRuntime {
        &self.arti_runtime
    }

    fn ytdlp_executable(&self) -> &Path {
        &self.ytdlp_executable
    }

    fn media_source(&self) -> &YtDlpMediaSourceUrl {
        &self.media_source
    }

    fn output_root(&self) -> &Path {
        &self.output_root
    }
}

#[derive(Debug)]
struct TorReadyDownloadRuntime {
    running_arti: RunningArti,
    transport: ReadyTorTransport,
    ytdlp_executable: PathBuf,
    media_source: YtDlpMediaSourceUrl,
    output_root: PathBuf,
}

impl TorReadyDownloadRuntime {
    fn transport(&self) -> ReadyTorTransport {
        self.transport
    }

    fn ytdlp_executable(&self) -> &Path {
        &self.ytdlp_executable
    }

    fn media_source(&self) -> &YtDlpMediaSourceUrl {
        &self.media_source
    }

    fn output_root(&self) -> &Path {
        &self.output_root
    }

    fn stop_and_wait(self) -> Result<(), DownloadActionError> {
        self.running_arti
            .stop_and_wait()
            .map(|_| ())
            .map_err(DownloadActionError::from)
    }
}

#[derive(Debug)]
struct TorGatedMediaRequestRuntime {
    running_arti: RunningArti,
    request: YtDlpMediaRequestPlan,
}

impl TorGatedMediaRequestRuntime {
    fn request(&self) -> &YtDlpMediaRequestPlan {
        &self.request
    }

    fn stop_and_wait(self) -> Result<(), DownloadActionError> {
        self.running_arti
            .stop_and_wait()
            .map(|_| ())
            .map_err(DownloadActionError::from)
    }
}

#[derive(Debug)]
struct RunningMediaDownloadRuntime {
    running_arti: RunningArti,
    running_ytdlp: RunningYtDlp,
}

impl RunningMediaDownloadRuntime {
    fn stop_and_wait(self) -> Result<(), DownloadActionError> {
        let RunningMediaDownloadRuntime {
            running_arti,
            running_ytdlp,
        } = self;

        let ytdlp_result = running_ytdlp.stop_and_wait();
        let arti_result = running_arti.stop_and_wait();

        match (ytdlp_result, arti_result) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(source), Ok(_)) => Err(DownloadActionError::from(source)),
            (Ok(_), Err(source)) => Err(DownloadActionError::from(source)),
            (Err(ytdlp_error), Err(arti_error)) => Err(DownloadActionError {
                code: "media-runtime-cleanup-failed",
                message: format!(
                    "yt-dlp cleanup failed: {ytdlp_error}; Arti cleanup failed: {arti_error}"
                ),
            }),
        }
    }
}

fn complete_media_download(
    runtime: RunningMediaDownloadRuntime,
) -> Result<CompletedDownloadResult, DownloadActionError> {
    let RunningMediaDownloadRuntime {
        running_arti,
        running_ytdlp,
    } = runtime;

    let completion = running_ytdlp.complete_download();
    let arti_cleanup = running_arti.stop_and_wait();

    match (completion, arti_cleanup) {
        (Ok(result), Ok(_)) => Ok(result),
        (Err(source), Ok(_)) => Err(DownloadActionError::from(source)),
        (Ok(_), Err(cleanup_error)) => Err(DownloadActionError {
            code: "arti-cleanup-after-download-failed",
            message: cleanup_error.to_string(),
        }),
        (Err(completion_error), Err(cleanup_error)) => Err(DownloadActionError {
            code: "download-completion-cleanup-failed",
            message: format!(
                "yt-dlp completion failed: {completion_error}; Arti cleanup failed: {cleanup_error}"
            ),
        }),
    }
}

fn derive_ffmpeg_remux_output_path(
    completed: &CompletedDownloadResult,
) -> Result<PathBuf, DownloadActionError> {
    let input = completed.artifact_path();
    let file_name = input.file_name().ok_or_else(|| DownloadActionError {
        code: "ffmpeg-output-derivation-failed",
        message: "validated completed artifact path must include a file name".to_owned(),
    })?;

    let mut output_name = OsString::from("pulqva-remux-");
    output_name.push(file_name);
    let mut output = input.with_file_name(output_name);
    output.set_extension("mp4");
    Ok(output)
}

fn build_ffmpeg_remux_plan(
    completed: &CompletedDownloadResult,
    ffmpeg_executable: impl Into<PathBuf>,
) -> Result<FfmpegRemuxPlan, DownloadActionError> {
    let output = derive_ffmpeg_remux_output_path(completed)?;

    FfmpegRemuxPlan::new(
        ffmpeg_executable,
        completed,
        output,
        FfmpegRemuxContainer::Mp4,
    )
    .map_err(DownloadActionError::from)
}

#[derive(Debug)]
struct RunningFfmpegRemuxRuntime {
    running_ffmpeg: RunningFfmpeg,
}

impl RunningFfmpegRemuxRuntime {
    fn stop_and_wait(self) -> Result<(), DownloadActionError> {
        self.running_ffmpeg
            .stop_and_wait()
            .map(|_| ())
            .map_err(DownloadActionError::from)
    }
}

fn launch_ffmpeg_remux_runtime(
    plan: FfmpegRemuxPlan,
) -> Result<RunningFfmpegRemuxRuntime, DownloadActionError> {
    let running_ffmpeg = launch_ffmpeg_remux(plan).map_err(DownloadActionError::from)?;
    Ok(RunningFfmpegRemuxRuntime { running_ffmpeg })
}

fn complete_ffmpeg_remux_runtime(
    runtime: RunningFfmpegRemuxRuntime,
) -> Result<CompletedFfmpegRemuxResult, DownloadActionError> {
    runtime
        .running_ffmpeg
        .complete_remux()
        .map_err(DownloadActionError::from)
}

fn run_completed_file_pipeline(
    candidate: &SearchCandidate,
    inputs: CompletedFilePipelineInputs,
) -> Result<CompletedFileView, DownloadActionError> {
    let CompletedFilePipelineInputs {
        download,
        ffmpeg_executable,
        tor_ready_timeout,
    } = inputs;

    let preflight = build_download_preflight(candidate, download)?;
    let prepared = prepare_download_runtime(preflight)?;
    let tor_ready = establish_tor_ready_download_runtime(prepared, tor_ready_timeout)?;
    let media_request = build_tor_gated_media_request(tor_ready)?;
    let running_download = launch_tor_gated_media_request(media_request)?;
    let completed_download = complete_media_download(running_download)?;
    let remux_plan = build_ffmpeg_remux_plan(&completed_download, ffmpeg_executable)?;
    let running_remux = launch_ffmpeg_remux_runtime(remux_plan)?;
    let completed_remux = complete_ffmpeg_remux_runtime(running_remux)?;
    sanitized_completed_file_view(&completed_remux)
}

fn local_candidate_source() -> Result<Vec<SearchCandidate>, SearchCandidateError> {
    Ok(vec![
        SearchCandidate::new(
            "Official live performance",
            "local:test:candidate:official-live",
        )?,
        SearchCandidate::new(
            "Archive performance",
            "local:test:candidate:archive-performance",
        )?,
    ])
}

fn local_candidate_by_locator(
    locator: &str,
) -> Result<Option<SearchCandidate>, SearchCandidateError> {
    Ok(local_candidate_source()?
        .into_iter()
        .find(|candidate| candidate.locator() == locator))
}

fn resolve_backend_media_source(
    candidate: &SearchCandidate,
) -> Result<YtDlpMediaSourceUrl, DownloadActionError> {
    if candidate.locator() != T024_PROOF_LOCATOR {
        return Err(DownloadActionError {
            code: "media-source-unsupported",
            message: "validated candidate has no approved backend media-source mapping".to_owned(),
        });
    }

    YtDlpMediaSourceUrl::parse(T024_MEDIA_SOURCE_URL).map_err(DownloadActionError::from)
}

fn build_download_preflight(
    candidate: &SearchCandidate,
    inputs: DownloadPreflightInputs,
) -> Result<DownloadPreflightSpec, DownloadActionError> {
    let media_source = resolve_backend_media_source(candidate)?;
    let socks = TorSocksEndpoint::new(inputs.socks_port).map_err(DownloadActionError::from)?;
    let arti_runtime = ArtiRuntimePlan::new(
        inputs.arti_executable,
        inputs.tor_config_file,
        inputs.tor_cache_dir,
        inputs.tor_state_dir,
        socks,
    );

    Ok(DownloadPreflightSpec {
        arti_runtime,
        ytdlp_executable: inputs.ytdlp_executable,
        media_source,
        output_root: inputs.output_root,
    })
}

fn prepare_download_runtime(
    preflight: DownloadPreflightSpec,
) -> Result<PreparedDownloadRuntime, DownloadActionError> {
    let DownloadPreflightSpec {
        arti_runtime,
        ytdlp_executable,
        media_source,
        output_root,
    } = preflight;

    let arti_runtime = prepare_arti_runtime(arti_runtime).map_err(DownloadActionError::from)?;

    Ok(PreparedDownloadRuntime {
        arti_runtime,
        ytdlp_executable,
        media_source,
        output_root,
    })
}

fn establish_tor_ready_download_runtime(
    prepared: PreparedDownloadRuntime,
    timeout: Duration,
) -> Result<TorReadyDownloadRuntime, DownloadActionError> {
    let PreparedDownloadRuntime {
        arti_runtime,
        ytdlp_executable,
        media_source,
        output_root,
    } = prepared;

    let mut running_arti = launch_prepared_arti(arti_runtime).map_err(DownloadActionError::from)?;

    let transport = match verify_tor_readiness(&mut running_arti, timeout) {
        Ok(transport) => transport,
        Err(source) => {
            let cleanup = running_arti.stop_and_wait();
            return match cleanup {
                Ok(_) => Err(DownloadActionError {
                    code: "tor-readiness-failed",
                    message: source.to_string(),
                }),
                Err(cleanup_error) => Err(DownloadActionError {
                    code: "tor-readiness-cleanup-failed",
                    message: format!(
                        "Tor readiness failed: {source}; Arti cleanup failed: {cleanup_error}"
                    ),
                }),
            };
        }
    };

    Ok(TorReadyDownloadRuntime {
        running_arti,
        transport,
        ytdlp_executable,
        media_source,
        output_root,
    })
}

fn build_tor_gated_media_request(
    runtime: TorReadyDownloadRuntime,
) -> Result<TorGatedMediaRequestRuntime, DownloadActionError> {
    let TorReadyDownloadRuntime {
        running_arti,
        transport,
        ytdlp_executable,
        media_source,
        output_root,
    } = runtime;

    match YtDlpMediaRequestPlan::new_tor_gated(
        ytdlp_executable,
        transport,
        media_source,
        output_root,
    ) {
        Ok(request) => Ok(TorGatedMediaRequestRuntime {
            running_arti,
            request,
        }),
        Err(source) => {
            let cleanup = running_arti.stop_and_wait();
            match cleanup {
                Ok(_) => Err(DownloadActionError::from(source)),
                Err(cleanup_error) => Err(DownloadActionError {
                    code: "media-request-cleanup-failed",
                    message: format!(
                        "media request planning failed: {source}; Arti cleanup failed: {cleanup_error}"
                    ),
                }),
            }
        }
    }
}

fn launch_tor_gated_media_request(
    runtime: TorGatedMediaRequestRuntime,
) -> Result<RunningMediaDownloadRuntime, DownloadActionError> {
    let TorGatedMediaRequestRuntime {
        running_arti,
        request,
    } = runtime;

    match launch_ytdlp_request(request) {
        Ok(running_ytdlp) => Ok(RunningMediaDownloadRuntime {
            running_arti,
            running_ytdlp,
        }),
        Err(source) => {
            let cleanup = running_arti.stop_and_wait();
            match cleanup {
                Ok(_) => Err(DownloadActionError::from(source)),
                Err(cleanup_error) => Err(DownloadActionError {
                    code: "ytdlp-launch-cleanup-failed",
                    message: format!(
                        "yt-dlp launch failed: {source}; Arti cleanup failed: {cleanup_error}"
                    ),
                }),
            }
        }
    }
}

fn runtime_layout_from_app_data_dir(
    app_data_dir: impl Into<PathBuf>,
) -> Result<AppRuntimeLayout, DownloadActionError> {
    AppRuntimeLayout::new(app_data_dir.into().join("runtime"))
}

fn resolve_app_runtime_layout<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<AppRuntimeLayout, DownloadActionError> {
    let app_data_dir = app.path().app_data_dir().map_err(|source| DownloadActionError {
        code: "app-data-path-resolution-failed",
        message: format!("failed to resolve application data directory: {source}"),
    })?;

    runtime_layout_from_app_data_dir(app_data_dir)
}

fn prepare_completed_file_command(
    query: String,
    locator: String,
    layout: &AppRuntimeLayout,
) -> Result<(SearchCandidate, CompletedFilePipelineInputs), DownloadActionError> {
    let _intent = SearchIntent::new(query).map_err(DownloadActionError::from)?;
    let candidate = local_candidate_by_locator(&locator)
        .map_err(DownloadActionError::from)?
        .ok_or_else(|| DownloadActionError {
            code: "candidate-not-found",
            message: "candidate locator is not present in the validated local candidate set".to_owned(),
        })?;

    // Reject validated-but-unsupported candidates before any blocking/runtime work starts.
    let _approved_source = resolve_backend_media_source(&candidate)?;
    let inputs = layout.completed_file_pipeline_inputs()?;
    Ok((candidate, inputs))
}

#[tauri::command]
fn app_status() -> AppStatus {
    AppStatus {
        product_name: PRODUCT_NAME,
        app_version: env!("CARGO_PKG_VERSION"),
        kernel_version: KERNEL_VERSION.trim(),
        core_ready: CORE_CRATE_READY,
        privacy_mode: "tor-required-fail-closed",
    }
}

#[tauri::command]
fn submit_intent(query: String) -> Result<IntentSubmission, IntentSubmissionError> {
    let intent = SearchIntent::new(query).map_err(IntentSubmissionError::from)?;

    Ok(IntentSubmission {
        query: intent.query().to_owned(),
        stage: "validated-search-intent",
    })
}

#[tauri::command]
fn list_local_candidates(query: String) -> Result<CandidateList, CandidateListError> {
    let intent = SearchIntent::new(query).map_err(CandidateListError::from)?;
    let candidates = local_candidate_source()
        .map_err(CandidateListError::from)?
        .iter()
        .map(DesktopCandidate::from)
        .collect();

    Ok(CandidateList {
        intent_query: intent.query().to_owned(),
        stage: "local-candidate-list",
        candidates,
    })
}

#[tauri::command]
fn select_local_candidate(
    query: String,
    locator: String,
) -> Result<CandidateSelection, CandidateSelectionError> {
    let intent = SearchIntent::new(query).map_err(CandidateSelectionError::from)?;
    let candidate = local_candidate_by_locator(&locator)
        .map_err(CandidateSelectionError::from)?
        .ok_or_else(|| CandidateSelectionError {
            code: "candidate-not-found",
            message: "candidate locator is not present in the validated local candidate set".to_owned(),
        })?;

    Ok(CandidateSelection {
        intent_query: intent.query().to_owned(),
        title: candidate.title().to_owned(),
        locator: candidate.locator().to_owned(),
        stage: "candidate-selected",
    })
}

/// Data-only Download boundary.
///
/// The opaque locator is revalidated against the deterministic local candidate
/// set and is deliberately not interpreted as a URL or executable instruction.
#[tauri::command]
fn plan_local_download(
    query: String,
    locator: String,
) -> Result<DownloadAction, DownloadActionError> {
    let intent = SearchIntent::new(query).map_err(DownloadActionError::from)?;
    let candidate = local_candidate_by_locator(&locator)
        .map_err(DownloadActionError::from)?
        .ok_or_else(|| DownloadActionError {
            code: "candidate-not-found",
            message: "candidate locator is not present in the validated local candidate set".to_owned(),
        })?;
    let _approved_source = resolve_backend_media_source(&candidate)?;

    Ok(DownloadAction {
        intent_query: intent.query().to_owned(),
        title: candidate.title().to_owned(),
        locator: candidate.locator().to_owned(),
        action: "download",
        stage: "download-action-planned",
        media_source_ready: true,
        media_source_stage: "backend-media-source-resolved",
    })
}

#[tauri::command]
async fn download_completed_file(
    query: String,
    locator: String,
    app: tauri::AppHandle,
) -> Result<CompletedFileView, DownloadActionError> {
    let layout = resolve_app_runtime_layout(&app)?;
    let sidecar_plan = resolve_packaged_sidecar_materialization_plan(&app, &layout)?;
    let (candidate, inputs) = prepare_completed_file_command(query, locator, &layout)?;

    tauri::async_runtime::spawn_blocking(move || {
        let verified_sidecar_plan = validate_packaged_sidecar_sources(sidecar_plan)?;
        run_completed_file_with_materialized_sidecars(
            layout,
            &verified_sidecar_plan,
            &candidate,
            inputs,
            |prepared, resource_root, artifact| {
                ytdlp_materialize::materialize_verified_ytdlp(
                    prepared,
                    resource_root,
                    artifact,
                )
                .map(|result| result.path().to_path_buf())
            },
            |prepared, resource_root, artifact| {
                arti_materialize::materialize_verified_arti(
                    prepared,
                    resource_root,
                    artifact,
                )
                .map(|result| result.path().to_path_buf())
            },
            run_completed_file_pipeline,
        )
    })
        .await
        .map_err(|source| DownloadActionError {
            code: "completed-file-task-join-failed",
            message: format!("completed-file backend task failed to join: {source}"),
        })?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_status,
            submit_intent,
            list_local_candidates,
            select_local_candidate,
            plan_local_download,
            download_completed_file
        ])
        .run(tauri::generate_context!())
        .expect("PULQVA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{
        AppRuntimeLayout, BundledSidecarKind, DEFAULT_TOR_READY_TIMEOUT_SECS,
        DEFAULT_TOR_SOCKS_PORT, DownloadActionError, DownloadPreflightInputs, T024_MEDIA_SOURCE_URL,
        BundledSidecarIdentity, ResolvedBundledSidecarMaterializationItem,
        ResolvedBundledSidecarMaterializationPlan, VerifiedBundledSidecarArtifact,
        VerifiedBundledSidecarMaterializationPlan, T024_PROOF_LOCATOR, app_status,
        build_bundled_sidecar_materialization_plan, pinned_arti_sha256_for_platform,
        prepare_materialized_arti_prelaunch, prepare_materialized_ytdlp_prelaunch,
        resolve_bundled_sidecar_sources, run_completed_file_with_materialized_sidecars,
        validate_packaged_sidecar_sources, CompletedFileView,
        build_download_preflight, establish_tor_ready_download_runtime, list_local_candidates,
        local_candidate_by_locator, plan_local_download, prepare_app_runtime_directories,
        prepare_completed_file_command, prepare_download_runtime, resolve_backend_media_source,
        runtime_layout_from_app_data_dir, select_local_candidate, submit_intent,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-desktop-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn typed_status_contract_reports_linked_core_and_kernel() {
        let status = app_status();

        assert_eq!(status.product_name, "PULQVA");
        assert_eq!(status.app_version, "0.0.0");
        assert_eq!(status.kernel_version, "1.0.0");
        assert!(status.core_ready);
        assert_eq!(status.privacy_mode, "tor-required-fail-closed");
    }

    #[test]
    fn desktop_intent_command_preserves_validated_raw_text() {
        let submission =
            submit_intent("find the official live performance".to_owned()).expect("valid intent");

        assert_eq!(submission.query, "find the official live performance");
        assert_eq!(submission.stage, "validated-search-intent");
    }

    #[test]
    fn desktop_intent_command_rejects_whitespace_only_input() {
        let error = submit_intent("  \t\n  ".to_owned()).expect_err("blank intent must fail");

        assert_eq!(error.code, "invalid-search-intent");
        assert_eq!(error.message, "search query must not be empty");
    }

    #[test]
    fn local_candidate_command_is_deterministic_and_core_validated() {
        let result = list_local_candidates("find the official live performance".to_owned())
            .expect("validated intent gets local proof candidates");

        assert_eq!(result.intent_query, "find the official live performance");
        assert_eq!(result.stage, "local-candidate-list");
        assert_eq!(result.candidates.len(), 2);
        assert_eq!(result.candidates[0].title, "Official live performance");
        assert_eq!(
            result.candidates[0].locator,
            "local:test:candidate:official-live"
        );
        assert_eq!(result.candidates[1].title, "Archive performance");
        assert_eq!(
            result.candidates[1].locator,
            "local:test:candidate:archive-performance"
        );
    }

    #[test]
    fn local_candidate_command_rejects_blank_intent() {
        let error =
            list_local_candidates(" \n ".to_owned()).expect_err("blank intent must fail first");

        assert_eq!(error.code, "invalid-search-intent");
        assert_eq!(error.message, "search query must not be empty");
    }

    #[test]
    fn local_candidate_selection_requires_exact_validated_locator_match() {
        let selected = select_local_candidate(
            "find the official live performance".to_owned(),
            "local:test:candidate:official-live".to_owned(),
        )
        .expect("known local candidate must select");

        assert_eq!(selected.intent_query, "find the official live performance");
        assert_eq!(selected.title, "Official live performance");
        assert_eq!(selected.locator, "local:test:candidate:official-live");
        assert_eq!(selected.stage, "candidate-selected");
    }

    #[test]
    fn local_candidate_selection_rejects_unknown_locator() {
        let error = select_local_candidate(
            "find the official live performance".to_owned(),
            "local:test:candidate:not-returned".to_owned(),
        )
        .expect_err("unknown locator must fail closed");

        assert_eq!(error.code, "candidate-not-found");
        assert_eq!(
            error.message,
            "candidate locator is not present in the validated local candidate set"
        );
    }

    #[test]
    fn local_candidate_selection_rejects_blank_intent_before_matching_locator() {
        let error = select_local_candidate(
            "  \t ".to_owned(),
            "local:test:candidate:official-live".to_owned(),
        )
        .expect_err("blank intent must fail before selection");

        assert_eq!(error.code, "invalid-search-intent");
        assert_eq!(error.message, "search query must not be empty");
    }

    #[test]
    fn backend_download_preflight_is_typed_deterministic_and_side_effect_free() {
        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("supported proof candidate exists");
        let inputs = DownloadPreflightInputs::new(
            "bundle/arti",
            "bundle/yt-dlp",            "state/arti/pulqva.toml",
            "state/arti/cache",
            "state/arti/state",
            "downloads",
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect("explicit preflight inputs are valid");
        let preflight = build_download_preflight(&candidate, inputs)
            .expect("supported candidate builds a pure-data preflight");

        assert_eq!(preflight.arti_runtime().executable(), Path::new("bundle/arti"));
        assert_eq!(
            preflight.arti_runtime().config_file(),
            Path::new("state/arti/pulqva.toml")
        );
        assert_eq!(preflight.arti_runtime().cache_dir(), Path::new("state/arti/cache"));
        assert_eq!(preflight.arti_runtime().state_dir(), Path::new("state/arti/state"));
        assert_eq!(preflight.arti_runtime().socks_endpoint().port(), DEFAULT_TOR_SOCKS_PORT);
        assert_eq!(preflight.ytdlp_executable(), Path::new("bundle/yt-dlp"));
        assert_eq!(preflight.media_source().as_str(), T024_MEDIA_SOURCE_URL);
        assert_eq!(preflight.output_root(), Path::new("downloads"));
    }

    #[test]
    fn tor_ready_runtime_fails_closed_when_arti_cannot_launch() {
        let root = test_root("tor-ready-launch-fail");
        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("supported proof candidate exists");
        let inputs = DownloadPreflightInputs::new(
            root.join("missing/arti"),
            root.join("bin/yt-dlp"),
            root.join("config/pulqva.toml"),
            root.join("cache"),
            root.join("state"),
            root.join("downloads"),
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect("explicit preflight inputs are valid");
        let preflight = build_download_preflight(&candidate, inputs)
            .expect("supported candidate builds preflight");
        let prepared = prepare_download_runtime(preflight)
            .expect("config preparation succeeds before launch");

        let error = establish_tor_ready_download_runtime(prepared, Duration::from_secs(1))
            .expect_err("missing Arti executable must fail closed");

        assert_eq!(error.code, "arti-process-failed");
        assert!(!root.join("cache").exists());
        assert!(!root.join("state").exists());
        assert!(!root.join("downloads").exists());

        fs::remove_dir_all(root).expect("launch-fail test tree cleanup succeeds");
    }

    #[test]
    fn tor_ready_runtime_zero_timeout_cleans_up_launched_child() {
        let root = test_root("tor-ready-timeout");
        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("supported proof candidate exists");
        let current_exe = std::env::current_exe().expect("test executable path is available");
        let inputs = DownloadPreflightInputs::new(
            current_exe,
            root.join("bin/yt-dlp"),
            root.join("config/pulqva.toml"),
            root.join("cache"),
            root.join("state"),
            root.join("downloads"),
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect("explicit preflight inputs are valid");
        let preflight = build_download_preflight(&candidate, inputs)
            .expect("supported candidate builds preflight");
        let prepared = prepare_download_runtime(preflight)
            .expect("config preparation succeeds before readiness");

        let error = establish_tor_ready_download_runtime(prepared, Duration::ZERO)
            .expect_err("zero readiness timeout must fail closed and clean up child");

        assert_eq!(error.code, "tor-readiness-failed");
        assert_eq!(error.message, "Tor readiness verification timed out");
        assert!(!root.join("cache").exists());
        assert!(!root.join("state").exists());
        assert!(!root.join("downloads").exists());

        fs::remove_dir_all(root).expect("timeout test tree cleanup succeeds");
    }

    #[test]
    fn prepared_download_runtime_materializes_only_arti_config_and_retains_typed_state() {
        let root = test_root("prepared-runtime");
        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("supported proof candidate exists");
        let inputs = DownloadPreflightInputs::new(
            root.join("bin/arti"),
            root.join("bin/yt-dlp"),
            root.join("config/pulqva.toml"),
            root.join("cache"),
            root.join("state"),
            root.join("downloads"),
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect("explicit preflight inputs are valid");
        let preflight = build_download_preflight(&candidate, inputs)
            .expect("supported candidate builds preflight");
        let expected_config = preflight
            .arti_runtime()
            .render_config()
            .expect("test paths are UTF-8");

        let prepared = prepare_download_runtime(preflight)
            .expect("Arti config preparation succeeds without launching a process");

        assert_eq!(
            fs::read(prepared.arti_runtime().plan().config_file())
                .expect("prepared config exists"),
            expected_config.as_bytes()
        );
        assert_eq!(
            prepared.arti_runtime().plan().socks_endpoint().port(),
            DEFAULT_TOR_SOCKS_PORT
        );
        assert_eq!(prepared.ytdlp_executable(), root.join("bin/yt-dlp"));
        assert_eq!(prepared.media_source().as_str(), T024_MEDIA_SOURCE_URL);
        assert_eq!(prepared.output_root(), root.join("downloads"));
        assert!(!prepared.arti_runtime().plan().executable().exists());
        assert!(!prepared.arti_runtime().plan().cache_dir().exists());
        assert!(!prepared.arti_runtime().plan().state_dir().exists());
        assert!(!prepared.ytdlp_executable().exists());
        assert!(!prepared.output_root().exists());

        fs::remove_dir_all(root).expect("prepared test tree cleanup succeeds");
    }

    #[test]
    fn prepared_download_runtime_fails_closed_when_config_cannot_be_materialized() {
        let root = test_root("prepared-runtime-fail");
        fs::create_dir_all(&root).expect("test root creation succeeds");
        let blocked_parent = root.join("blocked-parent");
        fs::write(&blocked_parent, b"not a directory").expect("blocking file creation succeeds");

        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("supported proof candidate exists");
        let inputs = DownloadPreflightInputs::new(
            root.join("bin/arti"),
            root.join("bin/yt-dlp"),
            blocked_parent.join("pulqva.toml"),
            root.join("cache"),
            root.join("state"),
            root.join("downloads"),
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect("preflight paths are syntactically valid");
        let preflight = build_download_preflight(&candidate, inputs)
            .expect("supported candidate builds preflight");

        let error = prepare_download_runtime(preflight)
            .expect_err("materialization failure must return no prepared runtime");

        assert_eq!(error.code, "arti-runtime-preparation-failed");
        assert!(!root.join("cache").exists());
        assert!(!root.join("state").exists());
        assert!(!root.join("downloads").exists());

        fs::remove_dir_all(root).expect("failed preparation tree cleanup succeeds");
    }

    #[test]
    fn backend_download_preflight_rejects_missing_required_path() {
        let error = DownloadPreflightInputs::new(
            "",
            "bundle/yt-dlp",
            "state/arti/pulqva.toml",
            "state/arti/cache",
            "state/arti/state",
            "downloads",
            DEFAULT_TOR_SOCKS_PORT,
        )
        .expect_err("missing executable must fail closed");

        assert_eq!(error.code, "missing-preflight-path");
        assert_eq!(error.message, "arti executable must not be empty");
    }

    #[test]
    fn backend_download_preflight_rejects_zero_socks_port() {
        let error = DownloadPreflightInputs::new(
            "bundle/arti",
            "bundle/yt-dlp",
            "state/arti/pulqva.toml",
            "state/arti/cache",
            "state/arti/state",
            "downloads",
            0,
        )
        .expect_err("zero SOCKS port must fail closed");

        assert_eq!(error.code, "invalid-tor-socks-endpoint");
        assert_eq!(error.message, "Tor SOCKS port must be non-zero");
    }

    fn expected_runtime_executable(root: &Path, name: &str) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            return root.join("bin").join(format!("{name}.exe"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            root.join("bin").join(name)
        }
    }

    #[test]
    fn app_runtime_layout_derives_every_backend_path_beneath_one_root() {
        let layout = AppRuntimeLayout::new("runtime").expect("runtime root is valid");

        assert_eq!(layout.root(), Path::new("runtime"));
        assert_eq!(
            layout.arti_executable().unwrap(),
            expected_runtime_executable(Path::new("runtime"), "arti")
        );
        assert_eq!(
            layout.ytdlp_executable().unwrap(),
            expected_runtime_executable(Path::new("runtime"), "yt-dlp")
        );
        assert_eq!(
            layout.ffmpeg_executable().unwrap(),
            expected_runtime_executable(Path::new("runtime"), "ffmpeg")
        );
        assert_eq!(
            layout.tor_config_file().unwrap(),
            Path::new("runtime/arti/config/pulqva.toml")
        );
        assert_eq!(layout.tor_cache_dir().unwrap(), Path::new("runtime/arti/cache"));
        assert_eq!(layout.tor_state_dir().unwrap(), Path::new("runtime/arti/state"));
        assert_eq!(layout.output_root().unwrap(), Path::new("runtime/downloads"));

        for path in [
            layout.arti_executable().unwrap(),
            layout.ytdlp_executable().unwrap(),
            layout.ffmpeg_executable().unwrap(),
            layout.tor_config_file().unwrap(),
            layout.tor_cache_dir().unwrap(),
            layout.tor_state_dir().unwrap(),
            layout.output_root().unwrap(),
        ] {
            assert!(path.starts_with(layout.root()));
            assert!(!path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir)));
        }
    }

    #[test]
    fn app_runtime_layout_rejects_missing_or_traversing_root() {
        let missing = AppRuntimeLayout::new("").expect_err("empty root must fail closed");
        assert_eq!(missing.code, "runtime-layout-root-missing");

        let traversal =
            AppRuntimeLayout::new(PathBuf::from("runtime").join("..").join("escape"))
                .expect_err("parent traversal must fail closed");
        assert_eq!(traversal.code, "runtime-layout-root-traversal");
    }

    #[test]
    fn app_data_runtime_layout_places_runtime_beneath_resolved_app_data() {
        let app_data = PathBuf::from("os-data").join("app.pulqva.desktop");
        let layout =
            runtime_layout_from_app_data_dir(&app_data).expect("resolved app data path is valid");

        assert_eq!(layout.root(), app_data.join("runtime"));
        assert!(layout.root().starts_with(&app_data));
    }

    #[test]
    fn completed_file_command_preflight_uses_resolved_backend_runtime_layout() {
        let app_data = PathBuf::from("os-data").join("app.pulqva.desktop");
        let layout =
            runtime_layout_from_app_data_dir(&app_data).expect("resolved app data path is valid");
        let root = app_data.join("runtime");

        let (candidate, inputs) = prepare_completed_file_command(
            "find the official live performance".to_owned(),
            T024_PROOF_LOCATOR.to_owned(),
            &layout,
        )
        .expect("supported candidate prepares backend-owned inputs");

        assert_eq!(candidate.locator(), T024_PROOF_LOCATOR);
        assert_eq!(
            inputs.download.arti_executable,
            expected_runtime_executable(&root, "arti")
        );
        assert_eq!(
            inputs.download.ytdlp_executable,
            expected_runtime_executable(&root, "yt-dlp")
        );
        assert_eq!(
            inputs.download.tor_config_file,
            root.join("arti/config/pulqva.toml")
        );
        assert_eq!(inputs.download.tor_cache_dir, root.join("arti/cache"));
        assert_eq!(inputs.download.tor_state_dir, root.join("arti/state"));
        assert_eq!(inputs.download.output_root, root.join("downloads"));
        assert_eq!(
            inputs.ffmpeg_executable,
            expected_runtime_executable(&root, "ffmpeg")
        );
        assert_eq!(
            inputs.tor_ready_timeout,
            Duration::from_secs(DEFAULT_TOR_READY_TIMEOUT_SECS)
        );
    }

    #[test]
    fn arti_identity_parser_requires_one_well_formed_platform_entry() {
        let linux = pinned_arti_sha256_for_platform(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  linux-x86_64/arti\n",
            "linux-x86_64",
            "linux-x86_64/arti",
        )
        .expect("one exact Arti identity is accepted");
        assert_eq!(linux.len(), 64);

        let missing = pinned_arti_sha256_for_platform(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  windows-x86_64/arti.exe\n",
            "linux-x86_64",
            "linux-x86_64/arti",
        )
        .expect_err("missing platform identity must fail closed");
        assert_eq!(missing.code, "arti-source-identity-missing");

        let duplicate = pinned_arti_sha256_for_platform(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  linux-x86_64/arti\nbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  linux-x86_64/arti\n",
            "linux-x86_64",
            "linux-x86_64/arti",
        )
        .expect_err("duplicate platform identity must fail closed");
        assert_eq!(duplicate.code, "arti-source-identity-duplicate");

        let malformed = pinned_arti_sha256_for_platform(
            "not-a-digest  linux-x86_64/arti\n",
            "linux-x86_64",
            "linux-x86_64/arti",
        )
        .expect_err("malformed platform identity must fail closed");
        assert_eq!(malformed.code, "arti-source-identity-malformed");

        let wrong_platform = pinned_arti_sha256_for_platform(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  linux-x86_64/arti\n",
            "windows-x86_64",
            "linux-x86_64/arti",
        )
        .expect_err("wrong-platform asset binding must fail closed");
        assert_eq!(
            wrong_platform.code,
            "arti-source-identity-platform-mismatch"
        );
    }

    #[test]
    fn packaged_sidecar_source_validation_requires_arti_content_identity() {
        let root = test_root("arti-source-hash-required");
        let mut plan = source_validation_fixture_plan(&root, None);
        plan.items[0].identity.pinned_source_sha256 = None;

        let error = validate_packaged_sidecar_sources(plan)
            .expect_err("Arti without a content digest must fail closed");

        assert_eq!(error.code, "sidecar-source-hash-missing");
        assert!(!root.join("runtime").exists());

        fs::remove_dir_all(root).expect("Arti hash-required fixture cleanup succeeds");
    }

    #[test]
    fn bundled_sidecar_materialization_plan_is_typed_backend_owned_and_side_effect_free() {
        let root = test_root("sidecar-materialization-plan");
        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("runtime layout is valid");

        let plan = build_bundled_sidecar_materialization_plan(&layout)
            .expect("supported platform produces a typed sidecar plan");

        assert_eq!(plan.items.len(), 3);
        assert_eq!(plan.bin_root, layout.bin_root().expect("bin root derives"));
        assert!(!plan.bin_root.exists());

        assert_eq!(plan.items[0].kind, BundledSidecarKind::Arti);
        assert_eq!(plan.items[1].kind, BundledSidecarKind::YtDlp);
        assert_eq!(plan.items[2].kind, BundledSidecarKind::Ffmpeg);

        for item in &plan.items {
            assert!(item.destination.starts_with(&plan.bin_root));
            assert_eq!(item.destination.parent(), Some(plan.bin_root.as_path()));
            assert_ne!(item.source_resource, item.destination);
            assert!(!item.identity.version.is_empty());
            assert!(!item.source_resource.is_absolute());
        }

        assert_eq!(plan.items[0].identity.version, "2.6.0");
        assert_eq!(plan.items[1].identity.version, "2026.08.19");
        assert_eq!(plan.items[2].identity.version, "n9.0.2-3-ga5923073bf");
        assert_eq!(
            plan.items[0]
                .identity
                .pinned_source_sha256
                .as_deref()
                .expect("Arti direct binary has pinned digest")
                .len(),
            64
        );
        assert_eq!(
            plan.items[1]
                .identity
                .pinned_source_sha256
                .as_deref()
                .expect("yt-dlp direct binary has pinned digest")
                .len(),
            64
        );
        assert_eq!(
            plan.items[2]
                .identity
                .pinned_source_sha256
                .as_deref()
                .expect("FFmpeg source archive has pinned digest")
                .len(),
            64
        );

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            assert_eq!(plan.platform, "linux-x86_64");
            assert_eq!(
                plan.items[0].source_resource,
                Path::new("sidecars/arti/linux-x86_64/arti")
            );
            assert_eq!(
                plan.items[1].source_resource,
                Path::new("sidecars/yt-dlp/linux-x86_64/yt-dlp")
            );
            assert_eq!(
                plan.items[2].source_resource,
                Path::new(
                    "sidecars/ffmpeg/linux-x86_64/ffmpeg-n9.0.2-3-ga5923073bf-linux64-gpl-9.0.tar.xz"
                )
            );
        }

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            assert_eq!(plan.platform, "windows-x86_64");
            assert_eq!(
                plan.items[0].source_resource,
                Path::new("sidecars/arti/windows-x86_64/arti.exe")
            );
            assert_eq!(
                plan.items[1].source_resource,
                Path::new("sidecars/yt-dlp/windows-x86_64/yt-dlp.exe")
            );
            assert_eq!(
                plan.items[2].source_resource,
                Path::new(
                    "sidecars/ffmpeg/windows-x86_64/ffmpeg-n9.0.2-3-ga5923073bf-win64-gpl-9.0.zip"
                )
            );
        }

        assert!(!layout.root().exists());
    }

    #[test]
    fn bundled_sidecar_source_resolution_stays_beneath_backend_resource_root() {
        let root = test_root("sidecar-resource-resolution");
        let resource_root = root.join("package-resources");
        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("runtime layout is valid");
        let logical = build_bundled_sidecar_materialization_plan(&layout)
            .expect("supported platform produces logical sidecar plan");

        let resolved = resolve_bundled_sidecar_sources(&resource_root, logical)
            .expect("backend resource root resolves all sidecar sources");

        assert_eq!(resolved.items.len(), 3);
        assert_eq!(resolved.resource_root, resource_root);
        assert_eq!(resolved.bin_root, layout.bin_root().expect("bin root derives"));

        for item in &resolved.items {
            assert!(item.source.starts_with(&resolved.resource_root));
            assert!(item.destination.starts_with(&resolved.bin_root));
            assert_eq!(item.destination.parent(), Some(resolved.bin_root.as_path()));
            assert_ne!(item.source, item.destination);
            assert!(!item.identity.version.is_empty());
        }

        assert_eq!(resolved.items[0].kind, BundledSidecarKind::Arti);
        assert_eq!(resolved.items[1].kind, BundledSidecarKind::YtDlp);
        assert_eq!(resolved.items[2].kind, BundledSidecarKind::Ffmpeg);
        assert_eq!(resolved.items[0].identity.version, "2.6.0");
        assert_eq!(resolved.items[1].identity.version, "2026.08.19");
        assert_eq!(resolved.items[2].identity.version, "n9.0.2-3-ga5923073bf");
        assert!(!root.exists());
    }

    #[test]
    fn bundled_sidecar_source_resolution_rejects_traversing_resource_root() {
        let layout = AppRuntimeLayout::new("runtime").expect("runtime layout is valid");
        let logical = build_bundled_sidecar_materialization_plan(&layout)
            .expect("supported platform produces logical sidecar plan");

        let error = resolve_bundled_sidecar_sources(
            PathBuf::from("package-resources").join("..").join("escape"),
            logical,
        )
        .expect_err("resource root traversal must fail closed");

        assert_eq!(error.code, "package-resource-root-traversal");
    }

    fn source_validation_fixture_plan(
        root: &Path,
        hash_override: Option<&str>,
    ) -> ResolvedBundledSidecarMaterializationPlan {
        let resource_root = root.join("package-resources");
        fs::create_dir_all(&resource_root).expect("fixture resource root creation succeeds");

        let arti_source = resource_root.join("arti");
        let ytdlp_source = resource_root.join("yt-dlp");
        let ffmpeg_source = resource_root.join("ffmpeg-archive");
        fs::write(&arti_source, b"arti-fixture").expect("Arti fixture write succeeds");
        fs::write(&ytdlp_source, b"abc").expect("yt-dlp fixture write succeeds");
        fs::write(&ffmpeg_source, b"abc").expect("FFmpeg fixture write succeeds");

        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("fixture layout is valid");
        let digest = hash_override
            .unwrap_or("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
            .to_owned();

        ResolvedBundledSidecarMaterializationPlan {
            platform: "fixture",
            resource_root,
            bin_root: layout.bin_root().expect("fixture bin root derives"),
            items: vec![
                ResolvedBundledSidecarMaterializationItem {
                    kind: BundledSidecarKind::Arti,
                    source: arti_source,
                    destination: layout.arti_executable().expect("Arti destination derives"),
                    identity: BundledSidecarIdentity {
                        version: "2.6.0".to_owned(),
                        pinned_source_sha256: Some(
                            "ee35e504447b53206b28f1fa2874d4ac8ab94260a2dd3193bf0dd3819d638925"
                                .to_owned(),
                        ),
                    },
                },
                ResolvedBundledSidecarMaterializationItem {
                    kind: BundledSidecarKind::YtDlp,
                    source: ytdlp_source,
                    destination: layout.ytdlp_executable().expect("yt-dlp destination derives"),
                    identity: BundledSidecarIdentity {
                        version: "2026.08.19".to_owned(),
                        pinned_source_sha256: Some(digest.clone()),
                    },
                },
                ResolvedBundledSidecarMaterializationItem {
                    kind: BundledSidecarKind::Ffmpeg,
                    source: ffmpeg_source,
                    destination: layout.ffmpeg_executable().expect("FFmpeg destination derives"),
                    identity: BundledSidecarIdentity {
                        version: "n9.0.2-3-ga5923073bf".to_owned(),
                        pinned_source_sha256: Some(digest),
                    },
                },
            ],
        }
    }

    #[test]
    fn packaged_sidecar_source_validation_verifies_files_hashes_and_containment() {
        let root = test_root("sidecar-source-validation");
        let plan = source_validation_fixture_plan(&root, None);
        let runtime_root = root.join("runtime");

        let verified = validate_packaged_sidecar_sources(plan)
            .expect("regular contained sources with matching hashes validate");

        assert_eq!(verified.items.len(), 3);
        assert!(verified.resource_root.is_absolute() || verified.resource_root.exists());
        assert!(!verified.bin_root.exists());

        for item in &verified.items {
            assert!(item.source.starts_with(&verified.resource_root));
            assert!(item.source.is_file());
            assert!(item.destination.starts_with(&verified.bin_root));
            assert!(item.byte_size > 0);
        }

        assert_eq!(verified.items[0].kind, BundledSidecarKind::Arti);
        assert_eq!(verified.items[1].kind, BundledSidecarKind::YtDlp);
        assert_eq!(verified.items[2].kind, BundledSidecarKind::Ffmpeg);
        assert!(!runtime_root.exists());

        fs::remove_dir_all(root).expect("validation fixture cleanup succeeds");
    }

    #[test]
    fn packaged_sidecar_source_validation_fails_closed_on_hash_mismatch() {
        let root = test_root("sidecar-source-hash-mismatch");
        let plan = source_validation_fixture_plan(
            &root,
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        );

        let error = validate_packaged_sidecar_sources(plan)
            .expect_err("hash mismatch must fail before runtime destination creation");

        assert_eq!(error.code, "sidecar-source-hash-mismatch");
        assert!(!root.join("runtime").exists());

        fs::remove_dir_all(root).expect("hash mismatch fixture cleanup succeeds");
    }

    #[test]
    fn packaged_sidecar_source_validation_rejects_non_file_source() {
        let root = test_root("sidecar-source-not-file");
        let mut plan = source_validation_fixture_plan(&root, None);
        fs::remove_file(&plan.items[0].source).expect("fixture Arti file removal succeeds");
        fs::create_dir_all(&plan.items[0].source).expect("fixture Arti directory creation succeeds");

        let error = validate_packaged_sidecar_sources(plan)
            .expect_err("directory sidecar source must fail closed");

        assert_eq!(error.code, "sidecar-source-not-file");
        assert!(!root.join("runtime").exists());

        fs::remove_dir_all(root).expect("non-file fixture cleanup succeeds");
    }

    fn verified_ytdlp_prelaunch_fixture(
        root: &Path,
        ytdlp_count: usize,
    ) -> (AppRuntimeLayout, VerifiedBundledSidecarMaterializationPlan) {
        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("fixture runtime layout");
        let resource_root = root.join("resources");
        fs::create_dir_all(&resource_root).expect("fixture resource root");
        let mut items = Vec::new();

        for index in 0..ytdlp_count {
            let source = resource_root.join(format!("yt-dlp-{index}"));
            fs::write(&source, b"fixture").expect("fixture yt-dlp source");
            items.push(VerifiedBundledSidecarArtifact {
                kind: BundledSidecarKind::YtDlp,
                source,
                destination: layout.ytdlp_executable().expect("yt-dlp destination"),
                identity: BundledSidecarIdentity {
                    version: "fixture".to_owned(),
                    pinned_source_sha256: Some(
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_owned(),
                    ),
                },
                byte_size: 7,
            });
        }

        (
            layout.clone(),
            VerifiedBundledSidecarMaterializationPlan {
                platform: "fixture",
                resource_root,
                bin_root: layout.bin_root().expect("fixture bin root"),
                items,
            },
        )
    }

    #[test]
    fn ytdlp_prelaunch_prepares_runtime_before_materialization_and_binds_expected_path() {
        let root = test_root("ytdlp-prelaunch-order");
        let (layout, plan) = verified_ytdlp_prelaunch_fixture(&root, 1);
        let expected = layout.ytdlp_executable().expect("expected yt-dlp path");
        let expected_for_callback = expected.clone();

        let (verified_layout, materialized) = prepare_materialized_ytdlp_prelaunch(
            layout,
            &plan,
            move |prepared, resource_root, artifact| {
                assert!(prepared.layout().root().is_dir());
                assert!(prepared.layout().tor_cache_dir()?.is_dir());
                assert!(prepared.layout().tor_state_dir()?.is_dir());
                assert!(prepared.layout().output_root()?.is_dir());
                assert!(resource_root.is_dir());
                assert_eq!(artifact.destination, expected_for_callback);
                assert!(!artifact.destination.exists());
                Ok(artifact.destination.clone())
            },
        )
        .expect("prelaunch preparation succeeds");

        assert_eq!(materialized, expected);
        assert_eq!(verified_layout.ytdlp_executable().unwrap(), expected);
        assert!(!expected.exists());

        fs::remove_dir_all(root).expect("prelaunch order fixture cleanup");
    }

    #[test]
    fn ytdlp_prelaunch_fails_closed_when_verified_artifact_is_missing_before_runtime_mutation() {
        let root = test_root("ytdlp-prelaunch-missing");
        let (layout, plan) = verified_ytdlp_prelaunch_fixture(&root, 0);
        let runtime_root = layout.root().to_path_buf();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_in_callback = called.clone();

        let error = prepare_materialized_ytdlp_prelaunch(
            layout,
            &plan,
            move |_, _, _| {
                called_in_callback.store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("missing artifact must fail before materializer")
            },
        )
        .expect_err("missing yt-dlp artifact must fail closed");

        assert_eq!(error.code, "verified-ytdlp-artifact-missing");
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!runtime_root.exists());

        fs::remove_dir_all(root).expect("missing-artifact fixture cleanup");
    }

    #[test]
    fn ytdlp_prelaunch_fails_closed_on_duplicate_verified_artifacts_before_runtime_mutation() {
        let root = test_root("ytdlp-prelaunch-duplicate");
        let (layout, plan) = verified_ytdlp_prelaunch_fixture(&root, 2);
        let runtime_root = layout.root().to_path_buf();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_in_callback = called.clone();

        let error = prepare_materialized_ytdlp_prelaunch(
            layout,
            &plan,
            move |_, _, _| {
                called_in_callback.store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("duplicate artifacts must fail before materializer")
            },
        )
        .expect_err("duplicate yt-dlp artifacts must fail closed");

        assert_eq!(error.code, "verified-ytdlp-artifact-duplicate");
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!runtime_root.exists());

        fs::remove_dir_all(root).expect("duplicate-artifact fixture cleanup");
    }

    #[test]
    fn ytdlp_prelaunch_rejects_materializer_returning_non_layout_path() {
        let root = test_root("ytdlp-prelaunch-path-mismatch");
        let (layout, plan) = verified_ytdlp_prelaunch_fixture(&root, 1);
        let wrong = root.join("wrong-yt-dlp");

        let error = prepare_materialized_ytdlp_prelaunch(
            layout,
            &plan,
            |_, _, _| Ok(wrong.clone()),
        )
        .expect_err("wrong materialized destination must fail closed");

        assert_eq!(error.code, "materialized-ytdlp-path-mismatch");
        assert!(!wrong.exists());

        fs::remove_dir_all(root).expect("path-mismatch fixture cleanup");
    }

    fn verified_arti_prelaunch_fixture(
        root: &Path,
        arti_count: usize,
    ) -> (AppRuntimeLayout, VerifiedBundledSidecarMaterializationPlan) {
        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("fixture runtime layout");
        let resource_root = root.join("resources");
        fs::create_dir_all(&resource_root).expect("fixture resource root");
        let mut items = Vec::new();

        for index in 0..arti_count {
            let source = resource_root.join(format!("arti-{index}"));
            fs::write(&source, b"fixture").expect("fixture Arti source");
            items.push(VerifiedBundledSidecarArtifact {
                kind: BundledSidecarKind::Arti,
                source,
                destination: layout.arti_executable().expect("Arti destination"),
                identity: BundledSidecarIdentity {
                    version: "fixture".to_owned(),
                    pinned_source_sha256: Some(
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_owned(),
                    ),
                },
                byte_size: 7,
            });
        }

        (
            layout.clone(),
            VerifiedBundledSidecarMaterializationPlan {
                platform: "fixture",
                resource_root,
                bin_root: layout.bin_root().expect("fixture bin root"),
                items,
            },
        )
    }

    #[test]
    fn arti_prelaunch_prepares_runtime_before_materialization_and_binds_expected_path() {
        let root = test_root("arti-prelaunch-order");
        let (layout, plan) = verified_arti_prelaunch_fixture(&root, 1);
        let expected = layout.arti_executable().expect("expected Arti path");
        let expected_for_callback = expected.clone();

        let (verified_layout, materialized) = prepare_materialized_arti_prelaunch(
            layout,
            &plan,
            move |prepared, resource_root, artifact| {
                assert!(prepared.layout().root().is_dir());
                assert!(prepared.layout().tor_cache_dir()?.is_dir());
                assert!(prepared.layout().tor_state_dir()?.is_dir());
                assert!(prepared.layout().output_root()?.is_dir());
                assert!(resource_root.is_dir());
                assert_eq!(artifact.destination, expected_for_callback);
                assert!(!artifact.destination.exists());
                Ok(artifact.destination.clone())
            },
        )
        .expect("Arti prelaunch preparation succeeds");

        assert_eq!(materialized, expected);
        assert_eq!(verified_layout.arti_executable().unwrap(), expected);
        assert!(!expected.exists());

        fs::remove_dir_all(root).expect("Arti prelaunch order fixture cleanup");
    }

    #[test]
    fn arti_prelaunch_fails_closed_when_verified_artifact_is_missing_before_materializer() {
        let root = test_root("arti-prelaunch-missing");
        let (layout, plan) = verified_arti_prelaunch_fixture(&root, 0);
        let runtime_root = layout.root().to_path_buf();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_in_callback = called.clone();

        let error = prepare_materialized_arti_prelaunch(
            layout,
            &plan,
            move |_, _, _| {
                called_in_callback.store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("missing Arti artifact must fail before materializer")
            },
        )
        .expect_err("missing Arti artifact must fail closed");

        assert_eq!(error.code, "verified-arti-artifact-missing");
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!runtime_root.exists());

        fs::remove_dir_all(root).expect("missing Arti artifact fixture cleanup");
    }

    #[test]
    fn arti_prelaunch_fails_closed_on_duplicate_verified_artifacts_before_materializer() {
        let root = test_root("arti-prelaunch-duplicate");
        let (layout, plan) = verified_arti_prelaunch_fixture(&root, 2);
        let runtime_root = layout.root().to_path_buf();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_in_callback = called.clone();

        let error = prepare_materialized_arti_prelaunch(
            layout,
            &plan,
            move |_, _, _| {
                called_in_callback.store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("duplicate Arti artifacts must fail before materializer")
            },
        )
        .expect_err("duplicate Arti artifacts must fail closed");

        assert_eq!(error.code, "verified-arti-artifact-duplicate");
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!runtime_root.exists());

        fs::remove_dir_all(root).expect("duplicate Arti artifact fixture cleanup");
    }

    #[test]
    fn arti_prelaunch_rejects_materializer_returning_non_layout_path() {
        let root = test_root("arti-prelaunch-path-mismatch");
        let (layout, plan) = verified_arti_prelaunch_fixture(&root, 1);
        let wrong = root.join("wrong-arti");

        let error = prepare_materialized_arti_prelaunch(
            layout,
            &plan,
            |_, _, _| Ok(wrong.clone()),
        )
        .expect_err("wrong Arti materialized destination must fail closed");

        assert_eq!(error.code, "materialized-arti-path-mismatch");
        assert!(!wrong.exists());

        fs::remove_dir_all(root).expect("Arti path-mismatch fixture cleanup");
    }

    fn verified_completed_file_sidecars_fixture(
        root: &Path,
    ) -> (
        AppRuntimeLayout,
        VerifiedBundledSidecarMaterializationPlan,
    ) {
        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("fixture runtime layout");
        let resource_root = root.join("resources");
        fs::create_dir_all(&resource_root).expect("fixture resource root");

        let ytdlp_source = resource_root.join("yt-dlp");
        let arti_source = resource_root.join("arti");
        fs::write(&ytdlp_source, b"ytdlp").expect("fixture yt-dlp source");
        fs::write(&arti_source, b"arti").expect("fixture Arti source");

        let items = vec![
            VerifiedBundledSidecarArtifact {
                kind: BundledSidecarKind::YtDlp,
                source: ytdlp_source,
                destination: layout.ytdlp_executable().expect("yt-dlp destination"),
                identity: BundledSidecarIdentity {
                    version: "fixture".to_owned(),
                    pinned_source_sha256: Some(
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_owned(),
                    ),
                },
                byte_size: 5,
            },
            VerifiedBundledSidecarArtifact {
                kind: BundledSidecarKind::Arti,
                source: arti_source,
                destination: layout.arti_executable().expect("Arti destination"),
                identity: BundledSidecarIdentity {
                    version: "fixture".to_owned(),
                    pinned_source_sha256: Some(
                        "1111111111111111111111111111111111111111111111111111111111111111"
                            .to_owned(),
                    ),
                },
                byte_size: 4,
            },
        ];

        (
            layout.clone(),
            VerifiedBundledSidecarMaterializationPlan {
                platform: "fixture",
                resource_root,
                bin_root: layout.bin_root().expect("fixture bin root"),
                items,
            },
        )
    }

    #[test]
    fn completed_file_prelaunch_orders_ytdlp_then_arti_then_pipeline() {
        let root = test_root("completed-file-sidecar-order");
        let (layout, plan) = verified_completed_file_sidecars_fixture(&root);
        let (candidate, inputs) = prepare_completed_file_command(
            "find the official live performance".to_owned(),
            T024_PROOF_LOCATOR.to_owned(),
            &layout,
        )
        .expect("completed-file fixture inputs");

        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let ytdlp_events = events.clone();
        let arti_events = events.clone();
        let pipeline_events = events.clone();

        let result = run_completed_file_with_materialized_sidecars(
            layout,
            &plan,
            &candidate,
            inputs,
            move |prepared, _, artifact| {
                ytdlp_events.lock().unwrap().push("yt-dlp");
                assert!(prepared.layout().root().is_dir());
                assert_eq!(artifact.kind, BundledSidecarKind::YtDlp);
                Ok(artifact.destination.clone())
            },
            move |prepared, _, artifact| {
                arti_events.lock().unwrap().push("arti");
                assert!(prepared.layout().root().is_dir());
                assert_eq!(artifact.kind, BundledSidecarKind::Arti);
                Ok(artifact.destination.clone())
            },
            move |_, _| {
                pipeline_events.lock().unwrap().push("pipeline");
                Ok(CompletedFileView {
                    file_name: "fixture.mp4".to_owned(),
                    byte_size: 1,
                    stage: "completed-file-ready",
                })
            },
        )
        .expect("sidecar prelaunch orchestration succeeds");

        assert_eq!(result.file_name, "fixture.mp4");
        assert_eq!(
            *events.lock().unwrap(),
            vec!["yt-dlp", "arti", "pipeline"]
        );

        fs::remove_dir_all(root).expect("completed-file order fixture cleanup");
    }

    #[test]
    fn completed_file_prelaunch_arti_failure_blocks_pipeline() {
        let root = test_root("completed-file-arti-failure");
        let (layout, plan) = verified_completed_file_sidecars_fixture(&root);
        let (candidate, inputs) = prepare_completed_file_command(
            "find the official live performance".to_owned(),
            T024_PROOF_LOCATOR.to_owned(),
            &layout,
        )
        .expect("completed-file fixture inputs");

        let pipeline_called =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pipeline_called_in_closure = pipeline_called.clone();

        let error = run_completed_file_with_materialized_sidecars(
            layout,
            &plan,
            &candidate,
            inputs,
            |_, _, artifact| Ok(artifact.destination.clone()),
            |_, _, _| {
                Err(DownloadActionError {
                    code: "fixture-arti-materialization-failed",
                    message: "fixture Arti materialization failure".to_owned(),
                })
            },
            move |_, _| {
                pipeline_called_in_closure
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                unreachable!("pipeline must not run after Arti materialization failure")
            },
        )
        .expect_err("Arti materialization failure must block completed-file pipeline");

        assert_eq!(error.code, "fixture-arti-materialization-failed");
        assert!(!pipeline_called.load(std::sync::atomic::Ordering::SeqCst));

        fs::remove_dir_all(root).expect("completed-file failure fixture cleanup");
    }

    #[test]
    fn app_runtime_directory_preparation_is_idempotent_and_creates_only_required_directories() {
        let root = test_root("runtime-directories");
        let layout = AppRuntimeLayout::new(&root).expect("test runtime root is valid");

        let first = prepare_app_runtime_directories(layout.clone())
            .expect("first runtime directory preparation succeeds");
        let second = prepare_app_runtime_directories(first.layout().clone())
            .expect("second runtime directory preparation is idempotent");

        let config_parent = root.join("arti/config");
        let cache = root.join("arti/cache");
        let state = root.join("arti/state");
        let downloads = root.join("downloads");

        for directory in [&root, &config_parent, &cache, &state, &downloads] {
            assert!(directory.is_dir(), "required directory exists: {}", directory.display());
            assert!(directory.starts_with(&root));
        }

        assert_eq!(second.layout().root(), root.as_path());
        assert!(!root.join("bin").exists());
        assert!(!root.join("bin/arti").exists());
        assert!(!root.join("bin/yt-dlp").exists());
        assert!(!root.join("bin/ffmpeg").exists());
        assert!(!root.join("arti/config/pulqva.toml").exists());

        fs::remove_dir_all(root).expect("runtime directory test cleanup succeeds");
    }

    #[test]
    fn app_runtime_directory_preparation_fails_closed_on_non_directory_parent() {
        let root = test_root("runtime-directories-blocked");
        fs::create_dir_all(&root).expect("test runtime root creation succeeds");
        fs::write(root.join("arti"), b"blocking file").expect("blocking file creation succeeds");
        let layout = AppRuntimeLayout::new(&root).expect("test runtime root is valid");

        let error = prepare_app_runtime_directories(layout)
            .expect_err("non-directory Tor parent must fail closed");

        assert_eq!(error.code, "runtime-directory-not-directory");
        assert!(!root.join("downloads").exists());
        assert!(!root.join("bin").exists());

        fs::remove_dir_all(root).expect("blocked runtime directory test cleanup succeeds");
    }

    #[test]
    fn completed_file_command_preflight_fails_before_runtime_for_invalid_input() {
        let layout = AppRuntimeLayout::new("runtime").expect("test runtime root is valid");
        let blank = prepare_completed_file_command(
            "  \t ".to_owned(),
            T024_PROOF_LOCATOR.to_owned(),
            &layout,
        )
        .expect_err("blank intent must fail before runtime work");
        assert_eq!(blank.code, "invalid-search-intent");

        let unknown = prepare_completed_file_command(
            "find it".to_owned(),
            "local:test:candidate:not-returned".to_owned(),
            &layout,
        )
        .expect_err("unknown locator must fail before runtime work");
        assert_eq!(unknown.code, "candidate-not-found");

        let unsupported = prepare_completed_file_command(
            "find archive".to_owned(),
            "local:test:candidate:archive-performance".to_owned(),
            &layout,
        )
        .expect_err("unsupported candidate must fail before runtime work");
        assert_eq!(unsupported.code, "media-source-unsupported");
    }

    #[test]
    fn download_action_revalidates_candidate_and_returns_data_only_plan() {
        let action = plan_local_download(
            "find the official live performance".to_owned(),
            "local:test:candidate:official-live".to_owned(),
        )
        .expect("known validated candidate can produce a download action");

        assert_eq!(action.intent_query, "find the official live performance");
        assert_eq!(action.title, "Official live performance");
        assert_eq!(action.locator, "local:test:candidate:official-live");
        assert_eq!(action.action, "download");
        assert_eq!(action.stage, "download-action-planned");
        assert!(action.media_source_ready);
        assert_eq!(action.media_source_stage, "backend-media-source-resolved");
    }

    #[test]
    fn backend_media_resolution_constructs_the_typed_immutable_t024_source() {
        let candidate = local_candidate_by_locator(T024_PROOF_LOCATOR)
            .expect("local proof candidates are valid")
            .expect("T024 proof candidate exists");
        let source = resolve_backend_media_source(&candidate)
            .expect("supported proof candidate resolves to typed media source");

        assert_eq!(source.as_str(), T024_MEDIA_SOURCE_URL);
    }

    #[test]
    fn download_action_rejects_validated_but_unsupported_locator_fail_closed() {
        let error = plan_local_download(
            "find an archive performance".to_owned(),
            "local:test:candidate:archive-performance".to_owned(),
        )
        .expect_err("candidate without an approved media mapping must fail closed");

        assert_eq!(error.code, "media-source-unsupported");
        assert_eq!(
            error.message,
            "validated candidate has no approved backend media-source mapping"
        );
    }

    #[test]
    fn download_action_rejects_unknown_locator_fail_closed() {
        let error = plan_local_download(
            "find the official live performance".to_owned(),
            "https://example.invalid/not-a-validated-candidate".to_owned(),
        )
        .expect_err("arbitrary locator must not cross the Download boundary");

        assert_eq!(error.code, "candidate-not-found");
        assert_eq!(
            error.message,
            "candidate locator is not present in the validated local candidate set"
        );
    }

    #[test]
    fn download_action_rejects_blank_intent_before_candidate_lookup() {
        let error = plan_local_download(
            " \n ".to_owned(),
            "local:test:candidate:official-live".to_owned(),
        )
        .expect_err("blank intent must fail before planning Download");

        assert_eq!(error.code, "invalid-search-intent");
        assert_eq!(error.message, "search query must not be empty");
    }
}
