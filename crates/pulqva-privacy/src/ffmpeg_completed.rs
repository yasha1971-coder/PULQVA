use std::{
    error::Error,
    fmt,
    fs,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedFfmpegRemuxResult {
    input: PathBuf,
    output: PathBuf,
    byte_size: u64,
}

impl CompletedFfmpegRemuxResult {
    pub fn input(&self) -> &Path {
        &self.input
    }

    pub fn output(&self) -> &Path {
        &self.output
    }

    pub fn byte_size(&self) -> u64 {
        self.byte_size
    }

    fn new(input: PathBuf, output: PathBuf, byte_size: u64) -> Self {
        Self {
            input,
            output,
            byte_size,
        }
    }
}

pub(crate) fn validate_completed_ffmpeg_remux(
    input: &Path,
    output: &Path,
) -> Result<CompletedFfmpegRemuxResult, FfmpegRemuxArtifactError> {
    if input == output {
        return Err(FfmpegRemuxArtifactError::OutputEqualsInput);
    }

    let output_meta = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(FfmpegRemuxArtifactError::MissingOutput);
        }
        Err(source) => {
            return Err(FfmpegRemuxArtifactError::Io {
                operation: "read FFmpeg remux output metadata",
                source,
            });
        }
    };

    if output_meta.file_type().is_symlink() {
        return Err(FfmpegRemuxArtifactError::SymlinkOutput);
    }

    if !output_meta.is_file() {
        return Err(FfmpegRemuxArtifactError::NotRegularFile);
    }

    if output_meta.len() == 0 {
        return Err(FfmpegRemuxArtifactError::EmptyOutput);
    }

    let canonical_input = fs::canonicalize(input).map_err(|source| FfmpegRemuxArtifactError::Io {
        operation: "canonicalize validated FFmpeg input",
        source,
    })?;
    let canonical_output =
        fs::canonicalize(output).map_err(|source| FfmpegRemuxArtifactError::Io {
            operation: "canonicalize completed FFmpeg output",
            source,
        })?;

    if canonical_input == canonical_output {
        return Err(FfmpegRemuxArtifactError::OutputEqualsInput);
    }

    Ok(CompletedFfmpegRemuxResult::new(
        input.to_owned(),
        output.to_owned(),
        output_meta.len(),
    ))
}

#[derive(Debug)]
pub enum FfmpegRemuxArtifactError {
    MissingOutput,
    SymlinkOutput,
    NotRegularFile,
    EmptyOutput,
    OutputEqualsInput,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for FfmpegRemuxArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOutput => f.write_str("FFmpeg remux output does not exist"),
            Self::SymlinkOutput => f.write_str("FFmpeg remux output must not be a symlink"),
            Self::NotRegularFile => f.write_str("FFmpeg remux output must be a regular file"),
            Self::EmptyOutput => f.write_str("FFmpeg remux output must not be empty"),
            Self::OutputEqualsInput => {
                f.write_str("FFmpeg remux output must remain distinct from the validated input")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for FfmpegRemuxArtifactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FfmpegRemuxArtifactError, validate_completed_ffmpeg_remux};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-ffmpeg-completed-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn validates_non_empty_regular_output_distinct_from_input() {
        let root = test_root("valid");
        fs::create_dir_all(&root).expect("root creation succeeds");
        let input = root.join("input.mp4");
        let output = root.join("output.mp4");
        fs::write(&input, b"input").expect("input write succeeds");
        fs::write(&output, b"output").expect("output write succeeds");

        let result =
            validate_completed_ffmpeg_remux(&input, &output).expect("output validates");

        assert_eq!(result.input(), input);
        assert_eq!(result.output(), output);
        assert_eq!(result.byte_size(), 6);

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_missing_empty_directory_and_equal_output() {
        let root = test_root("reject");
        fs::create_dir_all(&root).expect("root creation succeeds");
        let input = root.join("input.mp4");
        fs::write(&input, b"input").expect("input write succeeds");

        assert!(matches!(
            validate_completed_ffmpeg_remux(&input, &root.join("missing.mp4")),
            Err(FfmpegRemuxArtifactError::MissingOutput)
        ));

        let empty = root.join("empty.mp4");
        fs::write(&empty, b"").expect("empty output write succeeds");
        assert!(matches!(
            validate_completed_ffmpeg_remux(&input, &empty),
            Err(FfmpegRemuxArtifactError::EmptyOutput)
        ));

        let directory = root.join("directory-output");
        fs::create_dir_all(&directory).expect("directory output creation succeeds");
        assert!(matches!(
            validate_completed_ffmpeg_remux(&input, &directory),
            Err(FfmpegRemuxArtifactError::NotRegularFile)
        ));

        assert!(matches!(
            validate_completed_ffmpeg_remux(&input, &input),
            Err(FfmpegRemuxArtifactError::OutputEqualsInput)
        ));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }
}
