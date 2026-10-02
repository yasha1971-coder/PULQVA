use std::{
    error::Error,
    ffi::OsString,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    ReadyTorTransport, TorSocksEndpoint,
    arti_ready::certify_tor_ready,
    tor_socks_probe::{ATTEMPT_SLICE, RETRY_DELAY, SocksProbeError, socks_connect_probe},
};

const READINESS_HOST: &str = "example.com";
const READINESS_PORT: u16 = 443;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Pure-data plan for the pinned little-t Tor sidecar.
///
/// The plan does not perform I/O, spawn a process, or open a socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LittleTorRuntimePlan {
    executable: PathBuf,
    config_file: PathBuf,
    data_dir: PathBuf,
    socks: TorSocksEndpoint,
}

impl LittleTorRuntimePlan {
    pub fn new(
        executable: impl Into<PathBuf>,
        config_file: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        socks: TorSocksEndpoint,
    ) -> Self {
        Self {
            executable: executable.into(),
            config_file: config_file.into(),
            data_dir: data_dir.into(),
            socks,
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn socks_endpoint(&self) -> TorSocksEndpoint {
        self.socks
    }

    pub fn launch_arguments(&self) -> Vec<OsString> {
        vec![
            OsString::from("-f"),
            self.config_file.as_os_str().to_os_string(),
        ]
    }

    pub fn render_config(&self) -> Result<String, LittleTorConfigRenderError> {
        let data_dir = torrc_path(self.data_dir())?;

        Ok(format!(
            "ClientOnly 1\n\
SocksPort 127.0.0.1:{}\n\
DNSPort 0\n\
DataDirectory \"{}\"\n\
AvoidDiskWrites 1\n\
SafeLogging 1\n",
            self.socks.port(),
            data_dir,
        ))
    }
}

fn torrc_path(path: &Path) -> Result<String, LittleTorConfigRenderError> {
    let raw = path
        .to_str()
        .ok_or(LittleTorConfigRenderError::NonUtf8DataDirectory)?;

    if raw.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r')) {
        return Err(LittleTorConfigRenderError::UnsafeDataDirectoryCharacter);
    }

    Ok(raw.replace('\\', "/").replace('"', "\\\""))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LittleTorConfigRenderError {
    NonUtf8DataDirectory,
    UnsafeDataDirectoryCharacter,
}

impl fmt::Display for LittleTorConfigRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUtf8DataDirectory => {
                f.write_str("little-t Tor DataDirectory must be valid UTF-8")
            }
            Self::UnsafeDataDirectoryCharacter => {
                f.write_str("little-t Tor DataDirectory contains an unsafe control character")
            }
        }
    }
}

impl Error for LittleTorConfigRenderError {}

/// Capability proving that PULQVA's deterministic little-t Tor config and
/// isolated data directory were prepared before process launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedLittleTorRuntime {
    plan: LittleTorRuntimePlan,
}

impl PreparedLittleTorRuntime {
    pub fn plan(&self) -> &LittleTorRuntimePlan {
        &self.plan
    }

    pub fn socks_endpoint(&self) -> TorSocksEndpoint {
        self.plan.socks_endpoint()
    }
}

pub fn prepare_little_tor_runtime(
    plan: LittleTorRuntimePlan,
) -> Result<PreparedLittleTorRuntime, LittleTorPrepareError> {
    let rendered = plan
        .render_config()
        .map_err(LittleTorPrepareError::Render)?;

    prepare_real_directory(plan.data_dir(), "prepare little-t Tor DataDirectory")?;

    let config_parent = plan.config_file().parent().unwrap_or_else(|| Path::new(""));
    if !config_parent.as_os_str().is_empty() {
        prepare_real_directory(config_parent, "prepare little-t Tor config directory")?;
    }

    materialize_owned_config(plan.config_file(), rendered.as_bytes())?;

    Ok(PreparedLittleTorRuntime { plan })
}

fn prepare_real_directory(
    path: &Path,
    operation: &'static str,
) -> Result<(), LittleTorPrepareError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(LittleTorPrepareError::UnsafeDirectory {
                    path: path.to_path_buf(),
                });
            }
            Ok(())
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(path).map_err(|source| LittleTorPrepareError::Io {
                operation,
                source,
            })
        }
        Err(source) => Err(LittleTorPrepareError::Io {
            operation,
            source,
        }),
    }
}

fn materialize_owned_config(
    destination: &Path,
    expected: &[u8],
) -> Result<(), LittleTorPrepareError> {
    if destination.file_name().is_none() {
        return Err(LittleTorPrepareError::MissingConfigFileName);
    }

    match fs::symlink_metadata(destination) {
        Ok(metadata) => return verify_existing_config(destination, &metadata, expected),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(LittleTorPrepareError::Io {
                operation: "inspect little-t Tor config",
                source,
            });
        }
    }

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
    {
        Ok(mut file) => {
            if let Err(source) = file
                .write_all(expected)
                .and_then(|_| file.flush())
                .and_then(|_| file.sync_all())
            {
                drop(file);
                let _ = fs::remove_file(destination);
                return Err(LittleTorPrepareError::Io {
                    operation: "write little-t Tor config",
                    source,
                });
            }
            Ok(())
        }
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            let metadata =
                fs::symlink_metadata(destination).map_err(|source| LittleTorPrepareError::Io {
                    operation: "inspect raced little-t Tor config",
                    source,
                })?;
            verify_existing_config(destination, &metadata, expected)
        }
        Err(source) => Err(LittleTorPrepareError::Io {
            operation: "create little-t Tor config",
            source,
        }),
    }
}

fn verify_existing_config(
    destination: &Path,
    metadata: &fs::Metadata,
    expected: &[u8],
) -> Result<(), LittleTorPrepareError> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(LittleTorPrepareError::UnsafeExistingConfig);
    }

    let actual = fs::read(destination).map_err(|source| LittleTorPrepareError::Io {
        operation: "read existing little-t Tor config",
        source,
    })?;

    if actual != expected {
        return Err(LittleTorPrepareError::ExistingConfigMismatch);
    }

    Ok(())
}

#[derive(Debug)]
pub enum LittleTorPrepareError {
    Render(LittleTorConfigRenderError),
    MissingConfigFileName,
    UnsafeDirectory { path: PathBuf },
    UnsafeExistingConfig,
    ExistingConfigMismatch,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for LittleTorPrepareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Render(source) => write!(f, "render little-t Tor config: {source}"),
            Self::MissingConfigFileName => {
                f.write_str("little-t Tor config path must include a file name")
            }
            Self::UnsafeDirectory { path } => write!(
                f,
                "little-t Tor runtime directory is not a real directory: {}",
                path.display()
            ),
            Self::UnsafeExistingConfig => {
                f.write_str("existing little-t Tor config must be a regular non-symlink file")
            }
            Self::ExistingConfigMismatch => {
                f.write_str("existing little-t Tor config differs from the validated plan")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for LittleTorPrepareError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Render(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::MissingConfigFileName
            | Self::UnsafeDirectory { .. }
            | Self::UnsafeExistingConfig
            | Self::ExistingConfigMismatch => None,
        }
    }
}

#[derive(Debug)]
pub struct RunningLittleTor {
    child: Child,
    plan: LittleTorRuntimePlan,
}

impl RunningLittleTor {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn endpoint(&self) -> TorSocksEndpoint {
        self.plan.socks_endpoint()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, LittleTorProcessError> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|source| LittleTorProcessError::Io {
                operation: "query little-t Tor child status",
                source,
            })?
        {
            return Ok(status);
        }

        self.child
            .kill()
            .map_err(|source| LittleTorProcessError::Io {
                operation: "terminate little-t Tor child",
                source,
            })?;

        self.child
            .wait()
            .map_err(|source| LittleTorProcessError::Io {
                operation: "wait for little-t Tor child",
                source,
            })
    }
}

pub fn launch_little_tor(
    prepared: PreparedLittleTorRuntime,
) -> Result<RunningLittleTor, LittleTorProcessError> {
    let plan = prepared.plan;
    let mut command = Command::new(plan.executable());
    command
        .args(plan.launch_arguments())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let child = command
        .spawn()
        .map_err(|source| LittleTorProcessError::Io {
            operation: "spawn prepared little-t Tor child",
            source,
        })?;

    Ok(RunningLittleTor { child, plan })
}

#[derive(Debug)]
pub enum LittleTorProcessError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for LittleTorProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for LittleTorProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}

/// Certifies a PULQVA-launched little-t Tor child into the same
/// ReadyTorTransport capability used by existing network adapters.
///
/// The verifier connects only to the local SOCKS endpoint and asks that proxy
/// to connect to a domain-name destination. There is no direct-network fallback.
pub fn verify_little_tor_readiness(
    running: &mut RunningLittleTor,
    timeout: Duration,
) -> Result<ReadyTorTransport, LittleTorReadinessError> {
    if timeout.is_zero() {
        return Err(LittleTorReadinessError::Timeout);
    }

    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(LittleTorReadinessError::Timeout)?;

    loop {
        if let Some(status) = running
            .try_wait()
            .map_err(|source| LittleTorReadinessError::Io {
                operation: "query little-t Tor child status",
                source,
            })?
        {
            return Err(LittleTorReadinessError::ChildExited(status));
        }

        let now = Instant::now();
        if now >= deadline {
            return Err(LittleTorReadinessError::Timeout);
        }

        let remaining = deadline.saturating_duration_since(now);
        let attempt_timeout = remaining.min(ATTEMPT_SLICE);

        match socks_connect_probe(
            running.endpoint(),
            READINESS_HOST,
            READINESS_PORT,
            attempt_timeout,
        ) {
            Ok(verified_loopback) => {
                return Ok(certify_tor_ready(
                    running.endpoint(),
                    verified_loopback,
                ));
            }
            Err(SocksProbeError::Retryable) => {
                let sleep_for = deadline
                    .saturating_duration_since(Instant::now())
                    .min(RETRY_DELAY);
                if sleep_for.is_zero() {
                    return Err(LittleTorReadinessError::Timeout);
                }
                thread::sleep(sleep_for);
            }
            Err(SocksProbeError::Protocol(message)) => {
                return Err(LittleTorReadinessError::Protocol(message));
            }
        }
    }
}

#[derive(Debug)]
pub enum LittleTorReadinessError {
    Timeout,
    ChildExited(ExitStatus),
    Protocol(&'static str),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for LittleTorReadinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("little-t Tor readiness verification timed out"),
            Self::ChildExited(status) => {
                write!(f, "little-t Tor child exited before Tor became ready: {status}")
            }
            Self::Protocol(message) => {
                write!(f, "little-t Tor SOCKS readiness protocol error: {message}")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for LittleTorReadinessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Timeout | Self::ChildExited(_) | Self::Protocol(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LittleTorPrepareError, LittleTorRuntimePlan, launch_little_tor,
        prepare_little_tor_runtime,
    };
    use crate::TorSocksEndpoint;
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-little-t-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    fn canonical_plan(root: &Path) -> LittleTorRuntimePlan {
        LittleTorRuntimePlan::new(
            root.join("tor/tor.exe"),
            root.join("config/torrc"),
            root.join("runtime/data"),
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
        )
    }

    #[test]
    fn canonical_relative_config_is_byte_stable() {
        let plan = LittleTorRuntimePlan::new(
            "runtime/tor/tor.exe",
            "runtime/config/torrc",
            "runtime/data",
            TorSocksEndpoint::new(19050).expect("canonical SOCKS port is non-zero"),
        );

        let rendered = plan.render_config().expect("canonical config renders");
        let fixture = include_str!("../../../sidecars/tor/pulqva.torrc");

        assert_eq!(rendered.as_bytes(), fixture.as_bytes());
    }

    #[test]
    fn launch_arguments_force_the_owned_torrc() {
        let plan = LittleTorRuntimePlan::new(
            "runtime/tor/tor.exe",
            "runtime/config/torrc",
            "runtime/data",
            TorSocksEndpoint::new(19050).expect("canonical SOCKS port is non-zero"),
        );

        assert_eq!(
            plan.launch_arguments(),
            vec![
                OsString::from("-f"),
                OsString::from("runtime/config/torrc"),
            ]
        );
    }

    #[test]
    fn preparation_creates_owned_data_and_exact_config_and_reuses_it() {
        let root = test_root("prepare");
        let plan = canonical_plan(&root);
        let expected = plan.render_config().expect("test config renders");

        let first = prepare_little_tor_runtime(plan.clone()).expect("first preparation succeeds");
        assert_eq!(first.socks_endpoint().port(), 19050);
        assert!(plan.data_dir().is_dir());
        assert_eq!(
            fs::read(plan.config_file()).expect("config exists"),
            expected.as_bytes()
        );

        prepare_little_tor_runtime(plan.clone()).expect("matching config is reusable");

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn preparation_fails_closed_on_existing_config_drift() {
        let root = test_root("config-drift");
        let plan = canonical_plan(&root);
        fs::create_dir_all(plan.config_file().parent().unwrap()).expect("config parent");
        fs::write(plan.config_file(), b"untrusted config").expect("stale config");

        let error = prepare_little_tor_runtime(plan).expect_err("drift must fail");
        assert!(matches!(error, LittleTorPrepareError::ExistingConfigMismatch));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn missing_executable_fails_before_any_network_readiness() {
        let root = test_root("missing-executable");
        let plan = canonical_plan(&root);
        let prepared = prepare_little_tor_runtime(plan).expect("preparation succeeds");

        let error = launch_little_tor(prepared).expect_err("missing executable must fail");
        assert!(error.to_string().contains("spawn prepared little-t Tor child"));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }
}
