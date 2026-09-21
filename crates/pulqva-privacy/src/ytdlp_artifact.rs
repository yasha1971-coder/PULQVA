use std::{
    error::Error,
    fmt,
    fs,
    io,
    path::{Component, Path, PathBuf},
};

/// Typed receipt for exactly one completed regular media artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedMediaArtifactReceipt {
    path: PathBuf,
    byte_size: u64,
}

impl CompletedMediaArtifactReceipt {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn byte_size(&self) -> u64 {
        self.byte_size
    }
}

pub(crate) fn validate_completed_media_artifact(
    output_root: &Path,
) -> Result<CompletedMediaArtifactReceipt, MediaArtifactReceiptError> {
    if output_root
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(MediaArtifactReceiptError::PathTraversal);
    }

    let root_metadata = match fs::symlink_metadata(output_root) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(MediaArtifactReceiptError::NoArtifact);
        }
        Err(source) => {
            return Err(MediaArtifactReceiptError::Io {
                operation: "inspect media output root",
                source,
            });
        }
    };

    if root_metadata.file_type().is_symlink() {
        return Err(MediaArtifactReceiptError::SymlinkFound(
            output_root.to_owned(),
        ));
    }

    if !root_metadata.is_dir() {
        return Err(MediaArtifactReceiptError::OutputRootNotDirectory);
    }

    let canonical_root =
        fs::canonicalize(output_root).map_err(|source| MediaArtifactReceiptError::Io {
            operation: "canonicalize media output root",
            source,
        })?;

    let mut files = Vec::new();
    collect_regular_files(output_root, &mut files)?;

    let artifact = match files.len() {
        0 => return Err(MediaArtifactReceiptError::NoArtifact),
        1 => files.pop().expect("length checked"),
        count => return Err(MediaArtifactReceiptError::MultipleArtifacts { count }),
    };

    let canonical_artifact =
        fs::canonicalize(&artifact).map_err(|source| MediaArtifactReceiptError::Io {
            operation: "canonicalize completed media artifact",
            source,
        })?;

    if !canonical_artifact.starts_with(&canonical_root) {
        return Err(MediaArtifactReceiptError::PathEscape);
    }

    let metadata =
        fs::metadata(&canonical_artifact).map_err(|source| MediaArtifactReceiptError::Io {
            operation: "read completed media artifact metadata",
            source,
        })?;

    let byte_size = metadata.len();
    if byte_size == 0 {
        return Err(MediaArtifactReceiptError::EmptyArtifact);
    }

    Ok(CompletedMediaArtifactReceipt {
        path: canonical_artifact,
        byte_size,
    })
}

fn collect_regular_files(
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), MediaArtifactReceiptError> {
    let entries = fs::read_dir(directory).map_err(|source| MediaArtifactReceiptError::Io {
        operation: "read media output directory",
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| MediaArtifactReceiptError::Io {
            operation: "read media output directory entry",
            source,
        })?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| MediaArtifactReceiptError::Io {
                operation: "inspect media output entry",
                source,
            })?;

        if metadata.file_type().is_symlink() {
            return Err(MediaArtifactReceiptError::SymlinkFound(path));
        }

        if metadata.is_dir() {
            collect_regular_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        } else {
            return Err(MediaArtifactReceiptError::UnsupportedFileType(path));
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum MediaArtifactReceiptError {
    PathTraversal,
    OutputRootNotDirectory,
    NoArtifact,
    MultipleArtifacts { count: usize },
    EmptyArtifact,
    SymlinkFound(PathBuf),
    UnsupportedFileType(PathBuf),
    PathEscape,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for MediaArtifactReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathTraversal => {
                f.write_str("media output root must not contain parent-directory traversal")
            }
            Self::OutputRootNotDirectory => {
                f.write_str("media output root must be a directory")
            }
            Self::NoArtifact => {
                f.write_str("media output root contains no completed artifact")
            }
            Self::MultipleArtifacts { count } => {
                write!(f, "media output root contains {count} regular files; exactly one is required")
            }
            Self::EmptyArtifact => {
                f.write_str("completed media artifact must not be empty")
            }
            Self::SymlinkFound(path) => {
                write!(f, "symlink is not allowed in media output: {}", path.display())
            }
            Self::UnsupportedFileType(path) => {
                write!(f, "unsupported file type in media output: {}", path.display())
            }
            Self::PathEscape => {
                f.write_str("canonical media artifact path escaped the canonical output root")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for MediaArtifactReceiptError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MediaArtifactReceiptError, validate_completed_media_artifact};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-artifact-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn accepts_exactly_one_non_empty_regular_file() {
        let root = test_root("one");
        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::write(root.join("media.bin"), b"pulqva").expect("artifact write succeeds");

        let receipt =
            validate_completed_media_artifact(&root).expect("single artifact is accepted");

        assert_eq!(receipt.byte_size(), 6);
        assert_eq!(
            receipt.path(),
            fs::canonicalize(root.join("media.bin"))
                .expect("artifact canonicalization succeeds")
        );

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_empty_output_root() {
        let root = test_root("empty");
        fs::create_dir_all(&root).expect("root creation succeeds");

        assert!(matches!(
            validate_completed_media_artifact(&root),
            Err(MediaArtifactReceiptError::NoArtifact)
        ));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_multiple_regular_files() {
        let root = test_root("multiple");
        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::write(root.join("one.bin"), b"1").expect("first artifact write succeeds");
        fs::write(root.join("two.bin"), b"2").expect("second artifact write succeeds");

        assert!(matches!(
            validate_completed_media_artifact(&root),
            Err(MediaArtifactReceiptError::MultipleArtifacts { count: 2 })
        ));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_zero_byte_artifact() {
        let root = test_root("zero");
        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::write(root.join("zero.bin"), b"").expect("empty artifact write succeeds");

        assert!(matches!(
            validate_completed_media_artifact(&root),
            Err(MediaArtifactReceiptError::EmptyArtifact)
        ));

        fs::remove_dir_all(root).expect("cleanup succeeds");
    }

    #[test]
    fn rejects_parent_directory_traversal_in_output_root() {
        let base = test_root("traversal");
        let root = base.join("inside");
        let traversal = root.join("..").join("outside");

        assert!(matches!(
            validate_completed_media_artifact(&traversal),
            Err(MediaArtifactReceiptError::PathTraversal)
        ));
    }

    #[test]
    fn rejects_symlink_inside_output_root() {
        let root = test_root("symlink");
        let outside = test_root("outside");
        fs::create_dir_all(&root).expect("root creation succeeds");
        fs::create_dir_all(&outside).expect("outside creation succeeds");
        let target = outside.join("target.bin");
        fs::write(&target, b"outside").expect("outside file write succeeds");
        let link = root.join("link.bin");

        create_file_symlink(&target, &link);

        assert!(matches!(
            validate_completed_media_artifact(&root),
            Err(MediaArtifactReceiptError::SymlinkFound(path)) if path == link
        ));

        fs::remove_dir_all(root).expect("root cleanup succeeds");
        fs::remove_dir_all(outside).expect("outside cleanup succeeds");
    }

    #[cfg(unix)]
    fn create_file_symlink(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("test symlink creation succeeds");
    }

    #[cfg(windows)]
    fn create_file_symlink(target: &Path, link: &Path) {
        std::os::windows::fs::symlink_file(target, link)
            .expect("test symlink creation succeeds");
    }
}
