use pulqva_core::{
    CORE_CRATE_READY, SearchCandidate, SearchCandidateError, SearchIntent, SearchIntentError,
};
use pulqva_privacy::{
    ArtiConfigMaterializeError, ArtiRuntimePlan, PreparedArtiRuntime, TorSocksEndpoint,
    TorSocksEndpointError, YtDlpMediaSourceError, YtDlpMediaSourceUrl, prepare_arti_runtime,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

const PRODUCT_NAME: &str = "PULQVA";
const KERNEL_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../kernel/KERNEL_VERSION"));
const T024_MEDIA_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/4d21a042353022326071acb0251ab75cd6bae114/big_buck_bunny.mp4";
const T024_PROOF_LOCATOR: &str = "local:test:candidate:official-live";
const DEFAULT_TOR_SOCKS_PORT: u16 = 19050;

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
    ytdlp_executable: PathBuf,
    media_source: YtDlpMediaSourceUrl,
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

fn default_download_preflight_inputs() -> Result<DownloadPreflightInputs, DownloadActionError> {
    DownloadPreflightInputs::new(
        "runtime/arti",
        "runtime/yt-dlp",
        "runtime/arti/config/pulqva.toml",
        "runtime/arti/cache",
        "runtime/arti/state",
        "downloads",
        DEFAULT_TOR_SOCKS_PORT,
    )
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
    let _preflight = build_download_preflight(&candidate, default_download_preflight_inputs()?)?;

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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_status,
            submit_intent,
            list_local_candidates,
            select_local_candidate,
            plan_local_download
        ])
        .run(tauri::generate_context!())
        .expect("PULQVA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_TOR_SOCKS_PORT, DownloadPreflightInputs, T024_MEDIA_SOURCE_URL,
        T024_PROOF_LOCATOR, app_status, build_download_preflight, list_local_candidates,
        local_candidate_by_locator, plan_local_download, prepare_download_runtime,
        resolve_backend_media_source, select_local_candidate, submit_intent,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
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
            "bundle/yt-dlp",
            "state/arti/pulqva.toml",
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
