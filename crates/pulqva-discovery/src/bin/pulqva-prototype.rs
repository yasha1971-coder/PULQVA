//! v0.1.0-alpha.1: local-browser prototype using the existing Tor/discovery/file boundaries.
//! This is not PULQVA v1, a network sandbox or a claim of complete anonymity.
use pulqva_core::{request_choices, ChoiceSet, FileReceipt};
use pulqva_discovery::{CommonsHttpsTransport, CommonsSearch, CommonsTransport, DiscoveryCancellation,
    DiscoveredMedia, MAX_MEDIA_BYTES};
use pulqva_privacy::{launch_prepared_arti, launch_ytdlp_request, prepare_arti_runtime,
    ArtiRuntimePlan, TorSocksEndpoint, YtDlpMediaRequestPlan, YtDlpMediaSourceUrl};
use serde::Deserialize;
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{env, fs::{self, File, OpenOptions}, io::{self, Read, Write},
    net::{TcpListener, TcpStream}, path::{Path, PathBuf}, process::{Command, Stdio},
    sync::{mpsc::{self, Receiver, SyncSender}, Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread, time::{Duration, Instant}};

const VERSION: &str = "0.1.0-alpha.1";
const SESSION_TTL: Duration = Duration::from_secs(600);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
const ARTI_SUMS: &str = include_str!("../../../../sidecars/arti/SHA256SUMS");
const YTDLP_SUMS: &str = include_str!("../../../../sidecars/yt-dlp/SHA256SUMS");
const PAGE: &str = include_str!("../../../../apps/pulqva-prototype/index.html");
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone)]
struct View(Arc<Mutex<Value>>);
impl View {
    fn new() -> Self { Self(Arc::new(Mutex::new(json!({"phase":"idle","version":VERSION})))) }
    fn set(&self, value: Value) { if let Ok(mut v) = self.0.lock() { *v = value; } }
    fn get(&self) -> Value { self.0.lock().map(|v| v.clone()).unwrap_or_else(|_| json!({"phase":"error","code":"state-unavailable"})) }
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Input { Search { query: String }, Select { session: String, index: usize }, Quit }
struct RuntimeDir(PathBuf);
impl RuntimeDir {
    fn new() -> Result<Self> {
        let path = env::temp_dir().join(format!("pulqva-alpha-{}", nonce()?));
        private_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for RuntimeDir { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn nonce() -> Result<String> {
    let mut bytes = [0u8; 32];
    rustls::crypto::ring::default_provider().secure_random.fill(&mut bytes).map_err(|_| "entropy-unavailable")?;
    Ok(hex(&bytes))
}
fn private_dir(path: &Path) -> Result<()> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; b.mode(0o700); }
    b.create(path).map_err(|_| "cannot-create-owned-directory")
}
fn regular(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path).map_err(|_| "file-missing")?;
    if !m.is_file() || m.file_type().is_symlink() { return Err("non-regular-file"); }
    Ok(())
}
fn check_binary(path: &Path, sums: &str, asset: &str) -> Result<()> {
    regular(path)?;
    let mut rows = sums.lines().filter_map(|l| { let mut p = l.split_whitespace(); Some((p.next()?, p.next()?)) }).filter(|(_,n)| *n == asset);
    let expected = rows.next().ok_or("missing-binary-pin")?.0;
    if rows.next().is_some() { return Err("duplicate-binary-pin"); }
    let mut input = File::open(path).map_err(|_| "binary-unreadable")?;
    let mut sha = Sha256::new(); let mut buf = [0u8; 65536];
    loop { let n = input.read(&mut buf).map_err(|_| "binary-unreadable")?; if n == 0 { break; } sha.update(&buf[..n]); }
    if hex(sha.finalize().as_ref()) != expected { return Err("binary-hash-mismatch"); }
    Ok(())
}
fn binaries() -> Result<(PathBuf, PathBuf)> {
    let exe = env::current_exe().map_err(|_| "executable-location-unavailable")?;
    let root = exe.parent().ok_or("package-location-unavailable")?.join("sidecars");
    #[cfg(windows)] let (arti, yt, asset) = ("arti.exe", "yt-dlp.exe", "windows-x86_64/arti.exe");
    #[cfg(not(windows))] let (arti, yt, asset) = ("arti", "yt-dlp_linux", "linux-x86_64/arti");
    let arti_path = root.join(arti); let yt_path = root.join(yt);
    check_binary(&arti_path, ARTI_SUMS, asset)?;
    check_binary(&yt_path, YTDLP_SUMS, yt)?;
    Ok((arti_path, yt_path))
}
fn validate_query(query: &str) -> Result<()> {
    if query.trim().is_empty() || query.len() > 512 || query.chars().any(char::is_control) { return Err("invalid-query"); }
    Ok(())
}
fn choose(session: &str, supplied: &str, created: Instant, count: usize, index: usize) -> Result<()> {
    if supplied != session { return Err("foreign-session"); }
    if created.elapsed() >= SESSION_TTL { return Err("session-expired"); }
    if index >= count { return Err("invalid-selection"); }
    Ok(())
}
fn output_dir() -> Result<PathBuf> {
    let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).ok_or("home-directory-unavailable")?;
    let home = PathBuf::from(home);
    if !home.is_absolute() { return Err("invalid-home-directory"); }
    #[cfg(windows)] {
        use std::path::{Component, Prefix};
        if !matches!(home.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))) {
            return Err("network-home-directory-not-supported");
        }
    }
    let parent = home.join("Downloads").join("PULQVA");
    let mut cursor = PathBuf::new();
    for part in parent.components() {
        cursor.push(part);
        if cursor.exists() {
            let m = fs::symlink_metadata(&cursor).map_err(|_| "output-directory-unavailable")?;
            if m.file_type().is_symlink() || !m.is_dir() { return Err("unsafe-output-directory"); }
        } else { fs::create_dir(&cursor).map_err(|_| "output-directory-unavailable")?; }
    }
    let dir = parent.join(format!("file-{}", nonce()?)); private_dir(&dir)?; Ok(dir)
}
fn retain(receipt: &FileReceipt, media: &DiscoveredMedia) -> Result<Value> {
    regular(receipt.path())?;
    let expected = media.declared_size();
    if expected == 0 || expected > MAX_MEDIA_BYTES || expected != receipt.byte_size() { return Err("file-size-mismatch"); }
    let output = output_dir()?; let partial = output.join("selected.webm.part"); let final_path = output.join("selected.webm");
    let result = (|| {
        let mut source = File::open(receipt.path()).map_err(|_| "file-unreadable")?.take(expected + 1);
        let mut target = OpenOptions::new().write(true).create_new(true).open(&partial).map_err(|_| "output-unwritable")?;
        let mut sha1 = Sha1::new(); let mut sha256 = Sha256::new(); let mut size = 0u64; let mut buf = [0u8; 65536];
        loop { let n = source.read(&mut buf).map_err(|_| "file-unreadable")?; if n == 0 { break; }
            size += n as u64; if size > expected { return Err("file-grew-during-copy"); }
            target.write_all(&buf[..n]).map_err(|_| "output-unwritable")?; sha1.update(&buf[..n]); sha256.update(&buf[..n]); }
        if size != expected || !hex(sha1.finalize().as_ref()).eq_ignore_ascii_case(media.declared_sha1()) { return Err("file-content-mismatch"); }
        target.sync_all().map_err(|_| "output-sync-failed")?; drop(target);
        fs::rename(&partial, &final_path).map_err(|_| "output-publish-failed")?;
        Ok(json!({"phase":"complete","path":final_path,"bytes":size,"sha256":hex(sha256.finalize().as_ref()),"publisherAuthenticated":false}))
    })();
    if result.is_err() { let _ = fs::remove_file(&partial); let _ = fs::remove_dir(&output); }
    result
}
fn retrieve<T: CommonsTransport>(search: &mut CommonsSearch<T>, choices: &ChoiceSet, index: usize,
    yt: &Path, dir: &Path, shutdown: &AtomicBool) -> Result<Value> {
    let selected = choices.select(index).map_err(|_| "invalid-selection")?;
    let media = search.last_results().get(index).ok_or("missing-selection-metadata")?.clone();
    if selected.candidate() != media.candidate() { return Err("selection-provenance-mismatch"); }
    let ready = search.ready_transport().ok_or("tor-not-ready")?;
    let source = YtDlpMediaSourceUrl::parse(media.candidate().locator().to_owned()).map_err(|_| "invalid-media-source")?;
    let plan = YtDlpMediaRequestPlan::new_tor_gated(yt, ready, source, dir).map_err(|_| "download-plan-rejected")?;
    let mut child = launch_ytdlp_request(plan).map_err(|_| "download-launch-failed")?;
    let started = Instant::now();
    loop {
        let size_ok = fs::read_dir(dir).ok().and_then(|mut rows| {
            rows.try_fold(0u64, |total, row| { let path = row.ok()?.path(); let m = fs::symlink_metadata(path).ok()?;
                if m.file_type().is_symlink() || !m.is_file() { return None; } total.checked_add(m.len()) })
        }).is_some_and(|n| n <= MAX_MEDIA_BYTES);
        if shutdown.load(Ordering::Acquire) || started.elapsed() >= DOWNLOAD_TIMEOUT || !size_ok {
            child.stop_and_wait().map_err(|_| "download-cleanup-failed")?;
            return Err("download-cancelled-timeout-or-size-limit");
        }
        match child.try_wait() { Ok(Some(_)) => break, Ok(None) => thread::sleep(Duration::from_millis(50)), Err(_) => {
            let _ = child.stop_and_wait(); return Err("download-status-failed"); } }
    }
    let completed = child.complete_download().map_err(|_| "download-failed")?;
    let verified = search.verify_completed_download(&selected, &completed).map_err(|_| "download-verification-failed")?;
    retain(&verified.into_receipt(), &media)
}
fn journey(query: String, arti_bin: &Path, yt: &Path, view: &View, rx: &Receiver<Input>, shutdown: &AtomicBool,
    canceller: &Mutex<Option<DiscoveryCancellation>>) -> Result<Option<String>> {
    let runtime = RuntimeDir::new()?;
    let plan = ArtiRuntimePlan::new(arti_bin, runtime.0.join("config/pulqva.toml"), runtime.0.join("cache"), runtime.0.join("state"),
        TorSocksEndpoint::new(19050).map_err(|_| "invalid-tor-port")?);
    let mut arti = launch_prepared_arti(prepare_arti_runtime(plan).map_err(|_| "tor-prepare-failed")?).map_err(|_| "tor-launch-failed")?;
    let result = (|| {
        let transport = CommonsHttpsTransport::new(&mut arti).map_err(|_| "tor-readiness-failed")?;
        if shutdown.load(Ordering::Acquire) { return Err("cancelled"); }
        let cancel = transport.cancellation();
        *canceller.lock().map_err(|_| "state-unavailable")? = Some(cancel);
        let mut search = transport.into_search();
        let choices = request_choices(&mut search, query).map_err(|_| "search-failed-or-too-few-results")?;
        let session = nonce()?; let created = Instant::now();
        let rows: Vec<Value> = search.last_results().iter().enumerate().map(|(index, m)| json!({"index":index,
            "title":m.candidate().title(),"bytes":m.declared_size(),"source":"Wikimedia Commons"})).collect();
        view.set(json!({"phase":"choosing","session":session,"candidates":rows}));
        loop {
            if shutdown.load(Ordering::Acquire) { return Ok((None, None)); }
            if created.elapsed() >= SESSION_TTL { return Err("session-expired"); }
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Input::Search { query }) => return Ok((Some(query), None)),
                Ok(Input::Quit) => return Ok((None, None)),
                Ok(Input::Select { session: supplied, index }) => {
                    choose(&session, &supplied, created, choices.candidates().len(), index)?;
                    let downloads = runtime.0.join("downloads"); private_dir(&downloads)?;
                    let file = retrieve(&mut search, &choices, index, yt, &downloads, shutdown)?;
                    return Ok((None, Some(file)));
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}, Err(_) => return Ok((None, None)),
            }
        }
    })();
    if let Ok(mut cancel) = canceller.lock() { *cancel = None; }
    arti.stop_and_wait().map_err(|_| "tor-cleanup-failed")?;
    let (next, completed) = result?;
    fs::remove_dir_all(&runtime.0).map_err(|_| "runtime-cleanup-failed")?;
    if let Some(file) = completed { view.set(file); }
    Ok(next)
}
fn engine(rx: Receiver<Input>, view: View, shutdown: Arc<AtomicBool>, done: Arc<AtomicBool>, canceller: Arc<Mutex<Option<DiscoveryCancellation>>>, arti: PathBuf, yt: PathBuf) {
    let mut pending = None;
    while !shutdown.load(Ordering::Acquire) {
        let query = match pending.take() {
            Some(query) => query,
            None => match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Input::Search { query }) => query, Ok(Input::Quit) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                _ => continue,
            }
        };
        view.set(json!({"phase":"searching"}));
        match journey(query, &arti, &yt, &view, &rx, &shutdown, &canceller) {
            Ok(next) => pending = next,
            Err(code) => view.set(json!({"phase":"error","code":code})),
        }
    }
    done.store(true, Ordering::Release);
}
struct Http { method: String, path: String, origin: Option<String>, body: Vec<u8> }
fn parse_http(raw: &[u8], authority: &str) -> Result<(Http, usize)> {
    let end = raw.windows(4).position(|v| v == b"\r\n\r\n").ok_or("incomplete-header")?;
    let header = std::str::from_utf8(&raw[..end]).map_err(|_| "invalid-header")?;
    let mut lines = header.split("\r\n"); let parts: Vec<_> = lines.next().ok_or("invalid-request")?.split(' ').collect();
    if parts.len() != 3 || parts[2] != "HTTP/1.1" || !matches!(parts[0], "GET" | "POST") { return Err("invalid-request"); }
    let mut host = None; let mut origin = None; let mut length = None; let mut content_type = None;
    for line in lines {
        let (key, value) = line.split_once(':').ok_or("invalid-header")?; let value = value.trim();
        match key.to_ascii_lowercase().as_str() {
            "host" => { if host.replace(value).is_some() { return Err("duplicate-host"); } },
            "origin" => { if origin.replace(value.to_owned()).is_some() { return Err("duplicate-origin"); } },
            "content-length" => { if length.replace(value.parse::<usize>().map_err(|_| "invalid-length")?).is_some() { return Err("duplicate-length"); } },
            "content-type" => { if content_type.replace(value).is_some() { return Err("duplicate-content-type"); } },
            "transfer-encoding" => return Err("transfer-encoding-not-supported"), _ => {},
        }
    }
    if host != Some(authority) { return Err("host-rejected"); }
    let count = length.unwrap_or(0); if count > 2048 { return Err("request-too-large"); }
    let expected_origin = format!("http://{authority}");
    if parts[0] == "POST" && (origin.as_deref() != Some(expected_origin.as_str()) || content_type != Some("application/json")) { return Err("origin-or-content-type-rejected"); }
    Ok((Http { method: parts[0].to_owned(), path: parts[1].to_owned(), origin, body: Vec::new() }, end + 4 + count))
}
fn read_http(stream: &mut TcpStream, authority: &str) -> Result<Http> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|_| "socket-error")?;
    let mut raw = Vec::new(); let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).map_err(|_| "socket-error")?; if n == 0 { return Err("incomplete-request"); }
        raw.extend_from_slice(&buf[..n]); if raw.len() > 8192 { return Err("request-too-large"); }
        if raw.windows(4).any(|v| v == b"\r\n\r\n") { let (mut request, needed) = parse_http(&raw, authority)?;
            if raw.len() >= needed {
                if raw.len() != needed { return Err("extra-request-bytes"); }
                let begin = raw.windows(4).position(|v| v == b"\r\n\r\n").ok_or("invalid-header")? + 4;
                request.body = raw[begin..].to_vec(); return Ok(request);
            }
        }
    }
}
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8;64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for b in bytes.chunks(3) {
        let n = ((b[0] as u32) << 16) | ((b.get(1).copied().unwrap_or(0) as u32) << 8) | b.get(2).copied().unwrap_or(0) as u32;
        result.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        result.push(if b.len() > 1 { ALPHABET[((n >> 6) & 63) as usize] as char } else { '=' });
        result.push(if b.len() > 2 { ALPHABET[(n & 63) as usize] as char } else { '=' });
    }
    result
}
fn inline_hash(tag: &str) -> String {
    let open = format!("<{tag}>"); let close = format!("</{tag}>");
    let content = PAGE.split_once(&open).and_then(|(_, tail)| tail.split_once(&close)).map(|(body, _)| body).unwrap_or("");
    base64(Sha256::digest(content.as_bytes()).as_ref())
}
fn reply(stream: &mut TcpStream, status: &str, mime: &str, body: &[u8]) {
    // Inline code is immutable package code; data is returned only as JSON/textContent.
    let headers = format!("HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; script-src 'sha256-{}'; style-src 'sha256-{}'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'\r\n\r\n", body.len(), inline_hash("script"), inline_hash("style"));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2))); let _ = stream.write_all(headers.as_bytes()); let _ = stream.write_all(body);
}
fn route(request: Http, prefix: &str, view: &View, tx: &SyncSender<Input>, shutdown: &AtomicBool, canceller: &Mutex<Option<DiscoveryCancellation>>) -> Result<(&'static str, Vec<u8>)> {
    if request.method == "GET" && request.path == format!("{prefix}/") { return Ok(("text/html; charset=utf-8", PAGE.as_bytes().to_vec())); }
    if request.method == "GET" && request.path == format!("{prefix}/status") { return Ok(("application/json", serde_json::to_vec(&view.get()).map_err(|_| "state-unavailable")?)); }
    if request.method != "POST" || request.path != format!("{prefix}/command") || request.origin.is_none() { return Err("route-rejected"); }
    let input: Input = serde_json::from_slice(&request.body).map_err(|_| "invalid-command")?;
    let mut current = view.0.lock().map_err(|_| "state-unavailable")?;
    let phase = current["phase"].as_str().unwrap_or("error");
    let next = match &input {
        Input::Search { query } => { validate_query(query)?; if matches!(phase,"searching"|"downloading") { return Err("operation-in-progress"); } "searching" },
        Input::Select { session, index } => { if phase != "choosing" || current["session"].as_str() != Some(session) || *index >= current["candidates"].as_array().map_or(0, Vec::len) { return Err("selection-rejected"); } "downloading" },
        Input::Quit => { shutdown.store(true, Ordering::Release); if let Ok(c) = canceller.lock() { if let Some(c) = c.as_ref() { c.cancel(); } } "stopping" },
    };
    tx.try_send(input).map_err(|_| "command-queue-unavailable")?;
    *current = json!({"phase":next});
    Ok(("application/json", b"{\"accepted\":true}".to_vec()))
}
fn open_browser(address: &str) -> Result<()> {
    #[cfg(windows)] let explorer = PathBuf::from(env::var_os("SystemRoot").ok_or("system-directory-unavailable")?).join("explorer.exe");
    #[cfg(windows)] if !explorer.is_absolute() { return Err("invalid-system-directory"); }
    #[cfg(windows)] let child = Command::new(explorer).arg(address).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    #[cfg(not(windows))] let child = Command::new("xdg-open").arg(address).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    child.map(|mut child| { thread::spawn(move || { let _ = child.wait(); }); }).map_err(|_| "browser-launch-failed")
}
fn run() -> Result<()> {
    let (arti, yt) = binaries()?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| "loopback-unavailable")?;
    let authority = listener.local_addr().map_err(|_| "loopback-unavailable")?.to_string();
    let prefix = format!("/{}", nonce()?);
    listener.set_nonblocking(true).map_err(|_| "loopback-unavailable")?;
    let view = View::new(); let (tx, rx) = mpsc::sync_channel(2);
    let shutdown = Arc::new(AtomicBool::new(false)); let done = Arc::new(AtomicBool::new(false)); let canceller = Arc::new(Mutex::new(None));
    let (v,s,d,c) = (view.clone(), shutdown.clone(), done.clone(), canceller.clone());
    let worker = thread::spawn(move || engine(rx,v,s,d,c,arti,yt));
    let opened = open_browser(&format!("http://{authority}{prefix}/"));
    if opened.is_err() { shutdown.store(true, Ordering::Release); }
    let mut touched = Instant::now();
    while !done.load(Ordering::Acquire) && !worker.is_finished() {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() { continue; }
                let response = read_http(&mut stream, &authority).and_then(|r| route(r,&prefix,&view,&tx,&shutdown,&canceller));
                match response { Ok((mime,body)) => { touched = Instant::now(); reply(&mut stream,"200 OK",mime,&body); },
                    Err(code) => reply(&mut stream,"400 Bad Request","application/json",serde_json::to_string(&json!({"error":code})).unwrap_or_default().as_bytes()) }
            },
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(20)),
            Err(_) => { shutdown.store(true, Ordering::Release); },
        }
        if touched.elapsed() > Duration::from_secs(1200) { shutdown.store(true, Ordering::Release); }
    }
    let _ = tx.try_send(Input::Quit); shutdown.store(true, Ordering::Release);
    worker.join().map_err(|_| "worker-failed")?; opened
}
fn main() {
    if env::args().any(|a| a == "--version") { println!("PULQVA {VERSION}"); return; }
    if let Err(code) = run() { eprintln!("PULQVA prototype: {code}"); std::process::exit(1); }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn inline_policy_hash_encoding() {
        assert_eq!(base64(b""), ""); assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8="); assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(inline_hash("script").len(), 44);
        assert!(!PAGE.contains("<script src="));
    }
    #[test] fn tokens_and_query_boundary() { let a=nonce().unwrap(); assert_eq!(a.len(),64); assert_ne!(a,nonce().unwrap()); assert!(validate_query("birds").is_ok()); for q in ["", "\n", &"a".repeat(513)] { assert!(validate_query(q).is_err()); } }
    #[test] fn session_and_index_are_bound() { let now=Instant::now(); assert!(choose("a","a",now,2,1).is_ok()); assert_eq!(choose("a","b",now,2,1),Err("foreign-session")); assert_eq!(choose("a","a",now,2,2),Err("invalid-selection")); }
    #[test] fn foreign_host_origin_and_smuggling_fail_closed() {
        let good=b"POST /token/command HTTP/1.1\r\nHost: 127.0.0.1:1234\r\nOrigin: http://127.0.0.1:1234\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        assert!(parse_http(good,"127.0.0.1:1234").is_ok());
        let text=String::from_utf8(good.to_vec()).unwrap();
        for bad in [text.replace("Origin: http://127.0.0.1:1234","Origin: https://evil.invalid"), text.replace("Host: 127.0.0.1:1234","Host: evil.invalid"), text.replace("Content-Length: 2","Content-Length: 2\r\nContent-Length: 2"), text.replace("Content-Length: 2","Transfer-Encoding: chunked")] { assert!(parse_http(bad.as_bytes(),"127.0.0.1:1234").is_err()); }
    }
    #[test] fn local_fixture_or_foreign_selection_never_dispatches() {
        let v=View::new(); let (tx,rx)=mpsc::sync_channel(2); let stop=AtomicBool::new(false); let c=Mutex::new(None);
        let r=Http { method:"POST".into(),path:"/secret/command".into(),origin:Some("http://local".into()),body:br#"{"action":"select","session":"invented","index":0}"#.to_vec() };
        assert!(route(r,"/secret",&v,&tx,&stop,&c).is_err()); assert!(rx.try_recv().is_err());
    }
    #[test] fn closed_channel_does_not_mint_success() {
        let v=View::new();let(tx,rx)=mpsc::sync_channel(1);drop(rx);let stop=AtomicBool::new(false);let c=Mutex::new(None);
        let r=Http { method:"POST".into(),path:"/s/command".into(),origin:Some("local".into()),body:br#"{"action":"search","query":"birds"}"#.to_vec() };
        assert!(route(r,"/s",&v,&tx,&stop,&c).is_err()); assert_eq!(v.get()["phase"],"idle");
    }
}
