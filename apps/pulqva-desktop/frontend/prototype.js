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
  const message = err => err && typeof err.message === "string" ? err.message : "Операция завершилась ошибкой; прямое подключение не используется.";
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
      const size = document.createElement("span"); size.textContent = `${candidate.byteSize} байт`;
      const button = document.createElement("button"); button.type = "button"; button.textContent = "Выбрать";
      button.addEventListener("click", async () => {
        if (busy || epoch !== generation || current !== response) return;
        selected = null; el("prototype-selection").hidden = true; setBusy(true);
        status("Проверка выбранного результата…");
        try {
          const result = await invoke("prototype_select", { sessionId: response.sessionId, index: candidate.index });
          if (epoch !== generation) return;
          if (!result || result.sessionId !== response.sessionId || result.index !== candidate.index || result.title !== candidate.title) throw new Error("Ответ не соответствует выбранному результату.");
          selected = result;
          el("prototype-selected-title").textContent = result.title;
          el("prototype-selection").hidden = false;
          status("Результат выбран. Скачивание начнётся только по кнопке.");
        } catch (err) { if (epoch === generation) status(message(err)); }
        finally { if (epoch === generation) setBusy(false); }
      });
      item.append(label, size, button); el("prototype-choices").append(item);
    }
  }
  el("prototype-form").addEventListener("submit", async event => {
    event.preventDefault(); if (busy) return;
    const query = el("prototype-query").value;
    if (!query.trim()) { status("Введите запрос."); return; }
    const epoch = ++generation; reset(); setBusy(true); status("Подключение к Tor и поиск реальных вариантов…");
    try {
      const response = await invoke("prototype_search", { query });
      if (epoch !== generation) return;
      if (!validSearch(response)) throw new Error("Backend вернул некорректный список; скачивание заблокировано.");
      current = response; render(response, epoch); status(`Найдено: ${response.candidates.length}. Выберите результат.`);
    } catch (err) { if (epoch === generation) status(message(err)); }
    finally { if (epoch === generation) setBusy(false); }
  });
  el("prototype-download").addEventListener("click", async () => {
    if (busy || !selected || !current) return;
    const choice = selected, epoch = generation; setBusy(true); status("Скачивание через Tor и проверка сохранённого файла…");
    try {
      const file = await invoke("prototype_download", { sessionId: choice.sessionId, index: choice.index });
      if (epoch !== generation) return;
      if (!file || file.title !== choice.title || typeof file.filePath !== "string" || !file.filePath ||
          !Number.isSafeInteger(file.byteSize) || file.byteSize <= 0 || file.byteSize > 8388608 || !validToken(file.sha256)) throw new Error("Квитанция файла не прошла проверку.");
      el("prototype-file-title").textContent = file.title;
      el("prototype-file-path").textContent = file.filePath;
      el("prototype-file-size").textContent = String(file.byteSize);
      el("prototype-file-sha").textContent = file.sha256;
      el("prototype-file").hidden = false;
      status("Файл сохранён. Tor остановлен. Для следующей загрузки выполните новый поиск.");
    } catch (err) { if (epoch === generation) status(message(err)); }
    finally {
      if (epoch === generation) { current = null; selected = null; el("prototype-selection").hidden = true; setBusy(false); }
    }
  });
  el("prototype-clear").addEventListener("click", async () => {
    if (busy) return; const epoch = ++generation; reset(); setBusy(true);
    try { await invoke("prototype_clear"); status("Сессия очищена; Tor остановлен."); }
    catch (err) { status(message(err)); }
    finally { if (epoch === generation) setBusy(false); }
  });
})();
