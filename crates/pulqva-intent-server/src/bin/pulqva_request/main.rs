//! User-driven Linux backend entry, not the packaged desktop release.
//! Trusted operator supplies already-verified pinned components. The human request
//! and selected index arrive on stdin, never as shell text or executable paths.
//! No live invocation belongs in ordinary CI; model/Tor acceptance is separate.
mod dialogue;

use pulqva_discovery::CommonsHttpsTransport;
use pulqva_intent_server::{IntentServer, IntentServerPlan};
use pulqva_privacy::{launch_prepared_arti, prepare_arti_runtime, ArtiRuntimePlan,
    RunningArti, TorSocksEndpoint};
use std::{env, ffi::OsString, fs, io::{self, Write}, path::{Component, Path, PathBuf},
    process::ExitCode, time::Duration};

const SCHEMA: &str = include_str!("../../../../../sidecars/llama.cpp/intent.schema.json");
const USAGE: &str = "Development backend (Linux; not the consumer package)\n\
Usage: pulqva_request <pinned-llama-server> <pinned-model.gguf> <pinned-arti> <pinned-yt-dlp> <NEW-output-dir> <NEW-runtime-dir>\n\
Enter a request, then select one displayed number or q to cancel.\n\
Components must already be verified against the project's pins. No automatic installs.\n\
Windows official-Tor integration and the no-terminal GUI remain separate work.";

struct Config { server: PathBuf, model: PathBuf, arti: PathBuf, ytdlp: PathBuf,
    output: PathBuf, runtime: PathBuf }

fn existing_file(path: &Path) -> Result<PathBuf, &'static str> {
    if !path.is_absolute() { return Err("component path must be absolute"); }
    let path = fs::canonicalize(path).map_err(|_| "component is unavailable")?;
    if !path.is_file() { return Err("component is not a regular file"); }
    Ok(path)
}

fn new_directory_path(path: &Path) -> Result<PathBuf, &'static str> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("output and runtime must be absolute, traversal-free paths");
    }
    let name = path.file_name().ok_or("directory name is missing")?;
    let parent = fs::canonicalize(path.parent().ok_or("directory parent is missing")?)
        .map_err(|_| "directory parent is unavailable")?;
    if !parent.is_dir() { return Err("directory parent is not a directory"); }
    let resolved = parent.join(name);
    match fs::symlink_metadata(&resolved) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(resolved),
        _ => Err("output/runtime must be new; existing paths are never reused"),
    }
}

fn config(args: &[OsString]) -> Result<Config, &'static str> {
    if args.len() != 6 { return Err("expected exactly six trusted setup paths; use --help"); }
    let output = new_directory_path(Path::new(&args[4]))?;
    let runtime = new_directory_path(Path::new(&args[5]))?;
    if output.starts_with(&runtime) || runtime.starts_with(&output) {
        return Err("output must be outside the disposable runtime");
    }
    Ok(Config { server: existing_file(Path::new(&args[0]))?,
        model: existing_file(Path::new(&args[1]))?, arti: existing_file(Path::new(&args[2]))?,
        ytdlp: existing_file(Path::new(&args[3]))?, output, runtime })
}

// RunningArti has an explicit stop method rather than Drop. This caller owns
// that cleanup on early errors too; it does not introduce another process launcher.
struct ArtiOwner(Option<RunningArti>);
impl ArtiOwner {
    fn stop(&mut self) -> Result<(), &'static str> {
        if let Some(child) = self.0.as_mut() {
            child.stop_and_wait().map_err(|_| "Tor cleanup failed; runtime retained")?;
        }
        self.0 = None;
        Ok(())
    }
}
impl Drop for ArtiOwner { fn drop(&mut self) { let _ = self.stop(); } }

fn run(args: &[OsString]) -> Result<(), &'static str> {
    // Linux Arti is the existing owned transport. Never reinterpret a Windows
    // Arti failure as success or migrate Windows away from its official-Tor ADR.
    if !cfg!(target_os = "linux") { return Err("this development caller currently supports Linux only"); }
    let cfg = config(args)?;
    let stdin = io::stdin(); let stdout = io::stdout();
    let mut input = stdin.lock(); let mut output = stdout.lock();
    writeln!(output, "Describe the file you need:").map_err(|_| "presentation failed")?;
    output.flush().map_err(|_| "presentation failed")?;
    let request = dialogue::read_request(&mut input)?;
    // Request validation and trusted path validation precede all child/network work.
    dialogue::create_private_dir(&cfg.runtime).map_err(|_| "runtime creation failed")?;
    let mut arti = ArtiOwner(None);
    let outcome = (|| {
        let plan = IntentServerPlan::new(&cfg.server, &cfg.model, SCHEMA, 19081, Duration::from_secs(120))
            .map_err(|_| "invalid model runtime plan")?;
        let mut server = IntentServer::spawn(&plan).map_err(|_| "local model startup failed")?;
        let plan = ArtiRuntimePlan::new(&cfg.arti, cfg.runtime.join("config/pulqva.toml"),
            cfg.runtime.join("cache"), cfg.runtime.join("state"),
            TorSocksEndpoint::new(19050).map_err(|_| "invalid Tor plan")?);
        let prepared = prepare_arti_runtime(plan).map_err(|_| "Tor preparation failed")?;
        arti.0 = Some(launch_prepared_arti(prepared).map_err(|_| "Tor startup failed")?);
        let transport = CommonsHttpsTransport::new(arti.0.as_mut().ok_or("Tor child missing")?)
            .map_err(|_| "Tor readiness failed")?;
        let mut backend = transport.into_search().with_ytdlp_executable(&cfg.ytdlp);
        // CONCRETE production connection: same owned model + same real Commons
        // backend from interpretation through explicit selection and verification.
        let pending = server.request_choices(&request, &mut backend, Duration::from_secs(60))
            .map_err(|error| match error {
                pulqva_intent_http::LoopbackJourneyError::Rejected => "request rejected",
                pulqva_intent_http::LoopbackJourneyError::Search(_) => "Tor search failed",
                _ => "local interpretation failed",
            })?;
        dialogue::present_and_select(pending, &mut input, &mut output, &cfg.output)
        // backend and server drop here, before stopping Tor and removing runtime.
    })();
    arti.stop()?;
    fs::remove_dir_all(&cfg.runtime).map_err(|_| "runtime cleanup failed; output was not removed")?;
    match outcome? {
        Some(receipt) => writeln!(output, "Saved {} bytes to {:?}", receipt.byte_size(), receipt.path())
            .map_err(|_| "file saved but result presentation failed")?,
        None => writeln!(output, "Cancelled; no download started.").map_err(|_| "presentation failed")?,
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" { println!("{USAGE}"); return ExitCode::SUCCESS; }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(category) => { eprintln!("PULQVA: {category}"); ExitCode::FAILURE }
    }
}
