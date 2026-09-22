use std::{
    error::Error,
    fmt,
    io,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::{
    CompletedFfmpegRemuxResult, FfmpegRemuxArtifactError, FfmpegRemuxPlan,
    ffmpeg_completed::validate_completed_ffmpeg_remux,
};

/// Typed handle for a directly spawned FFmpeg child process.
#[derive(Debug)]
pub struct RunningFfmpeg {
    child: Child,
    input: PathBuf,
    output: PathBuf,
}

impl RunningFfmpeg {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn complete_remux(mut self) -> Result<CompletedFfmpegRemuxResult, FfmpegCompletionError> {
        let status = self
            .child
            .wait()
            .map_err(|source| FfmpegCompletionError::Io {
                operation: "wait for FFmpeg child completion",
                source,
            })?;

        if !status.success() {
            return Err(FfmpegCompletionError::ProcessFailed(status));
        }

        validate_completed_ffmpeg_remux(&self.input, &self.output)
            .map_err(FfmpegCompletionError::Artifact)
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
    let input = plan.input().to_owned();
    let output = plan.output().to_owned();
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

    Ok(RunningFfmpeg {
        child,
        input,
        output,
    })
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

#[derive(Debug)]
pub enum FfmpegCompletionError {
    ProcessFailed(ExitStatus),
    Artifact(FfmpegRemuxArtifactError),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for FfmpegCompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessFailed(status) => {
                write!(f, "FFmpeg child did not complete successfully: {status}")
            }
            Self::Artifact(source) => write!(f, "completed FFmpeg remux validation failed: {source}"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for FfmpegCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::ProcessFailed(_) => None,
        }
    }
}
