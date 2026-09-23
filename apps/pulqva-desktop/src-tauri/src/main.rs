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
use tauri::Manager;
use std::{
    ffi::OsString,
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

    fn arti_executable(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("arti")
    }

    fn ytdlp_executable(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("yt-dlp")
    }

    fn ffmpeg_executable(&self) -> Result<PathBuf, DownloadActionError> {
        self.derive("ffmpeg")
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
    let (candidate, inputs) = prepare_completed_file_command(query, locator, &layout)?;

    tauri::async_runtime::spawn_blocking(move || run_completed_file_pipeline(&candidate, inputs))
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
        AppRuntimeLayout, DEFAULT_TOR_READY_TIMEOUT_SECS, DEFAULT_TOR_SOCKS_PORT,
        DownloadPreflightInputs, T024_MEDIA_SOURCE_URL, T024_PROOF_LOCATOR, app_status,
        build_download_preflight, establish_tor_ready_download_runtime, list_local_candidates,
        local_candidate_by_locator, plan_local_download, prepare_completed_file_command,
        prepare_download_runtime, resolve_backend_media_source, runtime_layout_from_app_data_dir,
        select_local_candidate, submit_intent,
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

    #[test]
    fn app_runtime_layout_derives_every_backend_path_beneath_one_root() {
        let layout = AppRuntimeLayout::new("runtime").expect("runtime root is valid");

        assert_eq!(layout.root(), Path::new("runtime"));
        assert_eq!(layout.arti_executable().unwrap(), Path::new("runtime/arti"));
        assert_eq!(layout.ytdlp_executable().unwrap(), Path::new("runtime/yt-dlp"));
        assert_eq!(layout.ffmpeg_executable().unwrap(), Path::new("runtime/ffmpeg"));
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
        assert_eq!(inputs.download.arti_executable, root.join("arti"));
        assert_eq!(inputs.download.ytdlp_executable, root.join("yt-dlp"));
        assert_eq!(
            inputs.download.tor_config_file,
            root.join("arti/config/pulqva.toml")
        );
        assert_eq!(inputs.download.tor_cache_dir, root.join("arti/cache"));
        assert_eq!(inputs.download.tor_state_dir, root.join("arti/state"));
        assert_eq!(inputs.download.output_root, root.join("downloads"));
        assert_eq!(inputs.ffmpeg_executable, root.join("ffmpeg"));
        assert_eq!(
            inputs.tor_ready_timeout,
            Duration::from_secs(DEFAULT_TOR_READY_TIMEOUT_SECS)
        );
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