/* All network and file operations belong to Rust IPC. No frontend Internet API. */
(() => {
  "use strict";
  const invoke = window.__TAURI__.core.invoke;
  const el = id => document.querySelector(`#${id}`);
  let busy = false;
  let current = null;
  let selected = null;
  let generation = 0;
  const status = text => { el("prototype-status").textContent = text; };
  const setBusy = value => {
    busy = value;
    el("prototype-search").disabled = value;
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
    current = null; selected = null;
    el("prototype-choices").replaceChildren();
    el("prototype-selection").hidden = true;
    el("prototype-file").hidden = true;
    el("prototype-download").disabled = true;
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
  el("prototype-form").addEventListener("submit", async event => {
    event.preventDefault(); if (busy) return;
    const query = el("prototype-query").value;
    if (!query.trim()) { status("Describe the video you want to find."); return; }
    const epoch = ++generation; reset(); setBusy(true); status("Connecting through Tor and searching for real results…");
    try {
      const response = await invoke("prototype_search", { query });
      if (epoch !== generation) return;
      if (!validSearch(response)) throw new Error("The search results could not be validated. Download is blocked.");
      current = response; render(response, epoch); status(`${response.candidates.length} real results. Select a video to continue.`);
    } catch (err) { if (epoch === generation) status(message(err)); }
    finally { if (epoch === generation) setBusy(false); }
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
