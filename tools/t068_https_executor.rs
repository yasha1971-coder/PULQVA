//! T068-B production-executor API candidate. No external request is made here.
use std::{sync::Arc,time::Duration};
fn client(proxy:&str)->reqwest::Client {
 let roots=rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
 let tls=rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
  .with_safe_default_protocol_versions().unwrap().with_root_certificates(roots).with_no_client_auth();
 reqwest::Client::builder().no_proxy().proxy(reqwest::Proxy::all(proxy).unwrap().no_proxy(None))
  .https_only(true).http1_only().redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
  .referer(false).no_gzip().no_brotli().no_deflate().no_zstd().pool_max_idle_per_host(0)
  .connect_timeout(Duration::from_secs(15)).timeout(Duration::from_secs(60))
  .tls_backend_preconfigured(tls).tls_sslkeylogfile(false).build().unwrap()
}
async fn bounded(c:reqwest::Client)->Result<Vec<u8>,()> {
 let mut r=c.get("https://commons.wikimedia.org/w/api.php").header(reqwest::header::ACCEPT_ENCODING,"identity").send().await.map_err(|_|())?;
 if r.status()!=reqwest::StatusCode::OK{return Err(())}
 if r.content_length().is_some_and(|n|n>262144){return Err(())}
 let mut out=Vec::new(); while let Some(x)=r.chunk().await.map_err(|_|())? { if x.len()>262144usize.saturating_sub(out.len()){return Err(())} out.extend_from_slice(&x); } Ok(out)
}
fn main(){}
#[cfg(test)] mod tests { use super::*; #[test] fn exact_client_policy_builds(){let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();let _g=rt.enter();let _=client("socks5h://127.0.0.1:1");} #[test] fn bounded_api_compiles(){let _=bounded;} }
