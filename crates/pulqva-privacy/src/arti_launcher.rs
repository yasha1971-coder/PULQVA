use std::{
    error::Error,
    fmt,
    io,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::PreparedArtiRuntime;

/// Typed handle for a directly spawned Arti child process.
///
/// The child is launched without a shell. This type owns the process handle so
/// shutdown can be explicit and deterministic.
#[derive(Debug)]
pub struct RunningArti {
    child: Child,
}

impl RunningArti {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, ArtiProcessError> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|source| ArtiProcessError::Io {
                operation: "query Arti child status",
                source,
            })?
        {
            return Ok(status);
        }

        self.child
            .kill()
            .map_err(|source| ArtiProcessError::Io {
                operation: "terminate Arti child",
                source,
            })?;

        self.child
            .wait()
            .map_err(|source| ArtiProcessError::Io {
                operation: "wait for Arti child",
                source,
            })
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
    let executable = prepared.plan().executable().to_owned();
    let arguments = prepared.launch_arguments();

    let child = Command::new(&executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| ArtiProcessError::Io {
            operation: "spawn prepared Arti child",
            source,
        })?;

    Ok(RunningArti { child })
}

#[derive(Debug)]
pub enum ArtiProcessError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for ArtiProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for ArtiProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}
