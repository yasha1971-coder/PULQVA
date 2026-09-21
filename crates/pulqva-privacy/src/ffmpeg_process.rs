use std::{
    error::Error,
    fmt,
    io,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::FfmpegRemuxPlan;

/// Typed handle for a directly spawned FFmpeg child process.
#[derive(Debug)]
pub struct RunningFfmpeg {
    child: Child,
}

impl RunningFfmpeg {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, FfmpegProcessError> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|source| FfmpegProcessError::Io {
                operation: "query FFmpeg child status",
                source,
            })?
        {
            return Ok(status);
        }

        self.child
            .kill()
            .map_err(|source| FfmpegProcessError::Io {
                operation: "terminate FFmpeg child",
                source,
            })?;

        self.child
            .wait()
            .map_err(|source| FfmpegProcessError::Io {
                operation: "wait for FFmpeg child",
                source,
            })
    }
}

/// Spawns only a fully typed local FFmpeg remux plan.
///
/// The executable and argv are copied exclusively from `FfmpegRemuxPlan`.
/// No shell, URL, proxy argument, or fallback route is added here.
pub fn launch_ffmpeg_remux(
    plan: FfmpegRemuxPlan,
) -> Result<RunningFfmpeg, FfmpegProcessError> {
    let executable = plan.executable().to_owned();
    let arguments = plan.arguments();

    let child = Command::new(&executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| FfmpegProcessError::Io {
            operation: "spawn typed FFmpeg child",
            source,
        })?;

    Ok(RunningFfmpeg { child })
}

#[derive(Debug)]
pub enum FfmpegProcessError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for FfmpegProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for FfmpegProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}
