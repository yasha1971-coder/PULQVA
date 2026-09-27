//! T068-B0 dependency/API probe. No production executor or public network request.
//! A local SOCKS server observes the destination and rejects CONNECT before TLS.
use std::{error::Error, sync::Arc, time::Duration};

fn candidate_client(proxy: &str) -> Result<reqwest::Client, Box<dyn Error>> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(reqwest::Client::builder()
        .no_proxy()
        .proxy(reqwest::Proxy::all(proxy)?.no_proxy(None))
        .https_only(true)
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .referer(false)
        .no_gzip().no_brotli().no_deflate().no_zstd()
        .pool_max_idle_per_host(0)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(3))
        .tls_backend_preconfigured(tls)
        .tls_sslkeylogfile(false)
        .build()?)
}

// Compile the streaming API required by B. Runtime body/redirect/TLS acceptance
// must still be exercised by the full executor's controlled HTTPS tests.
async fn bounded_probe(client: reqwest::Client) -> Result<Vec<u8>, &'static str> {
    tokio::time::timeout(Duration::from_secs(3), async move {
        let mut response = client.get("https://commons.wikimedia.org/w/api.php?action=query&format=json")
            .header(reqwest::header::USER_AGENT, "PULQVA/0.0.0 (https://github.com/yasha1971-coder/PULQVA)")
            .header(reqwest::header::ACCEPT_ENCODING, "identity")
            .send().await.map_err(|_| "transport")?;
        if response.status() != reqwest::StatusCode::OK { return Err("status"); }
        if response.content_length().is_some_and(|size| size > 262_144) { return Err("size"); }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| "body")? {
            if chunk.len() > 262_144usize.saturating_sub(body.len()) { return Err("size"); }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }).await.map_err(|_| "deadline")?
}

fn main() {
    // Never run an external probe when invoked without the Rust test harness.
    eprintln!("T068 dependency probe: run cargo test --example t068_https_probe");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, process::Command, thread, time::Instant};

    fn accept_bounded(listener: &TcpListener) -> TcpStream {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((stream, peer)) => {
                    assert!(peer.ip().is_loopback());
                    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                    stream.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
                    return stream;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("local SOCKS was not used: {e}"),
            }
        }
    }

    fn remote_name_refusal_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = format!("socks5h://{}", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let mut stream = accept_bounded(&listener);
            let mut greeting = [0; 2]; stream.read_exact(&mut greeting).unwrap();
            assert_eq!(greeting[0], 5);
            let mut methods = vec![0; greeting[1] as usize]; stream.read_exact(&mut methods).unwrap();
            assert!(methods.contains(&0));
            stream.write_all(&[5, 0]).unwrap();
            let mut request = [0; 4]; stream.read_exact(&mut request).unwrap();
            assert_eq!(request, [5, 1, 0, 3], "SOCKS must forward a domain, not a host-resolved IP");
            let mut length = [0; 1]; stream.read_exact(&mut length).unwrap();
            let mut host = vec![0; length[0] as usize]; stream.read_exact(&mut host).unwrap();
            assert_eq!(host, b"commons.wikimedia.org");
            let mut port = [0; 2]; stream.read_exact(&mut port).unwrap();
            assert_eq!(u16::from_be_bytes(port), 443);
            // Ruleset denial: server does NOT connect anywhere or negotiate TLS.
            stream.write_all(&[5, 2, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
            drop(stream);
            let until = Instant::now() + Duration::from_millis(400);
            while Instant::now() < until {
                match listener.accept() {
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                    Ok(_) => panic!("unexpected SOCKS retry"),
                    Err(e) => panic!("listener failure: {e}"),
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = runtime.block_on(async { bounded_probe(candidate_client(&proxy).unwrap()).await });
        server.join().unwrap();
        assert_eq!(result, Err("transport"));
        println!("PULQVA_T068_SOCKS_DOMAIN_REFUSAL_OK fixture_outbound_connections=0");
    }

    #[test]
    fn minimal_tls_client_apis_compile_and_build() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let _guard = runtime.enter();
        assert!(candidate_client("socks5h://127.0.0.1:1").is_ok());
    }

    #[test]
    fn remote_dns_is_forwarded_and_proxy_refusal_is_not_success() { remote_name_refusal_probe(); }

    #[test]
    #[ignore = "called only by the poisoned-environment parent"]
    fn poisoned_environment_child() {
        assert_eq!(std::env::var("NO_PROXY").unwrap(), "*");
        assert_eq!(std::env::var("HTTPS_PROXY").unwrap(), "http://127.0.0.1:9");
        assert_eq!(std::env::var("SSLKEYLOGFILE").unwrap(), "t068-must-not-create-keylog");
        remote_name_refusal_probe();
        assert!(!std::path::Path::new("t068-must-not-create-keylog").exists());
    }

    #[test]
    fn ambient_proxy_exclusions_do_not_override_explicit_socks() {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "tests::poisoned_environment_child", "--ignored", "--nocapture", "--test-threads=1"]);
        for name in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"] {
            command.env(name, "http://127.0.0.1:9");
        }
        command.env("NO_PROXY", "*").env("no_proxy", "*")
            .env("SSLKEYLOGFILE", "t068-must-not-create-keylog")
            .env("SSL_CERT_FILE", "t068-not-a-certificate")
            .env("SSL_CERT_DIR", "t068-not-a-certificate-directory");
        let mut child = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "poisoned environment child failed"); break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill(); let _ = child.wait(); panic!("probe child deadline");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
