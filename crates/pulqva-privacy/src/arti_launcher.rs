use std::{
    error::Error,
    fmt,
    io,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::{
    ArtiConfigMaterializeError, ArtiConfigRenderError, ArtiRuntimePlan,
    PreparedArtiRuntime, TorSocksEndpoint,
    arti_materialize::materialize_arti_config_bytes,
};

/// Typed handle for a directly spawned Arti child process.
///
/// The child is launched without a shell. This type owns the process handle and
/// immutable runtime plan so privacy-layer lifecycle transitions can remain
/// deterministic.
#[derive(Debug)]
pub struct RunningArti {
    child: Child,
    plan: OwnedTorPlan,
    diagnostics: Option<std::sync::mpsc::Receiver<(&'static str, Option<&'static str>)>>,
}

#[derive(Debug)]
enum OwnedTorPlan { Arti(ArtiRuntimePlan), Official(TorSocksEndpoint) }

impl RunningArti {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn endpoint(&self) -> TorSocksEndpoint {
        match &self.plan { OwnedTorPlan::Arti(plan) => plan.socks_endpoint(), OwnedTorPlan::Official(endpoint) => *endpoint }
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    /// Explicitly transitions a deferred Arti child into its bootstrapping
    /// lifecycle.
    ///
    /// Arti 2.6.0 documents that `application.defer_bootstrap = true` keeps
    /// the client offline until explicitly told to bootstrap. PULQVA therefore
    /// rewrites only its validated config to `false`, stops the still-deferred
    /// child, and restarts the same executable/arguments. No shell or direct
    /// fallback is involved.
    pub(crate) fn activate_bootstrap(&mut self) -> Result<(), ArtiProcessError> {
        let plan = match &self.plan {
            OwnedTorPlan::Arti(plan) => plan.clone(),
            OwnedTorPlan::Official(_) => return Ok(()),
        };
        let rendered = plan.render_bootstrap_config().map_err(ArtiProcessError::RenderConfig)?;
        materialize_arti_config_bytes(plan.config_file(), rendered.as_bytes())
            .map_err(ArtiProcessError::MaterializeConfig)?;
        let _ = self.stop_child()?;
        self.child = spawn_plan(&plan)?;
        Ok(())
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, ArtiProcessError> {
        self.stop_child()
    }

    fn stop_child(&mut self) -> Result<ExitStatus, ArtiProcessError> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|source| ArtiProcessError::Io {
                operation: "query Arti child status",
                source,
            })?
        {
            self.report_diagnostics();
            return Ok(status);
        }

        self.child
            .kill()
            .map_err(|source| ArtiProcessError::Io {
                operation: "terminate Arti child",
                source,
            })?;

        let status = self.child.wait().map_err(|source| ArtiProcessError::Io {
            operation: "wait for Arti child", source,
        })?;
        self.report_diagnostics();
        Ok(status)
    }
    fn report_diagnostics(&mut self) {
        if let Some(receiver) = self.diagnostics.take() {
            // At most 200ms after owned child stop; never wait for reader threads.
            for _ in 0..2 {
                match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok((stream, Some(category))) => eprintln!("PULQVA_TOR_CHILD_DIAG v=1 stream={stream} category={category}"),
                    Ok((_, None)) => {},
                    Err(_) => { eprintln!("PULQVA_TOR_CHILD_DIAG v=1 category=capture-incomplete"); break; }
                }
            }
        }
    }

}

/// Launches only a previously prepared Arti runtime.
///
/// The executable path and argument vector come exclusively from the prepared
/// runtime plan. No shell is involved and this function performs no readiness
/// probe, SOCKS request, DNS request, or other network operation.
pub fn launch_prepared_arti(
    prepared: PreparedArtiRuntime,
) -> Result<RunningArti, ArtiProcessError> {
    let plan = prepared.plan().clone();
    let child = spawn_plan(&plan)?;

    Ok(RunningArti { child, plan: OwnedTorPlan::Arti(plan), diagnostics: None })
}

fn spawn_plan(plan: &ArtiRuntimePlan) -> Result<Child, ArtiProcessError> {
    Command::new(plan.executable())
        .args(plan.launch_arguments())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| ArtiProcessError::Io {
            operation: "spawn prepared Arti child",
            source,
        })
}

#[derive(Debug)]
pub enum ArtiProcessError {
    RenderConfig(ArtiConfigRenderError),
    MaterializeConfig(ArtiConfigMaterializeError),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for ArtiProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RenderConfig(source) => {
                write!(f, "render bootstrap-enabled Arti config: {source}")
            }
            Self::MaterializeConfig(source) => {
                write!(f, "materialize bootstrap-enabled Arti config: {source}")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for ArtiProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RenderConfig(source) => Some(source),
            Self::MaterializeConfig(source) => Some(source),
            Self::Io { source, .. } => Some(source),
        }
    }
}

/// Keep the owned handle/readiness supervision shared; official Tor boots once,
/// while Arti retains its existing deferred activation protocol.
pub fn launch_prepared_official_tor(
    prepared: crate::PreparedOfficialTorRuntime,
) -> Result<RunningArti, ArtiProcessError> {
    let diagnostic = std::env::var_os("PULQVA_TOR_DIAGNOSTICS").as_deref() == Some(std::ffi::OsStr::new("1"));
    let mut command = Command::new(&prepared.executable);
    let directory = prepared.config.parent().ok_or_else(|| ArtiProcessError::Io {
        operation: "resolve prepared official Tor directory",
        source: io::Error::new(io::ErrorKind::InvalidInput, "missing runtime directory"),
    })?;
    command.arg("-f").arg(&prepared.config).current_dir(directory).stdin(Stdio::null());
    if diagnostic { command.stdout(Stdio::piped()).stderr(Stdio::piped()); }
    else { command.stdout(Stdio::null()).stderr(Stdio::null()); }
    let mut child = command.spawn().map_err(|source| ArtiProcessError::Io {
        operation: "spawn prepared official Tor", source,
    })?;
    let diagnostics = if diagnostic {
        let (tx, rx) = std::sync::mpsc::channel();
        let stdout = child.stdout.take().expect("configured stdout pipe");
        let stderr = child.stderr.take().expect("configured stderr pipe");
        if let Err(source) = diagnostic_reader("stdout", stdout, tx.clone())
            .and_then(|_| diagnostic_reader("stderr", stderr, tx)) {
            let _ = child.kill(); let _ = child.wait();
            return Err(ArtiProcessError::Io { operation: "start Tor diagnostic drain", source });
        }
        Some(rx)
    } else { None };
    Ok(RunningArti { child, plan: OwnedTorPlan::Official(prepared.endpoint), diagnostics })
}

#[cfg(test)]
mod official_lifecycle_tests {
    use super::*;
    #[test]
    fn repeated_readiness_activation_keeps_same_owned_official_child() {
        let child = Command::new(std::env::current_exe().unwrap()).arg("--list")
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let endpoint = TorSocksEndpoint::new(19050).unwrap();
        let mut running = RunningArti { child, plan: OwnedTorPlan::Official(endpoint), diagnostics: None };
        let pid = running.id();
        let first = running.activate_bootstrap();
        let second = running.activate_bootstrap();
        let same_child = running.id() == pid;
        let same_endpoint = running.endpoint() == endpoint;
        let stopped = running.stop_and_wait();
        assert!(first.is_ok() && second.is_ok());
        assert!(same_child && same_endpoint);
        assert!(stopped.is_ok());
    }
}


// Only fixed categories leave this boundary; all raw Tor bytes are discarded.
fn fatal_category(line: &[u8]) -> Option<&'static str> {
    let text = String::from_utf8_lossy(line).to_ascii_lowercase();
    if text.contains("failed to parse/validate config") { Some("config-parse") }
    else if text.contains("reading config failed") || text.contains("unable to open configuration") { Some("config-read") }
    else if text.contains("permission denied") || text.contains("access is denied") { Some("access-denied") }
    else if text.contains("address already in use") || text.contains("could not bind") { Some("listener-bind") }
    else if text.contains("[err]") { Some("fatal-unclassified") }
    else { None }
}
fn drain_diagnostic(mut input: impl std::io::Read) -> Option<&'static str> {
    let mut chunk = [0u8; 1024];
    let mut line = Vec::with_capacity(2048);
    let mut overflow = false;
    let mut first = None;
    loop {
        let count = match input.read(&mut chunk) { Ok(0) => break, Ok(n) => n, Err(_) => return first.or(Some("capture-read-error")) };
        for &byte in &chunk[..count] {
            if byte == b'\n' {
                if !overflow && first.is_none() { first = fatal_category(&line); }
                line.clear(); overflow = false;
            } else if line.len() < 2048 { line.push(byte); }
            else { overflow = true; }
        }
        // Keep draining after the first category so diagnostics cannot fill a pipe.
    }
    if first.is_none() && !overflow { first = fatal_category(&line); }
    first
}
fn diagnostic_reader<R: std::io::Read + Send + 'static>(
    stream: &'static str, input: R,
    sender: std::sync::mpsc::Sender<(&'static str, Option<&'static str>)>,
) -> io::Result<()> {
    std::thread::Builder::new().name(format!("pulqva-tor-{stream}")).spawn(move || {
        let category = drain_diagnostic(input);
        let _ = sender.send((stream, category));
    }).map(|_| ())
}
#[cfg(test)]
mod child_diagnostic_tests {
    use super::*;
    #[test]
    fn fixed_categories_discard_private_payload_and_keep_first_error() {
        for (text, expected) in [
            ("[warn] Failed to parse/validate config: PRIVATE_QUERY https://secret/ 192.0.2.1 cookie=TOKEN", "config-parse"),
            ("[err] Reading config failed: PRIVATE_PATH", "config-read"),
            ("[warn] Permission denied: PRIVATE_PATH", "access-denied"),
            ("[warn] Could not bind: 192.0.2.1", "listener-bind"),
            ("[err] PRIVATE_UNKNOWN", "fatal-unclassified"),
        ] {
            let input = format!("{text}\n[err] SECOND_PRIVATE\n");
            assert_eq!(drain_diagnostic(std::io::Cursor::new(input)), Some(expected));
            assert!(!expected.contains("PRIVATE") && !expected.contains("192.0.2.1"));
        }
    }
    #[test]
    fn ignores_normal_logs_and_discards_overlong_lines_without_unbounded_memory() {
        assert_eq!(drain_diagnostic(std::io::Cursor::new(b"[notice] Bootstrapped 100%\n")), None);
        let mut bytes = vec![b'x'; 65536]; bytes.extend_from_slice(b"[err] secret\n[err] Reading config failed\n");
        assert_eq!(drain_diagnostic(std::io::Cursor::new(bytes)), Some("config-read"));
    }
}
