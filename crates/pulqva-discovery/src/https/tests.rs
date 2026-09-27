//! Local SOCKS + real Rustls fixtures. No fixture opens an outbound connection.
use super::*;
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, process::Command,
          sync::atomic::AtomicUsize, thread, time::Instant};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

const CA: &[u8] = include_bytes!("fixtures/ca.der");
const CERT: &[u8] = include_bytes!("fixtures/server.der");
const WRONG: &[u8] = include_bytes!("fixtures/wrong.der");
const KEY: &[u8] = include_bytes!("fixtures/server-key.der");
const AGENT: &str = "PULQVA/0.0.0 (https://github.com/yasha1971-coder/PULQVA)";
fn roots() -> rustls::RootCertStore {
    let mut r = rustls::RootCertStore::empty();
    r.add(CertificateDer::from(CA.to_vec())).unwrap(); r
}
fn body() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"query":{"pages":[
        {"pageid":1,"ns":6,"title":"File:A.webm","imageinfo":[{"url":"https://upload.wikimedia.org/wikipedia/commons/a/ab/A.webm","size":32,"sha1":"a".repeat(40),"mime":"video/webm"}]},
        {"pageid":2,"ns":6,"title":"File:B.webm","imageinfo":[{"url":"https://upload.wikimedia.org/wikipedia/commons/b/bc/B.webm","size":64,"sha1":"b".repeat(40),"mime":"video/webm"}]}
    ]}})).unwrap()
}
fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{headers}Connection: close\r\n\r\n").into_bytes();
    bytes.extend_from_slice(body); bytes
}
fn okay() -> Vec<u8> {
    let data = body(); response("200 OK", &format!("Content-Length: {}\r\n", data.len()), &data)
}
fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                assert!(peer.ip().is_loopback());
                // Windows accepted sockets can inherit nonblocking mode.
                stream.set_nonblocking(false).unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                stream.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
                return stream;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
            Err(e) => panic!("fixture was not contacted through SOCKS: {e}"),
        }
    }
}
struct Fixture { proxy: String, worker: thread::JoinHandle<()>, http: Arc<AtomicUsize> }
impl Fixture {
    fn finish(self) -> usize { self.worker.join().unwrap(); self.http.load(Ordering::SeqCst) }
}
fn fixture(cert: Option<&'static [u8]>, reply: Vec<u8>, delay: Duration) -> Fixture {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy = format!("socks5h://{}", listener.local_addr().unwrap());
    let http = Arc::new(AtomicUsize::new(0)); let observed = http.clone();
    let worker = thread::spawn(move || {
        let mut stream = accept(&listener);
        let mut greeting = [0; 2]; stream.read_exact(&mut greeting).unwrap(); assert_eq!(greeting[0], 5);
        let mut methods = vec![0; greeting[1] as usize]; stream.read_exact(&mut methods).unwrap(); assert!(methods.contains(&0));
        stream.write_all(&[5, 0]).unwrap();
        let mut request = [0; 4]; stream.read_exact(&mut request).unwrap();
        assert_eq!(request, [5, 1, 0, 3], "destination must be sent as domain, not resolved locally");
        let mut len = [0; 1]; stream.read_exact(&mut len).unwrap();
        let mut host = vec![0; len[0] as usize]; stream.read_exact(&mut host).unwrap(); assert_eq!(host, b"commons.wikimedia.org");
        let mut port = [0; 2]; stream.read_exact(&mut port).unwrap(); assert_eq!(u16::from_be_bytes(port), 443);
        if let Some(cert) = cert {
            stream.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
            let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions().unwrap().with_no_client_auth()
                .with_single_cert(vec![CertificateDer::from(cert.to_vec())],
                    PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(KEY.to_vec()))).unwrap();
            let conn = rustls::ServerConnection::new(Arc::new(config)).unwrap();
            let mut tls = rustls::StreamOwned::new(conn, stream);
            let mut request = Vec::new(); let mut byte = [0; 1];
            while request.len() < 8192 && !request.ends_with(b"\r\n\r\n") {
                if tls.read_exact(&mut byte).is_err() { break; }
                request.push(byte[0]);
            }
            if request.ends_with(b"\r\n\r\n") {
                observed.fetch_add(1, Ordering::SeqCst);
                let text = String::from_utf8(request).unwrap().to_ascii_lowercase();
                assert!(text.starts_with("get /w/api.php?"));
                assert!(text.contains("host: commons.wikimedia.org\r\n"));
                assert!(text.contains("accept-encoding: identity\r\n"));
                assert!(text.contains("user-agent: pulqva/"));
                assert!(!text.contains("cookie:") && !text.contains("referer:") && !text.contains("authorization:"));
                thread::sleep(delay);
                let _ = tls.write_all(&reply);
                tls.conn.send_close_notify(); let _ = tls.flush();
            }
        } else { stream.write_all(&[5, 2, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap(); }
        // A redirect or implicit retry must not open another SOCKS connection.
        let until = Instant::now() + Duration::from_millis(100);
        while Instant::now() < until {
            match listener.accept() {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                Ok(_) => panic!("unexpected retry/redirect SOCKS connection"),
                Err(e) => panic!("fixture listener: {e}"),
            }
            thread::sleep(Duration::from_millis(5));
        }
    });
    Fixture { proxy, worker, http }
}
fn run(fixture: &Fixture, trust: rustls::RootCertStore, budget: Duration) -> Result<Vec<u8>, DiscoveryError> {
    execute(&format!("{API}?action=query&format=json"), &fixture.proxy, AGENT, budget,
            crate::MAX_RESPONSE_BYTES, trust, &DiscoveryCancellation::default(), || true)
}

#[test]
fn tls_success_domain_forwarding_and_parser_integration() {
    let f = fixture(Some(CERT), okay(), Duration::ZERO);
    let bytes = run(&f, roots(), Duration::from_secs(5)).unwrap();
    assert_eq!(bytes, body()); assert_eq!(crate::parse_response(&bytes).unwrap().len(), 2);
    assert_eq!(f.finish(), 1);
}
#[test]
fn tls_rejects_untrusted_root_and_wrong_hostname_before_http() {
    for (cert, trust) in [(CERT, trusted_roots()), (WRONG, roots())] {
        let f = fixture(Some(cert), okay(), Duration::ZERO);
        assert_eq!(run(&f, trust, Duration::from_secs(5)), Err(DiscoveryError::Network(NetworkFailure::ConnectOrTls)));
        assert_eq!(f.finish(), 0);
    }
}
#[test]
fn proxy_denial_never_returns_success_or_retries() {
    let f = fixture(None, vec![], Duration::ZERO);
    assert_eq!(run(&f, roots(), Duration::from_secs(5)), Err(DiscoveryError::Network(NetworkFailure::ConnectOrTls)));
    assert_eq!(f.finish(), 0);
}
#[test]
fn rejects_redirect_error_encoding_type_and_declared_oversize() {
    let cases = [
        (response("302 Found", "Location: https://example.invalid/\r\nContent-Length: 0\r\n", b""), DiscoveryError::HttpStatus(302)),
        (response("429 Too Many Requests", "Content-Length: 0\r\n", b""), DiscoveryError::HttpStatus(429)),
        (response("403 Forbidden", "Content-Length: 0\r\n", b""), DiscoveryError::HttpStatus(403)),
        (response("200 OK", "Content-Encoding: gzip\r\nContent-Length: 0\r\n", b""), DiscoveryError::ContentEncoding),
        (b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 0\r\n\r\n".to_vec(), DiscoveryError::ContentType),
        (response("200 OK", "Content-Length: 262145\r\n", b""), DiscoveryError::ResponseTooLarge),
    ];
    for (reply, error) in cases {
        let f = fixture(Some(CERT), reply, Duration::ZERO);
        assert_eq!(run(&f, roots(), Duration::from_secs(5)), Err(error)); assert_eq!(f.finish(), 1);
    }
}
#[test]
fn chunked_streams_are_bounded_and_truncation_is_not_success() {
    let data = body();
    let mut chunks = format!("{:X}\r\n", data.len()).into_bytes();
    chunks.extend_from_slice(&data); chunks.extend_from_slice(b"\r\n0\r\n\r\n");
    let f = fixture(Some(CERT), response("200 OK", "Transfer-Encoding: chunked\r\n", &chunks), Duration::ZERO);
    assert_eq!(run(&f, roots(), Duration::from_secs(5)).unwrap(), data); assert_eq!(f.finish(), 1);
    let mut huge = b"40001\r\n".to_vec(); huge.extend(vec![b' '; 262145]); huge.extend_from_slice(b"\r\n0\r\n\r\n");
    let f = fixture(Some(CERT), response("200 OK", "Transfer-Encoding: chunked\r\n", &huge), Duration::ZERO);
    assert_eq!(run(&f, roots(), Duration::from_secs(5)), Err(DiscoveryError::ResponseTooLarge)); assert_eq!(f.finish(), 1);
    let f = fixture(Some(CERT), response("200 OK", "Content-Length: 50\r\n", b"{}"), Duration::ZERO);
    assert!(run(&f, roots(), Duration::from_secs(5)).is_err()); assert_eq!(f.finish(), 1);
}
#[test]
fn deadline_is_total_and_does_not_retry_stalled_server() {
    let f = fixture(Some(CERT), okay(), Duration::from_secs(2));
    assert_eq!(run(&f, roots(), Duration::from_secs(1)), Err(DiscoveryError::Network(NetworkFailure::Timeout)));
    assert_eq!(f.finish(), 1);
}
#[test]
fn cancellation_and_liveness_interrupt_a_pending_request() {
    for cancelled in [false, true] {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let flag = DiscoveryCancellation::default(); let signal = flag.clone();
        let alive = Arc::new(AtomicBool::new(true)); let stopped = alive.clone();
        let t = thread::spawn(move || { thread::sleep(Duration::from_millis(80));
            if cancelled { signal.cancel(); } else { stopped.store(false, Ordering::Release); } });
        let out = rt.block_on(async { tokio::time::timeout(Duration::from_secs(2),
            supervise(std::future::pending(), &flag, || alive.load(Ordering::Acquire))).await });
        t.join().unwrap(); assert_eq!(out.unwrap(), Err(DiscoveryError::Transport));
    }
}
#[test]
fn invalid_route_and_precancelled_requests_do_not_connect() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); listener.set_nonblocking(true).unwrap();
    let proxy = format!("socks5h://{}", listener.local_addr().unwrap());
    let cancelled = DiscoveryCancellation::default(); cancelled.cancel();
    assert_eq!(execute(API, &proxy, AGENT, Duration::from_secs(1), 100, roots(), &cancelled, || true), Err(DiscoveryError::Transport));
    for bad in ["http://commons.wikimedia.org/w/api.php", "https://commons.wikimedia.org.evil.invalid/w/api.php", "https://commons.wikimedia.org/other"] {
        assert_eq!(execute(bad, &proxy, AGENT, Duration::from_secs(1), 100, roots(), &DiscoveryCancellation::default(), || true), Err(DiscoveryError::InvalidRequest));
    }
    for bad in ["socks5://127.0.0.1:9050", "socks5h://localhost:9050", "socks5h://192.0.2.1:9050"] {
        assert_eq!(validate_route(API, bad), Err(DiscoveryError::InvalidRequest));
    }
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
}
#[test]
#[ignore = "only invoked by the isolated poisoned-environment parent"]
fn poisoned_child() {
    assert_eq!(std::env::var("NO_PROXY").unwrap(), "*");
    tls_success_domain_forwarding_and_parser_integration();
    assert!(!std::path::Path::new(&std::env::var("SSLKEYLOGFILE").unwrap()).exists());
}
#[test]
fn ambient_proxy_cert_and_keylog_environment_does_not_override_policy() {
    let root = std::env::temp_dir().join(format!("pulqva-t068-keylog-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&root).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", "https::tests::poisoned_child", "--ignored", "--nocapture", "--test-threads=1"]);
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"] {
        command.env(key, "http://127.0.0.1:9");
    }
    command.env("NO_PROXY", "*").env("no_proxy", "*").env("SSLKEYLOGFILE", root.join("keys"))
        .env("SSL_CERT_FILE", root.join("not-a-cert")).env("SSL_CERT_DIR", root.join("not-a-dir"));
    let mut child = command.spawn().unwrap(); let until = Instant::now() + Duration::from_secs(15);
    let success = loop {
        if let Some(status) = child.try_wait().unwrap() { break status.success(); }
        if Instant::now() >= until { let _ = child.kill(); let _ = child.wait(); break false; }
        thread::sleep(Duration::from_millis(10));
    };
    let no_log = !root.join("keys").exists();
    if !no_log { std::fs::remove_file(root.join("keys")).unwrap(); }
    std::fs::remove_dir(root).unwrap(); assert!(success); assert!(no_log);
}

#[test]
fn readiness_categories_do_not_expose_underlying_messages() {
    use pulqva_privacy::ArtiProcessError;
    let cases = [
        (TorReadinessError::Timeout, ReadinessFailure::Timeout),
        (TorReadinessError::Protocol("PRIVATE"), ReadinessFailure::Protocol),
        (TorReadinessError::Io { operation: "PRIVATE", source: std::io::Error::other("PRIVATE") }, ReadinessFailure::Io),
        (TorReadinessError::BootstrapActivation(ArtiProcessError::Io {
            operation: "PRIVATE", source: std::io::Error::other("PRIVATE"),
        }), ReadinessFailure::BootstrapActivation),
    ];
    for (source, expected) in cases {
        let error = readiness_failure(source);
        assert_eq!(error, DiscoveryError::Readiness(expected));
        assert!(!format!("{error:?}: {error}").contains("PRIVATE"));
        assert!(std::error::Error::source(&error).is_none());
    }
}
