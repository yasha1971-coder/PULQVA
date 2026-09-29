//! Fixed public Linux acceptance fixture; not UI/AI or Windows release evidence.
//! Uses the production coordinator/retriever, not a second download implementation.
//! CI applies a whole-process watchdog. Product-level in-flight cancellation,
//! descendant confinement and race-proof file ownership are still separate gates.
use pulqva_core::{request_choices, retrieve_choice, FileReceipt, JourneyError};
use pulqva_discovery::{CommonsHttpsTransport, DiscoveredMedia, MAX_MEDIA_BYTES};
use pulqva_privacy::{launch_prepared_arti, prepare_arti_runtime, ArtiRuntimePlan, RunningArti, TorSocksEndpoint};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs::{self, File, OpenOptions}, io::{Read, Write},
          path::Path, time::{SystemTime, UNIX_EPOCH}};

type Outcome<T> = Result<T, Box<dyn Error>>;
const QUERY: &str = "countdown";
const SELECTED_INDEX: usize = 1;

fn create_private_dir(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path) // Exclusive: never erase or reuse someone else's directory.
}

// Retain a byte-identical copy OUTSIDE the Tor runtime. The production verifier
// already ran; this independent acceptance check hashes the bytes actually saved.
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
    Ok(json!({"path":"selected.webm", "byte_size":size, "sha1":sha1, "sha256":sha256,
              "selected_index":receipt.selected_index(), "title":receipt.title()}))
}

fn journey(arti: &mut RunningArti, ytdlp: &Path, runtime: &Path, bundle: &Path) -> Outcome<Value> {
    eprintln!("PULQVA_COMMONS_STAGE readiness");
    let transport = CommonsHttpsTransport::new(arti).map_err(|error| {
        eprintln!("PULQVA_COMMONS_FAILURE stage=readiness category={error:?}"); error
    })?;
    let cancellation = transport.cancellation();
    let mut search = transport.into_search().with_ytdlp_executable(ytdlp);
    eprintln!("PULQVA_COMMONS_STAGE external_search");
    let choices = match request_choices(&mut search, QUERY) {
        Ok(choices) => choices,
        Err(error) => {
            eprintln!("PULQVA_COMMONS_FAILURE stage=external_search category={:?}", search.last_error());
            return Err(error.into());
        }
    };
    let selected = choices.select(SELECTED_INDEX)?;
    let media = search.last_results().get(SELECTED_INDEX).ok_or("selected metadata missing")?.clone();
    if choices.candidates().len() != search.last_results().len() || selected.candidate() != media.candidate() {
        return Err("presented choice disagrees with retained metadata".into());
    }
    let rows: Vec<Value> = search.last_results().iter().enumerate().map(|(index, item)| json!({
        "index":index, "page_id":item.page_id(), "title":item.candidate().title(),
        "locator":item.candidate().locator(), "declared_size":item.declared_size(),
        "declared_sha1":item.declared_sha1()
    })).collect();
    let downloads = runtime.join("downloads");
    create_private_dir(&downloads)?;
    eprintln!("PULQVA_COMMONS_STAGE selected_retrieval");
    // This is the SAME core retrieval boundary used by the application.
    let receipt = retrieve_choice(&mut search, &choices, SELECTED_INDEX, &downloads).map_err(|error| {
        eprintln!("PULQVA_COMMONS_FAILURE stage=selected_retrieval category={error:?}"); error
    })?;
    eprintln!("PULQVA_COMMONS_STAGE retained_file_verification");
    let file = retain_file(&receipt, &media, bundle)?;
    // Cancellation belongs AFTER retrieval: it deliberately clears retained data.
    cancellation.cancel();
    if !matches!(request_choices(&mut search, QUERY), Err(JourneyError::SearchFailed))
        || !search.last_results().is_empty() {
        return Err("cancelled search returned success or retained stale data".into());
    }
    Ok(json!({"schema":2, "scope":"linux-live-request-choice-file", "request":QUERY,
        "choice_count":rows.len(), "choices":rows, "selected_index":SELECTED_INDEX,
        "selected_locator":selected.candidate().locator(), "file":file,
        "core_retrieval_used":true, "file_downloaded":true,
        "cancelled_search_rejected":true, "stale_results_cleared":true,
        "publisher_authenticated":false, "windows_e2e_verified":false}))
}

fn run() -> Outcome<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 3 { return Err("expected pinned Arti, pinned yt-dlp, and NEW evidence directory".into()); }
    let arti_executable = fs::canonicalize(&args[0])?;
    let ytdlp = fs::canonicalize(&args[1])?;
    if !arti_executable.is_file() || !ytdlp.is_file() { return Err("pinned executable missing".into()); }
    let source_sha = env::var("PULQVA_SOURCE_SHA")?;
    if source_sha.len() != 40 || !source_sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid source SHA".into());
    }
    let bundle = Path::new(&args[2]);
    create_private_dir(bundle)?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let runtime = env::temp_dir().join(format!("pulqva-file-runtime-{}-{nonce}", std::process::id()));
    create_private_dir(&runtime)?;
    let outcome = (|| -> Outcome<Value> {
        let plan = ArtiRuntimePlan::new(arti_executable, runtime.join("config/pulqva.toml"),
            runtime.join("cache"), runtime.join("state"), TorSocksEndpoint::new(19050)?);
        let mut arti = launch_prepared_arti(prepare_arti_runtime(plan)?)?;
        let result = journey(&mut arti, &ytdlp, &runtime, bundle);
        // Always attempt to stop/reap on normal success/error; CI watchdog covers stalls.
        let stopped = arti.stop_and_wait();
        stopped?;
        result
    })();
    let cleanup = fs::remove_dir_all(&runtime);
    cleanup?;
    let mut evidence = outcome?;
    evidence["source_sha"] = json!(source_sha);
    let bytes = serde_json::to_vec_pretty(&evidence)?;
    if bytes.len() > 65_536 { return Err("evidence exceeded byte cap".into()); }
    let mut out = OpenOptions::new().write(true).create_new(true).open(bundle.join("receipt.json"))?;
    out.write_all(&bytes)?; out.write_all(b"\n")?; out.sync_all()?;
    println!("PULQVA_COMMONS_TOR_DISCOVERY_OK choices={}", evidence["choice_count"]);
    println!("PULQVA_COMMONS_FILE_E2E_OK");
    Ok(())
}

fn main() {
    if run().is_err() {
        // No raw paths, child stderr or request data in failure logs.
        eprintln!("PULQVA_COMMONS_FILE_E2E_FAILED");
        std::process::exit(1);
    }
}

// Encode the digest bytes, not the RustCrypto 0.11 Array wrapper.
// Same byte-to-text convention as the already tested artifact verifier;
// this is hexadecimal formatting, not a new hashing implementation.
fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{lower_hex, Digest, Sha1, Sha256};

    #[test]
    fn hex_is_fixed_width_lowercase_for_every_byte() {
        assert_eq!(lower_hex(&[]), "");
        for value in 0..=255u8 {
            assert_eq!(lower_hex(&[value]), format!("{value:02x}"));
        }
        assert_eq!(lower_hex(&[0, 1, 15, 16, 128, 255]), "00010f1080ff");
    }

    #[test]
    fn pinned_digest_outputs_encode_known_answers() {
        let mut sha1 = Sha1::new();
        sha1.update(b"abc");
        assert_eq!(lower_hex(sha1.finalize().as_ref()),
                   "a9993e364706816aba3e25717850c26c9cd0d89d");
        let mut sha256 = Sha256::new();
        sha256.update(b"abc");
        assert_eq!(lower_hex(sha256.finalize().as_ref()),
                   "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
