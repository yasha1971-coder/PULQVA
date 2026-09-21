/**
 * @typedef {Object} AppStatus
 * @property {string} productName
 * @property {string} appVersion
 * @property {string} kernelVersion
 * @property {boolean} coreReady
 * @property {string} privacyMode
 */

/**
 * The frontend boundary is intentionally limited to one typed Tauri command.
 * @returns {Promise<AppStatus>}
 */
async function invokeAppStatus() {
  return window.__TAURI__.core.invoke("app_status");
}

async function renderStatus() {
  const title = document.querySelector("#status-title");
  const dot = document.querySelector("#status-dot");

  try {
    const status = await invokeAppStatus();

    document.querySelector("#app-version").textContent = status.appVersion;
    document.querySelector("#kernel-version").textContent = status.kernelVersion;
    document.querySelector("#privacy-mode").textContent = status.privacyMode;
    document.querySelector("#core-ready").textContent = status.coreReady ? "ready" : "not ready";

    title.textContent = status.coreReady
      ? "Local Rust core ready"
      : "Local Rust core unavailable";
    dot.dataset.ready = String(status.coreReady);
  } catch (_error) {
    title.textContent = "Local command boundary unavailable";
    dot.dataset.ready = "false";
  }
}

renderStatus();
