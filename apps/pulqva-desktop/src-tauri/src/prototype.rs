//! Experimental Commons vertical slice, not full PULQVA release acceptance.
//! Network belongs to the backend; UI supplies a nonce and index, never a URL.
use super::*;
use pulqva_core::{ChoiceSet, request_choices};
use pulqva_discovery::{CommonsSearch, CommonsHttpsTransport, CommonsSearchPlan, CommonsTransport, DiscoveryError, MAX_MEDIA_BYTES};
use std::{fs::OpenOptions, io::Write, sync::{Arc, Mutex, TryLockError}, time::Instant};
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrototypeError { code: &'static str, message: &'static str }
fn error(code: &'static str, message: &'static str) -> PrototypeError {
    PrototypeError { code, message }
}
impl From<DownloadActionError> for PrototypeError {
    fn from(_: DownloadActionError) -> Self {
        error("package-runtime-unavailable", "Verified bundled runtime is missing or invalid.")
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceView { index: usize, title: String, byte_size: u64 }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchView {
    session_id: String, query: String, source: &'static str,
    interpretation: &'static str, candidates: Vec<ChoiceView>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionView { session_id: String, index: usize, title: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileView { file_path: String, byte_size: u64, sha256: String, title: String }

#[derive(Default)]
pub struct PrototypeState { engine: Arc<Mutex<Engine>> }
#[derive(Default)]
struct Engine { active: Option<Active> }
// Delegates HTTPS to the existing child-supervised transport. The shared owner
// allows later commands to keep the exact same Arti process and metadata alive.
struct OwnedTorTransport { arti: Arc<Mutex<Option<RunningArti>>> }
impl CommonsTransport for OwnedTorTransport {
    fn fetch(&mut self, plan: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
        let mut owner = self.arti.try_lock().map_err(|_| DiscoveryError::Transport)?;
        let child = owner.as_mut().ok_or(DiscoveryError::Transport)?;
        CommonsHttpsTransport::new(child)?.fetch(plan)
    }
}
struct Active {
    sessions: discovery_session::DiscoverySessions,
    id: String,
    choices: ChoiceSet,
    search: CommonsSearch<OwnedTorTransport>,
    arti: Arc<Mutex<Option<RunningArti>>>,
    root: PathBuf,
    ytdlp: PathBuf,
    selected: Option<usize>,
}
impl Active {
    fn alive(&mut self) -> Result<(), PrototypeError> {
        let mut owner = self.arti.try_lock().map_err(busy)?;
        if matches!(owner.as_mut().map(|p| p.try_wait()), Some(Ok(None))) { Ok(()) }
        else { Err(error("tor-unavailable", "Tor stopped; no direct connection is permitted.")) }
    }
    fn authorize(&self, id: &str, index: usize) -> Result<(), PrototypeError> {
        if id != self.id { return Err(error("session-mismatch", "Search session is no longer current.")); }
        let candidate = self.choices.candidates().get(index)
            .ok_or_else(|| error("invalid-selection", "Selection is not in this search."))?;
        self.sessions.select(id, self.choices.intent().query(), candidate.locator())
            .map_err(|_| error("session-expired", "Search session expired; search again."))?;
        Ok(())
    }
    fn close(&mut self) -> Result<(), PrototypeError> {
        {
            let mut owner = self.arti.try_lock().map_err(busy)?;
            if let Some(arti) = owner.take() {
                arti.stop_and_wait().map_err(|_| error("tor-cleanup-failed", "Tor shutdown could not be confirmed."))?;
            }
        }
        if self.root.exists() {
            fs::remove_dir_all(&self.root)
                .map_err(|_| error("runtime-cleanup-failed", "Temporary runtime cleanup failed."))?;
        }
        Ok(())
    }
}
impl Drop for Active { fn drop(&mut self) { let _ = self.close(); } }
impl PrototypeState {
    pub fn close_window(&self) -> bool {
        let Ok(mut engine) = self.engine.try_lock() else { return false; };
        if let Some(mut active) = engine.active.take() { return active.close().is_ok(); }
        true
    }
}

struct OwnedDirectory(PathBuf, bool);
impl OwnedDirectory { fn preserve(&mut self) { self.1 = true; } }
impl Drop for OwnedDirectory {
    fn drop(&mut self) { if !self.1 { let _ = fs::remove_dir_all(&self.0); } }
}
struct DownloadChild(Option<RunningYtDlp>);
impl Drop for DownloadChild {
    fn drop(&mut self) {
        if let Some(child) = self.0.take() { let _ = child.stop_and_wait(); }
    }
}
fn token() -> Result<String, PrototypeError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| error("entropy-unavailable", "Secure randomness is unavailable."))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn owned_child(parent: &Path) -> Result<OwnedDirectory, PrototypeError> {
    // Caller owns the app/Downloads parent. Reject observable symlink traversal.
    for ancestor in parent.ancestors() {
        if ancestor.is_symlink() { return Err(error("unsafe-output-path", "Output path contains a symbolic link.")); }
    }
    fs::create_dir_all(parent).map_err(|_| error("output-unavailable", "Cannot create application output directory."))?;
    let root = parent.join(token()?);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    builder.create(&root).map_err(|_| error("output-unavailable", "Cannot create exclusive application directory."))?;
    Ok(OwnedDirectory(root, false))
}
fn validate_query(query: String) -> Result<SearchIntent, PrototypeError> {
    if query.len() > 512 || query.chars().any(char::is_control) {
        return Err(error("invalid-query", "Use a single query of at most 512 UTF-8 bytes."));
    }
    SearchIntent::new(query).map_err(|_| error("invalid-query", "Enter a search query."))
}
fn busy<T>(failure: TryLockError<T>) -> PrototypeError {
    match failure {
        TryLockError::WouldBlock => error("busy", "Another backend operation is still running."),
        TryLockError::Poisoned(_) => error("backend-unavailable", "Restart PULQVA after a backend failure."),
    }
}

// Explicit paths are supplied by the trusted host; neither the UI nor a remote
// response can provide these values. The same backend journey can be exercised
// without manufacturing a Tauri AppHandle in a native acceptance harness.
#[derive(Clone, Debug)]
struct PrototypeRuntimePaths {
    resources: PathBuf,
    app_data: PathBuf,
    downloads: PathBuf,
}
impl PrototypeRuntimePaths {
    fn from_app(app: &tauri::AppHandle) -> Result<Self, PrototypeError> {
        Ok(Self {
            resources: app.path().resource_dir()
                .map_err(|_| error("package-missing", "Package resources unavailable."))?,
            app_data: app.path().app_data_dir()
                .map_err(|_| error("output-unavailable", "Application directory unavailable."))?,
            downloads: app.path().download_dir()
                .map_err(|_| error("output-unavailable", "Downloads directory unavailable."))?,
        })
    }
}
fn search_with_paths(query: String, paths: &PrototypeRuntimePaths) -> Result<Active, PrototypeError> {
    search_impl(query, paths.resources.clone(), paths.app_data.clone())
}
// Native acceptance uses this exact backend path. It does not bypass Tor,
// candidate provenance, selection, file verification, or runtime cleanup.
// The caller supplies only trusted local paths, never remote URLs or commands.
#[cfg(test)]
fn run_native_journey_with_paths(
    query: String,
    index: usize,
    paths: &PrototypeRuntimePaths,
) -> Result<FileView, PrototypeError> {
    let mut active = search_with_paths(query, paths)?;
    let result = (|| {
        active.authorize(&active.id.clone(), index)?;
        active.alive()?;
        active.selected = Some(index);
        download_impl(&mut active, index, &paths.downloads)
    })();
    let cleanup = active.close();
    cleanup?;
    result
}
fn search_impl(query: String, resources: PathBuf, app_data: PathBuf) -> Result<Active, PrototypeError> {
    let intent = validate_query(query)?;
    let mut directory = owned_child(&app_data.join("prototype-runtime"))?;
    let layout = AppRuntimeLayout::new(&directory.0)?;
    let plan = build_bundled_sidecar_materialization_plan(&layout)?;
    let plan = validate_packaged_sidecar_sources(resolve_bundled_sidecar_sources(resources, plan)?)?;
    let (layout, ytdlp) = prepare_materialized_ytdlp_prelaunch(layout, &plan, |p,r,a| {
        ytdlp_materialize::materialize_verified_ytdlp(p,r,a).map(|v| v.path().to_path_buf())
    })?;
    let (layout, arti_path) = prepare_materialized_arti_prelaunch(layout, &plan, |p,r,a| {
        arti_materialize::materialize_verified_arti(p,r,a).map(|v| v.path().to_path_buf())
    })?;
    let arti_plan = ArtiRuntimePlan::new(arti_path, layout.tor_config_file()?,
        layout.tor_cache_dir()?, layout.tor_state_dir()?,
        TorSocksEndpoint::new(DEFAULT_TOR_SOCKS_PORT).map_err(DownloadActionError::from)?);
    let mut arti = launch_prepared_arti(prepare_arti_runtime(arti_plan).map_err(DownloadActionError::from)?)
        .map_err(DownloadActionError::from)?;
    let diagnostics = crate::tor_diagnostics::enabled(
        std::env::var_os("PULQVA_TOR_DIAGNOSTICS").as_deref());
    let (readiness, trace) = if diagnostics {
        let (result, trace) = pulqva_privacy::verify_tor_readiness_observed(
            &mut arti, Duration::from_secs(90));
        (result, Some(trace))
    } else {
        (verify_tor_readiness(&mut arti, Duration::from_secs(90)), None)
    };
    let ready = match readiness {
        Ok(ready) => ready,
        Err(source) => {
            let cleanup = arti.stop_and_wait();
            if let Some(trace) = trace {
                // Cleanup precedes diagnostic I/O; a failed sink cannot skip it.
                let _ = crate::tor_diagnostics::write_failure(
                    &mut std::io::stderr().lock(), &source, &trace, cleanup.is_ok());
            }
            cleanup.map_err(|_| error("tor-cleanup-failed", "Tor shutdown could not be confirmed."))?;
            return Err(error("tor-not-ready", "Tor could not establish the required route."));
        }
    };
    let owned_arti = Arc::new(Mutex::new(Some(arti)));
    let discovered = (|| {
        let transport = OwnedTorTransport { arti: Arc::clone(&owned_arti) };
        let mut search = CommonsSearch::new(transport, ready);
        let choices = request_choices(&mut search, intent.query())
            .map_err(|_| error("search-failed", "No usable Commons results, or Tor search failed. No fallback was used."))?;
        let mut sessions = discovery_session::DiscoverySessions::default();
        let id = sessions.insert(intent.query(), choices.candidates())
            .map_err(|_| error("session-failed", "Could not establish a bounded search session."))?;
        Ok((search, choices, sessions, id))
    })();
    match discovered {
        Ok((search, choices, sessions, id)) => {
            let root = directory.0.clone();
            directory.preserve();
            Ok(Active { sessions, id, choices, search, arti: owned_arti, root, ytdlp, selected: None })
        }
        Err(failure) => {
            let mut owner = owned_arti.try_lock().map_err(busy)?;
            if let Some(arti) = owner.take() {
                arti.stop_and_wait().map_err(|_| error("tor-cleanup-failed", "Tor shutdown could not be confirmed."))?;
            }
            Err(failure)
        }
    }
}

// Detect observed output growth and unexpected filesystem entries while the child runs.
// This polling guard is not a hard filesystem quota or a process-tree sandbox.
fn output_size(root: &Path) -> Result<u64, PrototypeError> {
    let mut total = 0u64;
    for (index, entry) in fs::read_dir(root).map_err(|_| error("download-io", "Cannot inspect download output."))?.enumerate() {
        if index >= 16 { return Err(error("download-limit", "Unexpected download output count.")); }
        let entry = entry.map_err(|_| error("download-io", "Cannot inspect download output."))?;
        let meta = fs::symlink_metadata(entry.path()).map_err(|_| error("download-io", "Cannot inspect download output."))?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(error("download-limit", "Unexpected download output type."));
        }
        total = total.checked_add(meta.len()).ok_or_else(|| error("download-limit", "Download size overflow."))?;
    }
    Ok(total)
}
fn download_impl(active: &mut Active, index: usize, downloads: &Path) -> Result<FileView, PrototypeError> {
    active.alive()?;
    let selection = active.choices.select(index).map_err(|_| error("invalid-selection", "Invalid selection."))?;
    let ready = active.search.ready_transport().ok_or_else(|| error("tor-not-ready", "No verified Tor route."))?;
    let source = YtDlpMediaSourceUrl::parse(selection.candidate().locator().to_owned()).map_err(DownloadActionError::from)?;
    let output = active.root.join("transfer");
    fs::create_dir(&output).map_err(|_| error("download-io", "Cannot create exclusive download staging."))?;
    let request = YtDlpMediaRequestPlan::new_tor_gated(active.ytdlp.clone(), ready, source, &output)
        .map_err(DownloadActionError::from)?;
    let mut child = DownloadChild(Some(launch_ytdlp_request(request).map_err(DownloadActionError::from)?));
    let started = Instant::now();
    loop {
        active.alive()?;
        if started.elapsed() >= Duration::from_secs(180) {
            return Err(error("download-timeout", "Download deadline exceeded."));
        }
        if output_size(&output)? > MAX_MEDIA_BYTES {
            return Err(error("download-limit", "Download exceeded the prototype size limit."));
        }
        let process = child.0.as_mut().ok_or_else(|| error("download-process", "Download process is unavailable."))?;
        if let Some(status) = process.try_wait().map_err(|_| error("download-process", "Cannot query download process."))? {
            if !status.success() { return Err(error("download-failed", "Download did not complete successfully.")); }
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let process = child.0.take().ok_or_else(|| error("download-process", "Download process is unavailable."))?;
    let completed = process.complete_download().map_err(DownloadActionError::from)?;
    let verified = active.search.verify_completed_download(&selection, &completed)
        .map_err(|_| error("file-verification-failed", "Downloaded bytes do not match the selected result."))?;
    let mut saved = owned_child(&downloads.join("PULQVA"))?;
    let path = saved.0.join("selected.webm");
    let mut input = fs::File::open(verified.receipt().path())
        .map_err(|_| error("file-read-failed", "Cannot read verified download."))?.take(MAX_MEDIA_BYTES + 1);
    let mut output = OpenOptions::new().write(true).create_new(true).open(&path)
        .map_err(|_| error("file-save-failed", "Cannot save selected file."))?;
    let count = io::copy(&mut input, &mut output).map_err(|_| error("file-save-failed", "Cannot save selected file."))?;
    output.flush().and_then(|_| output.sync_all()).map_err(|_| error("file-save-failed", "Cannot flush selected file."))?;
    drop(output);
    drop(input);
    if count != verified.receipt().byte_size() || sha256_file(&path)? != verified.sha256() {
        return Err(error("file-verification-failed", "Saved file readback did not match."));
    }
    active.alive()?;
    let view = FileView { file_path: path.to_string_lossy().into_owned(), byte_size: count,
        sha256: verified.sha256().to_owned(), title: selection.candidate().title().to_owned() };
    // Cleanup is required before reporting success; the completed file is outside the runtime.
    active.close()?;
    saved.preserve();
    Ok(view)
}

#[tauri::command]
pub async fn prototype_search(query: String, app: tauri::AppHandle,
    state: tauri::State<'_, PrototypeState>) -> Result<SearchView, PrototypeError> {
    validate_query(query.clone())?;
    let paths = PrototypeRuntimePaths::from_app(&app)?;
    let engine = Arc::clone(&state.engine);
    spawn_backend_task(move || {
        let mut engine = engine.try_lock().map_err(busy)?;
        if let Some(mut previous) = engine.active.take() { previous.close()?; }
        let active = search_with_paths(query, &paths)?;
        let view = SearchView { session_id: active.id.clone(), query: active.choices.intent().query().to_owned(),
            source: "Wikimedia Commons", interpretation: "keyword-search-not-AI",
            candidates: active.search.last_results().iter().enumerate().map(|(index, media)| ChoiceView {
                index, title: media.candidate().title().to_owned(), byte_size: media.declared_size(),
            }).collect() };
        engine.active = Some(active);
        Ok(view)
    }).await.map_err(|_| error("backend-task-failed", "Backend operation could not finish."))?
}

#[tauri::command]
pub fn prototype_select(session_id: String, index: usize,
    state: tauri::State<'_, PrototypeState>) -> Result<SelectionView, PrototypeError> {
    let mut engine = state.engine.try_lock().map_err(busy)?;
    let active = engine.active.as_mut().ok_or_else(|| error("session-missing", "Search again."))?;
    active.authorize(&session_id, index)?;
    active.alive()?;
    active.selected = Some(index);
    Ok(SelectionView { session_id, index, title: active.choices.candidates()[index].title().to_owned() })
}

#[tauri::command]
pub async fn prototype_download(session_id: String, index: usize, app: tauri::AppHandle,
    state: tauri::State<'_, PrototypeState>) -> Result<FileView, PrototypeError> {
    let paths = PrototypeRuntimePaths::from_app(&app)?;
    let engine = Arc::clone(&state.engine);
    spawn_backend_task(move || {
        let mut engine = engine.try_lock().map_err(busy)?;
        let active = engine.active.as_mut().ok_or_else(|| error("session-missing", "Search again."))?;
        active.authorize(&session_id, index)?;
        if active.selected != Some(index) { return Err(error("selection-required", "Choose this result before downloading.")); }
        let mut active = engine.active.take().ok_or_else(|| error("session-missing", "Search again."))?;
        let outcome = download_impl(&mut active, index, &paths.downloads);
        let cleanup = active.close();
        cleanup?;
        outcome
    }).await.map_err(|_| error("backend-task-failed", "Backend operation could not finish."))?
}

#[tauri::command]
pub async fn prototype_clear(state: tauri::State<'_, PrototypeState>) -> Result<(), PrototypeError> {
    let engine = Arc::clone(&state.engine);
    spawn_backend_task(move || {
        let mut engine = engine.try_lock().map_err(busy)?;
        if let Some(mut active) = engine.active.take() { active.close()?; }
        Ok(())
    }).await.map_err(|_| error("backend-task-failed", "Backend operation could not finish."))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_empty_oversize_and_control_query() {
        assert!(validate_query(" ".into()).is_err());
        assert!(validate_query("a".repeat(513)).is_err());
        assert!(validate_query("one\ntwo".into()).is_err());
        assert_eq!(validate_query("птицы".into()).unwrap().query(), "птицы");
    }
    #[test]
    fn typed_runtime_paths_do_not_accept_frontend_input() {
        let paths = PrototypeRuntimePaths {
            resources: PathBuf::from("trusted-resources"),
            app_data: PathBuf::from("trusted-app-data"),
            downloads: PathBuf::from("trusted-downloads"),
        };
        assert_eq!(paths.resources, PathBuf::from("trusted-resources"));
        assert_eq!(paths.app_data, PathBuf::from("trusted-app-data"));
        assert_eq!(paths.downloads, PathBuf::from("trusted-downloads"));
    }
    #[test]
    fn native_journey_rejects_invalid_query_before_network_or_filesystem() {
        let paths = PrototypeRuntimePaths {
            resources: PathBuf::from("resources-not-accessed"),
            app_data: PathBuf::from("app-data-not-accessed"),
            downloads: PathBuf::from("downloads-not-accessed"),
        };
        let result = run_native_journey_with_paths("bad\nquery".to_owned(), 1, &paths);
        assert!(matches!(result, Err(PrototypeError { code: "invalid-query", .. })));
    }
    #[test]
    #[ignore = "live network acceptance requires an explicitly authorized workflow dispatch"]
    fn authorized_live_native_journey_emits_bounded_receipt() {
        use std::io::Write;

        let source_sha = std::env::var("PULQVA_SOURCE_SHA")
            .expect("PULQVA_SOURCE_SHA must identify the exact tested Git commit");
        assert_eq!(source_sha.len(), 40, "source SHA must be a full Git SHA");
        assert!(source_sha.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "source SHA must be lowercase or uppercase hexadecimal");

        let paths = PrototypeRuntimePaths {
            resources: PathBuf::from(std::env::var_os("PULQVA_E2E_RESOURCES")
                .expect("PULQVA_E2E_RESOURCES must point to staged pinned sidecars")),
            app_data: PathBuf::from(std::env::var_os("PULQVA_E2E_APP_DATA")
                .expect("PULQVA_E2E_APP_DATA must be a new isolated directory")),
            downloads: PathBuf::from(std::env::var_os("PULQVA_E2E_DOWNLOADS")
                .expect("PULQVA_E2E_DOWNLOADS must be a new isolated directory")),
        };
        let receipt_path = PathBuf::from(std::env::var_os("PULQVA_E2E_RECEIPT")
            .expect("PULQVA_E2E_RECEIPT must name a new receipt path"));

        let file = run_native_journey_with_paths("countdown".to_owned(), 1, &paths)
            .expect("real native Tor → Commons search → selected file journey must pass");
        assert!(file.byte_size > 0 && file.byte_size <= MAX_MEDIA_BYTES);
        assert_eq!(sha256_file(Path::new(&file.file_path)).expect("independent file readback"), file.sha256);
        let metadata = fs::symlink_metadata(&file.file_path).expect("saved file metadata");
        assert!(metadata.is_file() && !metadata.file_type().is_symlink());
        assert_eq!(metadata.len(), file.byte_size);

        // Dynamic receipt fields are restricted to hex digests, a fixed OS token,
        // and a bounded integer; no remote/user-controlled strings are serialized.
        let bytes = format!(
            concat!(
                "{{\n",
                "  \"schema\": 1,\n",
                "  \"scope\": \"live-native-prototype-request-choice-file\",\n",
                "  \"status\": \"PASS\",\n",
                "  \"source_sha\": \"{}\",\n",
                "  \"platform\": \"{}\",\n",
                "  \"provider\": \"Wikimedia Commons\",\n",
                "  \"query\": \"countdown\",\n",
                "  \"selected_index\": 1,\n",
                "  \"tor_required\": true,\n",
                "  \"direct_fallback_allowed\": false,\n",
                "  \"file_name\": \"selected.webm\",\n",
                "  \"byte_size\": {},\n",
                "  \"sha256\": \"{}\",\n",
                "  \"independent_saved_file_readback\": true,\n",
                "  \"absolute_anonymity_claimed\": false\n",
                "}}\n"
            ),
            source_sha, std::env::consts::OS, file.byte_size, file.sha256
        ).into_bytes();
        assert!(bytes.len() <= 16 * 1024, "receipt exceeds fixed evidence bound");
        let mut output = std::fs::OpenOptions::new().write(true).create_new(true)
            .open(&receipt_path).expect("receipt path must be new and exclusively created");
        output.write_all(&bytes).expect("write receipt");
        output.write_all(b"\n").expect("terminate receipt");
        output.sync_all().expect("sync receipt");
        println!("PULQVA_LIVE_NATIVE_JOURNEY_OK source_sha={source_sha} bytes={}", file.byte_size);
    }

    #[test]
    fn ownership_token_has_fixed_safe_encoding() {
        let a = token().unwrap(); let b = token().unwrap();
        assert_eq!(a.len(), 64); assert_ne!(a,b);
        assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
    }
}

// Linux HTTPS coordinator rejects entered Tokio contexts. Tauri's blocking pool
// still enters its runtime, so use an owned/joined OS worker only on Linux.
#[cfg(not(target_os = "linux"))]
use tauri::async_runtime::spawn_blocking as spawn_backend_task;
#[cfg(target_os = "linux")]
fn spawn_backend_task<T, F>(operation: F) -> tauri::async_runtime::JoinHandle<Result<T, PrototypeError>>
where T: Send + 'static, F: FnOnce() -> Result<T, PrototypeError> + Send + 'static {
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::Builder::new().name("pulqva-linux-backend".into()).spawn(operation)
            .map_err(|_| error("backend-worker-unavailable", "Backend worker could not start."))?
            .join().map_err(|_| error("backend-task-failed", "Backend operation could not finish."))?
    })
}
#[cfg(all(test, target_os = "linux"))]
mod linux_backend_context_tests {
    use super::*;
    #[test]
    fn blocking_pool_has_tokio_context_but_production_worker_does_not() {
        tauri::async_runtime::block_on(async {
            let entered = tauri::async_runtime::spawn_blocking(||
                tauri::async_runtime::TokioHandle::try_current().is_ok()).await.unwrap();
            assert!(entered, "regression must reproduce the original Tokio context");
            let detached = spawn_backend_task(|| Ok(
                tauri::async_runtime::TokioHandle::try_current().is_err())).await.unwrap().unwrap();
            assert!(detached, "production coordinator must remain outside Tokio");
        });
    }
}
