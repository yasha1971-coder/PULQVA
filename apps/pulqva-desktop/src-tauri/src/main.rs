use pulqva_core::{
    CORE_CRATE_READY, SearchCandidate, SearchCandidateError, SearchIntent, SearchIntentError,
};
use pulqva_privacy::{YtDlpMediaSourceError, YtDlpMediaSourceUrl};
use serde::Serialize;

const PRODUCT_NAME: &str = "PULQVA";
const KERNEL_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../kernel/KERNEL_VERSION"));
const T024_MEDIA_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/4d21a042353022326071acb0251ab75cd6bae114/big_buck_bunny.mp4";
const T024_PROOF_LOCATOR: &str = "local:test:candidate:official-live";

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
    let _source = resolve_backend_media_source(&candidate)?;

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
        T024_MEDIA_SOURCE_URL, T024_PROOF_LOCATOR, app_status, list_local_candidates,
        local_candidate_by_locator, plan_local_download, resolve_backend_media_source,
        select_local_candidate, submit_intent,
    };

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
