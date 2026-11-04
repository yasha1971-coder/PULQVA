/* All network and file operations belong to Rust IPC. No frontend Internet API. */
(() => {
  "use strict";
  const invoke = window.__TAURI__.core.invoke;
  const el = id => document.querySelector(`#${id}`);
  let busy = false;
  let current = null;
  let selected = null;
  let generation = 0;
  let interpretationGeneration = 0;
  let activeInterpretation = null;
  let verifiedInterpretation = null;
  const status = text => { el("prototype-status").textContent = text; };
  const setBusy = value => {
    busy = value;
    el("prototype-search").disabled = value;
    el("prototype-interpret").disabled = value;
    el("prototype-search-intent").disabled = value || !verifiedTermsCurrent();
    el("prototype-query").disabled = value;
    el("prototype-clear").disabled = value;
    el("prototype-download").disabled = value || selected === null;
    for (const button of el("prototype-choices").querySelectorAll("button")) button.disabled = value;
  };
  const message = err => err && typeof err.message === "string" ? err.message : "The operation failed. No direct connection is used.";
  const formatBytes = bytes => bytes < 1024 ? `${bytes} bytes` : bytes < 1048576 ? `${(bytes / 1024).toFixed(1)} KiB` : `${(bytes / 1048576).toFixed(2)} MiB`;
  function clearSelectionStyle() {
    for (const row of el("prototype-choices").children) {
      row.className = "candidate-item";
      for (const button of row.querySelectorAll("button")) button.textContent = "Select";
    }
  }
  function reset() {
    invalidateInterpretation();
    current = null; selected = null;
    el("prototype-choices").replaceChildren();
    el("prototype-selection").hidden = true;
    el("prototype-file").hidden = true;
    el("prototype-download").disabled = true;
  }
  const aiStatus = text => { el("prototype-ai-status").textContent = text; };
  function verifiedTermsCurrent() {
    const value = verifiedInterpretation;
    return value !== null && value.ticket === interpretationGeneration && value.searchGeneration === generation &&
      value.sourceQuery === el("prototype-query").value && !el("prototype-interpretation").hidden &&
      el("prototype-interpretation-query").textContent === value.query &&
      el("prototype-interpretation-mode").textContent === value.choiceMode;
  }
  function hideInterpretation() {
    verifiedInterpretation = null;
    el("prototype-search-intent").disabled = true;
    el("prototype-interpretation").hidden = true;
    el("prototype-interpretation-query").textContent = "";
    el("prototype-interpretation-mode").textContent = "";
  }
  function invalidateInterpretation() {
    ++interpretationGeneration;
    hideInterpretation();
    aiStatus("Interpretation is separate from keyword search.");
  }
  const aiErrors = {
    unavailable: "Local AI is unavailable. Start the local Qwen server and try again.",
    timeout: "Local AI timed out. Try again when the server is ready.",
    invalid_response: "The local model response failed validation. No search was started.",
    invalid_request: "Please enter a valid request for local interpretation.",
    busy: "A local interpretation is already running. Please wait.",
    worker_failed: "The local interpretation worker failed. Please try again."
  };
  function validInterpretation(value) {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const keys = Object.keys(value).sort().join(",");
    if (value.kind === "reject") return keys === "kind,reason" && value.reason === "semantic_authority";
    return value.kind === "intent" && keys === "choice_mode,kind,query" &&
      typeof value.query === "string" && value.query.trim().length > 0 &&
      new TextEncoder().encode(value.query).length <= 512 &&
      (value.choice_mode === "ask" || value.choice_mode === "autopilot");
  }
  const validToken = value => typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
  function validSearch(value) {
    if (!value || !validToken(value.sessionId) || !Array.isArray(value.candidates) ||
        value.candidates.length < 2 || value.candidates.length > 10) return false;
    return value.candidates.every((c, i) => c && c.index === i && typeof c.title === "string" &&
      c.title.length > 0 && Number.isSafeInteger(c.byteSize) && c.byteSize > 0 && c.byteSize <= 8388608);
  }
  function render(response, epoch) {
    for (const candidate of response.candidates) {
      const item = document.createElement("li"); item.className = "candidate-item";
      const label = document.createElement("strong"); label.textContent = candidate.title;
      const size = document.createElement("span"); size.textContent = `${formatBytes(candidate.byteSize)}`;
      const button = document.createElement("button"); button.type = "button"; button.textContent = "Select";
      button.addEventListener("click", async () => {
        if (busy || epoch !== generation || current !== response) return;
        selected = null; clearSelectionStyle(); el("prototype-selection").hidden = true; setBusy(true);
        status("Checking your selection…");
        try {
          const result = await invoke("prototype_select", { sessionId: response.sessionId, index: candidate.index });
          if (epoch !== generation) return;
          if (!result || result.sessionId !== response.sessionId || result.index !== candidate.index || result.title !== candidate.title) throw new Error("The chosen result is no longer valid. Search again.");
          selected = result; item.className = "candidate-item is-selected"; button.textContent = "Selected";
          el("prototype-selected-title").textContent = result.title;
          el("prototype-selection").hidden = false;
          status("Selected. Choose Download selected to save this video.");
        } catch (err) { if (epoch === generation) status(message(err)); }
        finally { if (epoch === generation) setBusy(false); }
      });
      item.append(label, size, button); el("prototype-choices").append(item);
    }
  }
  el("prototype-query").addEventListener("input", () => {
    invalidateInterpretation();
    aiStatus("Request changed. Interpret again to see its local interpretation.");
  });
  el("prototype-interpret").addEventListener("click", async () => {
    if (busy) return;
    const query = el("prototype-query").value;
    if (!query.trim()) {
      invalidateInterpretation();
      aiStatus("Enter a request to interpret locally.");
      return;
    }
    const ticket = ++interpretationGeneration, epoch = generation;
    activeInterpretation = ticket;
    hideInterpretation(); setBusy(true); aiStatus("Interpreting locally…");
    const isCurrent = () => ticket === interpretationGeneration && epoch === generation &&
      el("prototype-query").value === query;
    try {
      const result = await invoke("interpret_local_request", { query });
      if (!isCurrent()) return;
      if (!validInterpretation(result)) throw { code: "invalid_response" };
      if (result.kind === "reject") {
        aiStatus("The local model could not interpret this request safely. Please rephrase. No search was started.");
        return;
      }
      verifiedInterpretation = Object.freeze({ query: result.query, choiceMode: result.choice_mode,
        sourceQuery: query, ticket, searchGeneration: epoch });
      el("prototype-interpretation-query").textContent = verifiedInterpretation.query;
      el("prototype-interpretation-mode").textContent = verifiedInterpretation.choiceMode;
      el("prototype-interpretation").hidden = false;
      aiStatus("Intent validated by Rust. No search was started.");
    } catch (err) {
      if (isCurrent()) {
        hideInterpretation();
        const code = err && typeof err.code === "string" ? err.code : "";
        aiStatus(Object.hasOwn(aiErrors, code) ? aiErrors[code] : "Local interpretation failed. No search was started.");
      }
    } finally {
      if (activeInterpretation === ticket) {
        activeInterpretation = null;
        setBusy(false);
      }
    }
  });
  async function searchViaTor(query) {
    if (busy) return;
    if (!query.trim()) { status("Describe the video you want to find."); return; }
    const epoch = ++generation; reset(); setBusy(true); status("Connecting through Tor and searching for real results…");
    try {
      const response = await invoke("prototype_search", { query });
      if (epoch !== generation) return;
      if (!validSearch(response)) throw new Error("The search results could not be validated. Download is blocked.");
      current = response; render(response, epoch); status(`${response.candidates.length} real results. Select a video to continue.`);
    } catch (err) { if (epoch === generation) status(message(err)); }
    finally { if (epoch === generation) setBusy(false); }
  }
  el("prototype-form").addEventListener("submit", async event => {
    event.preventDefault();
    await searchViaTor(el("prototype-query").value);
  });
  el("prototype-search-intent").addEventListener("click", async () => {
    if (busy) return;
    if (!verifiedTermsCurrent()) {
      invalidateInterpretation();
      aiStatus("Interpret the current request before searching these terms.");
      return;
    }
    const query = verifiedInterpretation.query;
    await searchViaTor(query);
  });
  el("prototype-download").addEventListener("click", async () => {
    if (busy || !selected || !current) return;
    const choice = selected, epoch = generation; setBusy(true); status("Downloading through Tor and verifying the saved file…");
    try {
      const file = await invoke("prototype_download", { sessionId: choice.sessionId, index: choice.index });
      if (epoch !== generation) return;
      if (!file || file.title !== choice.title || typeof file.filePath !== "string" || !file.filePath ||
          !Number.isSafeInteger(file.byteSize) || file.byteSize <= 0 || file.byteSize > 8388608 || !validToken(file.sha256)) throw new Error("File verification failed. It was not marked as saved.");
      el("prototype-file-title").textContent = file.title;
      el("prototype-file-path").textContent = file.filePath;
      el("prototype-file-size").textContent = String(file.byteSize);
      el("prototype-file-sha").textContent = file.sha256;
      el("prototype-file").hidden = false;
      status("Saved and verified. Tor stopped. Start a new search for another download.");
    } catch (err) { if (epoch === generation) status(message(err)); }
    finally {
      if (epoch === generation) { current = null; selected = null; el("prototype-selection").hidden = true; setBusy(false); }
    }
  });
  el("prototype-clear").addEventListener("click", async () => {
    if (busy) return; const epoch = ++generation; reset(); setBusy(true);
    try { await invoke("prototype_clear"); status("Session cleared. Tor stopped."); }
    catch (err) { status(message(err)); }
    finally { if (epoch === generation) setBusy(false); }
  });
})();
