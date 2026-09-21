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
    plan: ArtiRuntimePlan,
}

impl RunningArti {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn endpoint(&self) -> TorSocksEndpoint {
        self.plan.socks_endpoint()
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
        let rendered = self
            .plan
            .render_bootstrap_config()
            .map_err(ArtiProcessError::RenderConfig)?;

        materialize_arti_config_bytes(self.plan.config_file(), rendered.as_bytes())
            .map_err(ArtiProcessError::MaterializeConfig)?;

        let _ = self.stop_child()?;
        self.child = spawn_plan(&self.plan)?;
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
    let plan = prepared.plan().clone();
    let child = spawn_plan(&plan)?;

    Ok(RunningArti { child, plan })
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
