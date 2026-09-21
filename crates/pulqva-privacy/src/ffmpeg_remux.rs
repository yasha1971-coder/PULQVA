use std::{
    error::Error,
    ffi::{OsStr, OsString},
    fmt,
    path::{Component, Path, PathBuf},
};

use crate::CompletedDownloadResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfmpegRemuxContainer {
    Mp4,
    Matroska,
}

impl FfmpegRemuxContainer {
    fn format_name(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Matroska => "matroska",
        }
    }
}

/// Pure-data local FFmpeg remux plan.
///
/// The input path is copied only from a validated CompletedDownloadResult.
/// This type never spawns FFmpeg and has no network/proxy input field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfmpegRemuxPlan {
    executable: PathBuf,
    input: PathBuf,
    output: PathBuf,
    container: FfmpegRemuxContainer,
}

impl FfmpegRemuxPlan {
    pub fn new(
        executable: impl Into<PathBuf>,
        completed: &CompletedDownloadResult,
        output: impl Into<PathBuf>,
        container: FfmpegRemuxContainer,
    ) -> Result<Self, FfmpegRemuxPlanError> {
        let executable = executable.into();
        let output = output.into();

        if executable.as_os_str().is_empty() {
            return Err(FfmpegRemuxPlanError::MissingExecutable);
        }

        if output.as_os_str().is_empty() {
            return Err(FfmpegRemuxPlanError::MissingOutputPath);
        }

        if output
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(FfmpegRemuxPlanError::OutputPathTraversal);
        }

        let input = completed.artifact_path().to_owned();

        if output == input {
            return Err(FfmpegRemuxPlanError::OutputEqualsInput);
        }

        Ok(Self {
            executable,
            input,
            output,
            container,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn input(&self) -> &Path {
        &self.input
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn container(&self) -> FfmpegRemuxContainer {
        self.container
    }

    /// Deterministic argv for a future directly-spawned FFmpeg process.
    ///
    /// The protocol whitelist intentionally permits only local file access.
    /// Stream codecs are copied, so this plan is remux-only.
    pub fn arguments(&self) -> Vec<OsString> {
        vec![
            OsString::from("-hide_banner"),
            OsString::from("-loglevel"),
            OsString::from("error"),
            OsString::from("-nostdin"),
            OsString::from("-y"),
            OsString::from("-protocol_whitelist"),
            OsString::from("file"),
            OsString::from("-i"),
            self.input.as_os_str().to_os_string(),
            OsString::from("-map"),
            OsString::from("0"),
            OsString::from("-c"),
            OsString::from("copy"),
            OsString::from("-f"),
            OsString::from(self.container.format_name()),
            self.output.as_os_str().to_os_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfmpegRemuxPlanError {
    MissingExecutable,
    MissingOutputPath,
    OutputPathTraversal,
    OutputEqualsInput,
}

impl fmt::Display for FfmpegRemuxPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExecutable => f.write_str("FFmpeg executable path must not be empty"),
            Self::MissingOutputPath => f.write_str("FFmpeg remux output path must not be empty"),
            Self::OutputPathTraversal => {
                f.write_str("FFmpeg remux output path must not contain parent-directory traversal")
            }
            Self::OutputEqualsInput => {
                f.write_str("FFmpeg remux output path must differ from the validated input artifact")
            }
        }
    }
}

impl Error for FfmpegRemuxPlanError {}

#[cfg(test)]
mod tests {
    use super::{FfmpegRemuxContainer, FfmpegRemuxPlan, FfmpegRemuxPlanError};
    use crate::{
        CompletedDownloadResult, YtDlpMediaSourceUrl,
        ytdlp_artifact::validate_completed_media_artifact,
    };
    use std::{
        ffi::OsString,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn completed_download(label: &str) -> (PathBuf, CompletedDownloadResult) {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "pulqva-ffmpeg-remux-{label}-{}-{sequence}",
            std::process::id()
        ));

        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::write(root.join("input.bin"), b"PULQVA").expect("artifact write succeeds");

        let source =
            YtDlpMediaSourceUrl::parse("https://example.invalid/media").expect("valid source");
        let receipt =
            validate_completed_media_artifact(&root).expect("artifact receipt validates");
        let completed = CompletedDownloadResult::from_validated_completion(source, receipt);

        (root, completed)
    }

    #[test]
    fn input_can_only_come_from_completed_download_result_and_argv_is_deterministic() {
        let (root, completed) = completed_download("argv");
        let output = root.join("remux.mp4");

        let plan = FfmpegRemuxPlan::new(
            "runtime/ffmpeg",
            &completed,
            &output,
            FfmpegRemuxContainer::Mp4,
        )
        .expect("plan is valid");

        assert_eq!(plan.input(), completed.artifact_path());
        assert_eq!(plan.output(), output);
        assert_eq!(plan.container(), FfmpegRemuxContainer::Mp4);
        assert_eq!(
            plan.arguments(),
            vec![
                OsString::from("-hide_banner"),
                OsString::from("-loglevel"),
                OsString::from("error"),
                OsString::from("-nostdin"),
                OsString::from("-y"),
                OsString::from("-protocol_whitelist"),
                OsString::from("file"),
                OsString::from("-i"),
                completed.artifact_path().as_os_str().to_os_string(),
                OsString::from("-map"),
                OsString::from("0"),
                OsString::from("-c"),
                OsString::from("copy"),
                OsString::from("-f"),
                OsString::from("mp4"),
                output.as_os_str().to_os_string(),
            ]
        );

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn matroska_is_a_typed_supported_container() {
        let (root, completed) = completed_download("mkv");
        let output = root.join("remux.mkv");

        let plan = FfmpegRemuxPlan::new(
            "runtime/ffmpeg",
            &completed,
            &output,
            FfmpegRemuxContainer::Matroska,
        )
        .expect("plan is valid");

        let arguments = plan.arguments();
        let format_index = arguments
            .iter()
            .position(|arg| arg == OsStr::new("-f"))
            .expect("format flag exists");

        assert_eq!(arguments[format_index + 1], OsString::from("matroska"));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_missing_executable_and_output() {
        let (root, completed) = completed_download("missing");

        assert_eq!(
            FfmpegRemuxPlan::new(
                "",
                &completed,
                root.join("out.mp4"),
                FfmpegRemuxContainer::Mp4,
            )
            .unwrap_err(),
            FfmpegRemuxPlanError::MissingExecutable
        );

        assert_eq!(
            FfmpegRemuxPlan::new(
                "runtime/ffmpeg",
                &completed,
                "",
                FfmpegRemuxContainer::Mp4,
            )
            .unwrap_err(),
            FfmpegRemuxPlanError::MissingOutputPath
        );

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_output_equal_to_validated_input() {
        let (root, completed) = completed_download("same");

        assert_eq!(
            FfmpegRemuxPlan::new(
                "runtime/ffmpeg",
                &completed,
                completed.artifact_path(),
                FfmpegRemuxContainer::Mp4,
            )
            .unwrap_err(),
            FfmpegRemuxPlanError::OutputEqualsInput
        );

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_parent_directory_traversal_in_output() {
        let (root, completed) = completed_download("traversal");

        assert_eq!(
            FfmpegRemuxPlan::new(
                "runtime/ffmpeg",
                &completed,
                root.join("nested").join("..").join("out.mp4"),
                FfmpegRemuxContainer::Mp4,
            )
            .unwrap_err(),
            FfmpegRemuxPlanError::OutputPathTraversal
        );

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }
}
