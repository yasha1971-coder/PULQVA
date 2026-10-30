# Native Linux GUI WebDriver boundary
Verified framework: apps/pulqva-desktop/src-tauri/Cargo.toml pins Tauri2.11.6; main.rs registers prototype_search/select/download; prototype.html is the real packaged entry.
Choose official tauri-driver2.0.6 + Linux WebKitWebDriver, driven through standard W3C HTTP by the existing Python harness. No application plugin, IPC mocks, new user runtime or architectural change.
Resolved tauri-runtime-wry2.11.4 source explicitly enables native automation with TAURI_WEBVIEW_AUTOMATION=true. Keep it test-only in the isolated runner environment.
This first smoke checks a real displayed query field, native WebDriver text input/readback and PNG screenshot, then owned descendant cleanup under accepted independent guardian. It deliberately does NOT submit a query or claim results/choice/Tor/file acceptance. Do not report this limited PASS as the requested full GUI SMOKE.
Next separate phase extends the same driver to actual submit, >=2 real Commons results, explicit GUI choice, allowed file<=8MiB, independent saved SHA and TCP/UDP/process cleanup. No mocked final PASS. If hosted networking fails, retain exact Tor logs and reuse this same native driver locally without Computer Use; no security-policy bypass.
Human acceptance on the laptop is requested only AFTER automated scenario and repeat are evidenced; the owner's checklist is acceptance criteria, not current results.
Official references: https://tauri.app/develop/tests/webdriver/manual-setup/ and https://tauri.app/develop/tests/webdriver/ci/ .
