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

/**
 * @typedef {Object} DesktopCandidate
 * @property {string} title
 * @property {string} locator
 */

/**
 * @typedef {Object} CandidateList
 * @property {string} intentQuery
 * @property {string} stage
 * @property {DesktopCandidate[]} candidates
 */

/**
 * @typedef {Object} CandidateSelection
 * @property {string} intentQuery
 * @property {string} title
 * @property {string} locator
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

/**
 * @param {string} query
 * @returns {Promise<CandidateList>}
 */
async function invokeLocalCandidates(query) {
  return invoke("list_local_candidates", { query });
}

/**
 * Selection sends only validated intent text plus the opaque locator.
 * The locator remains inert data in the frontend and Rust backend.
 * @param {string} query
 * @param {string} locator
 * @returns {Promise<CandidateSelection>}
 */
async function invokeSelectCandidate(query, locator) {
  return invoke("select_local_candidate", { query, locator });
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

/**
 * @param {CandidateSelection} selection
 */
function renderSelection(selection) {
  const panel = document.querySelector("#selection-panel");
  document.querySelector("#selection-title-value").textContent = selection.title;
  document.querySelector("#selection-locator-value").textContent = selection.locator;
  document.querySelector("#selection-stage-value").textContent = selection.stage;
  panel.hidden = false;
}

/**
 * Render candidate fields and explicit select actions only.
 * The opaque locator is never interpreted, navigated, or executed.
 * @param {CandidateList} response
 */
function renderCandidates(response) {
  const panel = document.querySelector("#candidate-panel");
  const list = document.querySelector("#candidate-list");
  const selectionPanel = document.querySelector("#selection-panel");
  const state = document.querySelector("#intent-state");

  list.replaceChildren();
  selectionPanel.hidden = true;

  for (const candidate of response.candidates) {
    const item = document.createElement("li");
    item.className = "candidate-item";

    const copy = document.createElement("div");
    copy.className = "candidate-copy";

    const title = document.createElement("strong");
    title.className = "candidate-name";
    title.textContent = candidate.title;

    const locator = document.createElement("code");
    locator.className = "candidate-locator";
    locator.textContent = candidate.locator;

    const select = document.createElement("button");
    select.type = "button";
    select.className = "candidate-select";
    select.textContent = "Select";
    select.addEventListener("click", async () => {
      select.disabled = true;
      state.dataset.kind = "pending";
      state.textContent = "Validating candidate selection locally…";

      try {
        const selection = await invokeSelectCandidate(
          response.intentQuery,
          candidate.locator,
        );

        renderSelection(selection);
        state.dataset.kind = "success";
        state.textContent = "Candidate selected locally. Locator remains inert.";
      } catch (error) {
        const message =
          error && typeof error === "object" && "message" in error
            ? String(error.message)
            : "Candidate selection was rejected by the local core.";

        state.dataset.kind = "error";
        state.textContent = message;
      } finally {
        select.disabled = false;
      }
    });

    copy.append(title, locator);
    item.append(copy, select);
    list.append(item);
  }

  panel.hidden = false;
}

function bindIntentForm() {
  const form = document.querySelector("#intent-form");
  const query = document.querySelector("#intent-query");
  const button = document.querySelector("#intent-submit");
  const state = document.querySelector("#intent-state");
  const result = document.querySelector("#intent-result");
  const resultQuery = document.querySelector("#intent-result-query");
  const resultStage = document.querySelector("#intent-result-stage");
  const candidatePanel = document.querySelector("#candidate-panel");
  const selectionPanel = document.querySelector("#selection-panel");

  form.addEventListener("submit", async (event) => {
    event.preventDefault();

    button.disabled = true;
    state.dataset.kind = "pending";
    state.textContent = "Validating in local Rust core…";
    result.hidden = true;
    candidatePanel.hidden = true;
    selectionPanel.hidden = true;

    try {
      const submission = await invokeSubmitIntent(query.value);
      const candidates = await invokeLocalCandidates(submission.query);

      resultQuery.textContent = submission.query;
      resultStage.textContent = submission.stage;
      result.hidden = false;
      renderCandidates(candidates);

      state.dataset.kind = "success";
      state.textContent = `Intent accepted. ${candidates.candidates.length} local candidates ready.`;
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
