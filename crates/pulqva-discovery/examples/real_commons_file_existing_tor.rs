//! Native Windows acceptance fixture using an already-running verified Tor SOCKS
//! endpoint. The caller owns Tor lifecycle. This does not introduce a direct route.
use pulqva_core::{request_choices, retrieve_choice, FileReceipt, JourneyError};
use pulqva_discovery::{CommonsVerifiedTorTransport, DiscoveredMedia, MAX_MEDIA_BYTES};
use pulqva_privacy::{verify_existing_tor_readiness, TorSocksEndpoint};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs::{self, File, OpenOptions}, io::{Read, Write},
          path::Path, time::Duration};

type Outcome<T> = Result<T, Box<dyn Error>>;
const QUERY: &str = "countdown";
const SELECTED_INDEX: usize = 1;

fn create_private_dir(path: &Path) -> std::io::Result<()> {
    fs::DirBuilder::new().create(path)
}

fn retain_file(receipt: &FileReceipt, media: &DiscoveredMedia, bundle: &Path) -> Outcome<Value> {
    let expected = media.declared_size();
    if expected == 0 || expected > MAX_MEDIA_BYTES || receipt.byte_size() != expected
        || receipt.selected_index() != SELECTED_INDEX || receipt.title() != media.candidate().title() {
        return Err("core receipt disagrees with selected metadata".into());
    }
    let entry = fs::symlink_metadata(receipt.path())?;
    if !entry.is_file() || entry.file_type().is_symlink() || entry.len() != expected {
        return Err("completed artifact is not the expected regular file".into());
    }
    let input = File::open(receipt.path())?;
    let mut input = input.take(expected + 1);
    let mut output = OpenOptions::new().write(true).create_new(true).open(bundle.join("selected.webm"))?;
    let mut sha1 = Sha1::new();
    let mut sha256 = Sha256::new();
    let mut buffer = [0u8; 65_536];
    let mut size = 0u64;
    loop {
        let n = match input.read(&mut buffer) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if n == 0 { break; }
        size += n as u64;
        if size > expected { return Err("completed artifact grew during read".into()); }
        output.write_all(&buffer[..n])?;
        sha1.update(&buffer[..n]);
        sha256.update(&buffer[..n]);
    }
    let sha1 = lower_hex(sha1.finalize().as_ref());
    let sha256 = lower_hex(sha256.finalize().as_ref());
    if size != expected || !sha1.eq_ignore_ascii_case(media.declared_sha1()) {
        return Err("retained artifact size or digest mismatch".into());
    }
    output.sync_all()?;
    Ok(json!({"path":"selected.webm","byte_size":size,"sha1":sha1,"sha256":sha256,
              "selected_index":receipt.selected_index(),"title":receipt.title()}))
}

fn run() -> Outcome<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("expected pinned yt-dlp and NEW evidence directory".into()); }
    let ytdlp = fs::canonicalize(&args[0])?;
    if !ytdlp.is_file() { return Err("pinned yt-dlp executable missing".into()); }
    let source_sha = env::var("PULQVA_SOURCE_SHA")?;
    if source_sha.len() != 40 || !source_sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid source SHA".into());
    }
    let bundle = Path::new(&args[1]);
    create_private_dir(bundle)?;
    let downloads = bundle.join("downloads");
    create_private_dir(&downloads)?;

    eprintln!("PULQVA_WINDOWS_STAGE readiness");
    let ready = verify_existing_tor_readiness(TorSocksEndpoint::new(19050)?, Duration::from_secs(90))?;
    let transport = CommonsVerifiedTorTransport::new(ready)?;
    let cancellation = transport.cancellation();
    let mut search = transport.into_search().with_ytdlp_executable(&ytdlp);

    eprintln!("PULQVA_WINDOWS_STAGE external_search");
    let choices = request_choices(&mut search, QUERY)?;
    let selected = choices.select(SELECTED_INDEX)?;
    let media = search.last_results().get(SELECTED_INDEX).ok_or("selected metadata missing")?.clone();
    if choices.candidates().len() != search.last_results().len() || selected.candidate() != media.candidate() {
        return Err("presented choice disagrees with retained metadata".into());
    }
    let rows: Vec<Value> = search.last_results().iter().enumerate().map(|(index,item)| json!({
        "index":index,"page_id":item.page_id(),"title":item.candidate().title(),
        "locator":item.candidate().locator(),"declared_size":item.declared_size(),
        "declared_sha1":item.declared_sha1()
    })).collect();

    eprintln!("PULQVA_WINDOWS_STAGE selected_retrieval");
    let receipt = retrieve_choice(&mut search, &choices, SELECTED_INDEX, &downloads)?;
    eprintln!("PULQVA_WINDOWS_STAGE retained_file_verification");
    let file = retain_file(&receipt, &media, bundle)?;
    cancellation.cancel();
    if !matches!(request_choices(&mut search, QUERY), Err(JourneyError::SearchFailed))
        || !search.last_results().is_empty() {
        return Err("cancelled search returned success or retained stale data".into());
    }

    let evidence = json!({
        "schema":3,"scope":"native-windows-existing-tor-request-choice-file",
        "request":QUERY,"choice_count":rows.len(),"choices":rows,
        "selected_index":SELECTED_INDEX,"selected_locator":selected.candidate().locator(),
        "file":file,"core_retrieval_used":true,"file_downloaded":true,
        "cancelled_search_rejected":true,"stale_results_cleared":true,
        "source_sha":source_sha,"external_tor_lifecycle_owned_by_caller":true,
        "publisher_authenticated":false,"absolute_anonymity_claimed":false
    });
    let bytes = serde_json::to_vec_pretty(&evidence)?;
    if bytes.len() > 65_536 { return Err("evidence exceeded byte cap".into()); }
    let mut out = OpenOptions::new().write(true).create_new(true).open(bundle.join("receipt.json"))?;
    out.write_all(&bytes)?; out.write_all(b"\n")?; out.sync_all()?;
    println!("PULQVA_WINDOWS_TOR_DISCOVERY_OK choices={}", evidence["choice_count"]);
    println!("PULQVA_WINDOWS_FILE_E2E_OK");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("PULQVA_WINDOWS_FILE_E2E_FAILED category={}", error);
        std::process::exit(1);
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8;16] = b"0123456789abcdef";
    let mut result=String::with_capacity(bytes.len()*2);
    for &byte in bytes { result.push(HEX[(byte>>4) as usize] as char); result.push(HEX[(byte&15) as usize] as char); }
    result
}
