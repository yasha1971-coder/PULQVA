use std::{
    error::Error,
    fmt,
    io,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::{
    CompletedDownloadResult, CompletedMediaArtifactReceipt, MediaArtifactReceiptError,
    YtDlpMediaRequestPlan, YtDlpMediaSourceUrl,
    ytdlp_artifact::validate_completed_media_artifact,
};

/// Typed handle for a directly spawned yt-dlp child process.
///
/// The process is spawned without a shell and is owned until explicit shutdown.
#[derive(Debug)]
pub struct RunningYtDlp {
    child: Child,
    output_root: PathBuf,
    source: YtDlpMediaSourceUrl,
}

impl RunningYtDlp {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn complete(mut self) -> Result<CompletedMediaArtifactReceipt, YtDlpCompletionError> {
        let status = self
            .child
            .wait()
            .map_err(|source| YtDlpCompletionError::Io {
                operation: "wait for yt-dlp child completion",
                source,
            })?;

        if !status.success() {
            return Err(YtDlpCompletionError::ProcessFailed(status));
        }

        validate_completed_media_artifact(&self.output_root)
            .map_err(YtDlpCompletionError::Artifact)
    }

    pub fn complete_download(mut self) -> Result<CompletedDownloadResult, YtDlpCompletionError> {
        let status = self
            .child
            .wait()
            .map_err(|source| YtDlpCompletionError::Io {
                operation: "wait for yt-dlp child completion",
                source,
            })?;

        if !status.success() {
            return Err(YtDlpCompletionError::ProcessFailed(status));
        }

        let artifact = validate_completed_media_artifact(&self.output_root)
            .map_err(YtDlpCompletionError::Artifact)?;

        Ok(CompletedDownloadResult::from_validated_completion(
            self.source,
            artifact,
        ))
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, YtDlpProcessError> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|source| YtDlpProcessError::Io {
                operation: "query yt-dlp child status",
                source,
            })?
        {
            return Ok(status);
        }

        self.child
            .kill()
            .map_err(|source| YtDlpProcessError::Io {
                operation: "terminate yt-dlp child",
                source,
            })?;

        self.child
            .wait()
            .map_err(|source| YtDlpProcessError::Io {
                operation: "wait for yt-dlp child",
                source,
            })
    }
}

/// Spawns only a fully typed yt-dlp media request.
///
/// Executable and argv come exclusively from YtDlpMediaRequestPlan. No shell
/// is involved and this function adds no fallback route or ambient arguments.
pub fn launch_ytdlp_request(
    request: YtDlpMediaRequestPlan,
) -> Result<RunningYtDlp, YtDlpProcessError> {
    let executable = request.executable().to_owned();
    let arguments = request.arguments();
    let output_root = request.output_root().to_owned();
    let source = request.source().clone();

    let child = Command::new(&executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| YtDlpProcessError::Io {
            operation: "spawn typed yt-dlp child",
            source,
        })?;

    Ok(RunningYtDlp {
        child,
        output_root,
        source,
    })
}

#[derive(Debug)]
pub enum YtDlpProcessError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for YtDlpProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for YtDlpProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum YtDlpCompletionError {
    ProcessFailed(ExitStatus),
    Artifact(MediaArtifactReceiptError),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for YtDlpCompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessFailed(status) => {
                write!(f, "yt-dlp child did not complete successfully: {status}")
            }
            Self::Artifact(source) => write!(f, "completed media artifact validation failed: {source}"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for YtDlpCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::ProcessFailed(_) => None,
        }
    }
}
