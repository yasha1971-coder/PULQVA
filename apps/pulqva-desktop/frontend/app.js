/**
 * @typedef {Object} AppStatus
 * @property {string} productName
 * @property {string} appVersion
 * @property {string} kernelVersion
 * @property {boolean} coreReady
 * @property {string} privacyMode
 */

/**
 * @typedef {Object} IntentSubmission
 * @property {string} query
 * @property {string} stage
 */

const invoke = window.__TAURI__.core.invoke;

/**
 * @returns {Promise<AppStatus>}
 */
async function invokeAppStatus() {
  return invoke("app_status");
}

/**
 * Raw user text crosses only the typed Tauri command boundary.
 * @param {string} query
 * @returns {Promise<IntentSubmission>}
 */
async function invokeSubmitIntent(query) {
  return invoke("submit_intent", { query });
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

function bindIntentForm() {
  const form = document.querySelector("#intent-form");
  const query = document.querySelector("#intent-query");
  const button = document.querySelector("#intent-submit");
  const state = document.querySelector("#intent-state");
  const result = document.querySelector("#intent-result");
  const resultQuery = document.querySelector("#intent-result-query");
  const resultStage = document.querySelector("#intent-result-stage");

  form.addEventListener("submit", async (event) => {
    event.preventDefault();

    button.disabled = true;
    state.dataset.kind = "pending";
    state.textContent = "Validating in local Rust core…";
    result.hidden = true;

    try {
      const submission = await invokeSubmitIntent(query.value);

      resultQuery.textContent = submission.query;
      resultStage.textContent = submission.stage;
      result.hidden = false;
      state.dataset.kind = "success";
      state.textContent = "Intent accepted locally.";
    } catch (error) {
      const message =
        error && typeof error === "object" && "message" in error
          ? String(error.message)
          : "Intent rejected by the local core.";

      state.dataset.kind = "error";
      state.textContent = message;
    } finally {
      button.disabled = false;
    }
  });
}

renderStatus();
bindIntentForm();
