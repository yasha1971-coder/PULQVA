use pulqva_core::{CORE_CRATE_READY, SearchIntent, SearchIntentError};
use serde::Serialize;

const PRODUCT_NAME: &str = "PULQVA";
const KERNEL_VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../kernel/KERNEL_VERSION"));

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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_status, submit_intent])
        .run(tauri::generate_context!())
        .expect("PULQVA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{app_status, submit_intent};

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
}
