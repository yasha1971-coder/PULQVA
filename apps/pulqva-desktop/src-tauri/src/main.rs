use pulqva_core::CORE_CRATE_READY;
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_status])
        .run(tauri::generate_context!())
        .expect("PULQVA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::app_status;

    #[test]
    fn typed_status_contract_reports_linked_core_and_kernel() {
        let status = app_status();

        assert_eq!(status.product_name, "PULQVA");
        assert_eq!(status.app_version, "0.0.0");
        assert_eq!(status.kernel_version, "1.0.0");
        assert!(status.core_ready);
        assert_eq!(status.privacy_mode, "tor-required-fail-closed");
    }
}
