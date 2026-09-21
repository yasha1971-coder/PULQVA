use std::path::{Path, PathBuf};

use crate::{CompletedMediaArtifactReceipt, YtDlpMediaSourceUrl};

/// Stable completed-download value for the future desktop/UI boundary.
///
/// Construction is private to the crate so callers cannot substitute an
/// arbitrary filesystem path for a validated artifact receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedDownloadResult {
    source: YtDlpMediaSourceUrl,
    artifact: CompletedMediaArtifactReceipt,
}

impl CompletedDownloadResult {
    pub fn source(&self) -> &YtDlpMediaSourceUrl {
        &self.source
    }

    pub fn source_url(&self) -> &str {
        self.source.as_str()
    }

    pub fn artifact_path(&self) -> &Path {
        self.artifact.path()
    }

    pub fn byte_size(&self) -> u64 {
        self.artifact.byte_size()
    }

    /// Produces deterministic, process-free fields for a future UI/serialization
    /// boundary. No process handle, proxy URL, or other transport internals are
    /// included.
    pub fn display_fields(&self) -> CompletedDownloadDisplayFields {
        CompletedDownloadDisplayFields {
            source_url: self.source.as_str().to_owned(),
            artifact_path: self.artifact.path().to_owned(),
            byte_size: self.artifact.byte_size(),
        }
    }

    pub(crate) fn from_validated_completion(
        source: YtDlpMediaSourceUrl,
        artifact: CompletedMediaArtifactReceipt,
    ) -> Self {
        Self { source, artifact }
    }
}

/// Deterministic display-facing fields derived only from validated completion
/// data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedDownloadDisplayFields {
    source_url: String,
    artifact_path: PathBuf,
    byte_size: u64,
}

impl CompletedDownloadDisplayFields {
    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    pub fn byte_size(&self) -> u64 {
        self.byte_size
    }
}

#[cfg(test)]
mod tests {
    use super::CompletedDownloadResult;
    use crate::{
        YtDlpMediaSourceUrl,
        ytdlp_artifact::validate_completed_media_artifact,
    };
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-download-result-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn result_fields_come_only_from_typed_source_and_validated_receipt() {
        let root = test_root("fields");
        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::write(root.join("media.bin"), b"PULQVA").expect("artifact write succeeds");

        let source =
            YtDlpMediaSourceUrl::parse("https://example.invalid/media").expect("valid source");
        let receipt =
            validate_completed_media_artifact(&root).expect("artifact receipt validates");

        let expected_path = receipt.path().to_owned();
        let expected_size = receipt.byte_size();
        let result = CompletedDownloadResult::from_validated_completion(source, receipt);

        assert_eq!(result.source_url(), "https://example.invalid/media");
        assert_eq!(result.artifact_path(), expected_path);
        assert_eq!(result.byte_size(), expected_size);

        let display = result.display_fields();
        assert_eq!(display.source_url(), result.source_url());
        assert_eq!(display.artifact_path(), result.artifact_path());
        assert_eq!(display.byte_size(), result.byte_size());

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }
}
