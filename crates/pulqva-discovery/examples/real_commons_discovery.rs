//! Native live acceptance harness. No alternate HTTP client or canned candidates.
//! A successful run proves discovery only, NOT a selected-file download or UI E2E.
use pulqva_core::{request_choices, JourneyError};
use pulqva_discovery::CommonsHttpsTransport;
use pulqva_privacy::{
    launch_prepared_arti, prepare_arti_runtime, ArtiRuntimePlan, RunningArti,
    TorSocksEndpoint,
};
use serde_json::{json, Value};
use std::{env, error::Error, fs, io::Write, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

const QUERY: &str = "countdown"; // Public, fixed full-text fixture, not an AI interpretation.
const SELECTED_INDEX: usize = 1; // Explicit test choice from the returned set, not a URL.

fn discover(arti: &mut RunningArti) -> Result<Value, Box<dyn Error>> {
    eprintln!("PULQVA_COMMONS_STAGE readiness");
    let transport = CommonsHttpsTransport::new(arti).map_err(|error| {
        // DiscoveryError contains only fixed categories and a numeric HTTP status.
        eprintln!("PULQVA_COMMONS_FAILURE stage=readiness category={error:?}");
        error
    })?;
    let cancellation = transport.cancellation();
    let mut search = transport.into_search();
    eprintln!("PULQVA_COMMONS_STAGE external_search");
    let choices = match request_choices(&mut search, QUERY) {
        Ok(choices) => choices,
        Err(error) => {
            eprintln!("PULQVA_COMMONS_FAILURE stage=external_search category={:?}", search.last_error());
            return Err(error.into());
        }
    };
    let selected = choices.select(SELECTED_INDEX)?;
    if choices.candidates().len() != search.last_results().len() {
        return Err("presented choices differ from discovered metadata".into());
    }
    let rows: Vec<Value> = search.last_results().iter().enumerate().map(|(index, media)| {
        json!({
            "index": index, "page_id": media.page_id(),
            "title": media.candidate().title(), "locator": media.candidate().locator(),
            "declared_size": media.declared_size(), "declared_sha1": media.declared_sha1()
        })
    }).collect();
    if rows.len() < 2 || selected.candidate() != search.last_results()[SELECTED_INDEX].candidate() {
        return Err("selection did not identify one of the real external choices".into());
    }
    // A second coordinator call after sticky cancellation must not reuse old results.
    // Existing executor checks cancellation before creating the request client.
    cancellation.cancel();
    if !matches!(request_choices(&mut search, QUERY), Err(JourneyError::SearchFailed))
        || !search.last_results().is_empty() {
        return Err("cancelled search returned success or retained stale choices".into());
    }
    Ok(json!({
        "schema": 1, "scope": "live Tor Commons discovery only",
        "request": QUERY, "choice_count": rows.len(), "choices": rows,
        "selected_index": SELECTED_INDEX, "selected_locator": selected.candidate().locator(),
        "cancelled_search_rejected": true, "stale_results_cleared": true,
        "file_downloaded": false, "hashes_are_server_declared_metadata": true
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("expected pinned Arti path and new evidence file path".into()); }
    let executable = PathBuf::from(&args[0]);
    let evidence = PathBuf::from(&args[1]);
    if !executable.is_file() { return Err("pinned Arti executable is missing".into()); }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("pulqva-commons-live-{}-{nonce}", std::process::id()));
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    // Exclusive creation: never delete a pre-existing directory to make a test pass.
    builder.create(&root)?;
    let result = (|| -> Result<Value, Box<dyn Error>> {
        let plan = ArtiRuntimePlan::new(
            executable, root.join("config/pulqva.toml"), root.join("cache"),
            root.join("state"), TorSocksEndpoint::new(19050)?,
        );
        let mut arti = launch_prepared_arti(prepare_arti_runtime(plan)?)?;
        let discovered = discover(&mut arti);
        // Always stop/reap the owned process, including readiness/search errors.
        let stopped = arti.stop_and_wait();
        stopped?;
        discovered
    })();
    let cleanup = fs::remove_dir_all(&root);
    // Neither platform may convert a readiness timeout into a positive live result.
    cleanup?;
    let result = result?;
    let bytes = serde_json::to_vec_pretty(&result)?;
    if bytes.len() > 65_536 { return Err("public fixture evidence exceeded its byte cap".into()); }
    let mut out = fs::OpenOptions::new().write(true).create_new(true).open(evidence)?;
    out.write_all(&bytes)?;
    out.write_all(b"\n")?;
    out.sync_all()?;
    println!("PULQVA_COMMONS_TOR_DISCOVERY_OK choices={}", result["choice_count"]);
    Ok(())
}
