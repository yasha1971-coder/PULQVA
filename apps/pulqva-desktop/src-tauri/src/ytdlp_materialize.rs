use super::{
    AppRuntimeLayout, BundledSidecarKind, DownloadActionError, PreparedAppRuntimeDirectories,
    VerifiedBundledSidecarArtifact, YTDLP_SIDECAR_SHA256SUMS, YTDLP_SIDECAR_VERSION,
    current_sidecar_platform_spec, pinned_sha256_for_asset,
};
use same_file::{Handle, is_same_file};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MaterializedYtDlp {
    path: PathBuf,
    byte_size: u64,
    reused_existing: bool,
}

impl MaterializedYtDlp {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    fn byte_size(&self) -> u64 {
        self.byte_size
    }

    #[cfg(test)]
    fn reused_existing(&self) -> bool {
        self.reused_existing
    }
}

#[derive(Debug, Clone)]
struct ExpectedYtDlpIdentity {
    version: String,
    sha256: String,
}

struct OwnedStage {
    path: PathBuf,
    identity: Handle,
    active: bool,
}

impl OwnedStage {
    fn cleanup_checked(&mut self) -> Result<(), DownloadActionError> {
        if !self.active {
            return Ok(());
        }

        match Handle::from_path(&self.path) {
            Ok(current) if current == self.identity => {
                fs::remove_file(&self.path).map_err(|source| DownloadActionError {
                    code: "ytdlp-stage-cleanup-failed",
                    message: format!(
                        "failed to remove owned yt-dlp staging file {}: {source}",
                        self.path.display()
                    ),
                })?;
                self.active = false;
                Ok(())
            }
            Ok(_) => Err(DownloadActionError {
                code: "ytdlp-stage-ownership-lost",
                message: format!(
                    "yt-dlp staging path no longer names the file created by this operation: {}",
                    self.path.display()
                ),
            }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                self.active = false;
                Ok(())
            }
            Err(source) => Err(DownloadActionError {
                code: "ytdlp-stage-inspection-failed",
                message: format!(
                    "failed to inspect yt-dlp staging path {}: {source}",
                    self.path.display()
                ),
            }),
        }
    }
}

impl Drop for OwnedStage {
    fn drop(&mut self) {
        let _ = self.cleanup_checked();
    }
}

pub(super) fn materialize_verified_ytdlp(
    prepared: &PreparedAppRuntimeDirectories,
    resource_root: &Path,
    artifact: &VerifiedBundledSidecarArtifact,
) -> Result<MaterializedYtDlp, DownloadActionError> {
    let platform = current_sidecar_platform_spec()?;
    let expected = ExpectedYtDlpIdentity {
        version: YTDLP_SIDECAR_VERSION.trim().to_owned(),
        sha256: pinned_sha256_for_asset(YTDLP_SIDECAR_SHA256SUMS, platform.ytdlp_digest_asset)?,
    };

    materialize_verified_ytdlp_with_identity(prepared, resource_root, artifact, &expected)
}

fn materialize_verified_ytdlp_with_identity(
    prepared: &PreparedAppRuntimeDirectories,
    resource_root: &Path,
    artifact: &VerifiedBundledSidecarArtifact,
    expected: &ExpectedYtDlpIdentity,
) -> Result<MaterializedYtDlp, DownloadActionError> {
    validate_digest(&expected.sha256)?;

    if artifact.kind != BundledSidecarKind::YtDlp {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-kind-unsupported",
            message: "direct yt-dlp materialization accepts only the verified yt-dlp artifact"
                .to_owned(),
        });
    }

    if artifact.identity.version != expected.version
        || artifact.identity.pinned_source_sha256.as_deref() != Some(expected.sha256.as_str())
    {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-identity-mismatch",
            message: "verified yt-dlp artifact identity does not match pinned backend metadata"
                .to_owned(),
        });
    }

    let expected_destination = prepared.layout().ytdlp_executable()?;
    if artifact.destination != expected_destination {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-destination-mismatch",
            message: "verified yt-dlp destination does not match the prepared runtime layout"
                .to_owned(),
        });
    }

    if artifact.source == artifact.destination {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-destination-alias",
            message: "yt-dlp source and destination paths must differ".to_owned(),
        });
    }

    let canonical_source =
        validate_source_under_resource_root(resource_root, &artifact.source, artifact.byte_size)?;

    let (bin_root, canonical_bin_root) = ensure_bin_root(prepared.layout())?;
    if artifact.destination.parent() != Some(bin_root.as_path()) {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-destination-escaped",
            message: "yt-dlp destination must be a direct child of runtime/bin".to_owned(),
        });
    }

    match validate_existing_destination(
        &artifact.destination,
        &canonical_source,
        &canonical_bin_root,
        &expected.sha256,
    )? {
        Some(result) => return Ok(result),
        None => {}
    }

    let mut source = File::open(&canonical_source).map_err(|source| DownloadActionError {
        code: "ytdlp-materialization-source-open-failed",
        message: format!(
            "failed to open verified yt-dlp source {}: {source}",
            canonical_source.display()
        ),
    })?;

    let source_metadata = source.metadata().map_err(|source| DownloadActionError {
        code: "ytdlp-materialization-source-inspection-failed",
        message: format!(
            "failed to inspect opened yt-dlp source {}: {source}",
            canonical_source.display()
        ),
    })?;
    if !source_metadata.is_file() {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-not-file",
            message: "opened yt-dlp source must remain a regular file".to_owned(),
        });
    }

    let mut stage = create_owned_stage(&bin_root)?;
    let stage_path = stage.path.clone();
    let mut stage_file = OpenOptions::new()
        .write(true)
        .open(&stage_path)
        .map_err(|source| DownloadActionError {
            code: "ytdlp-stage-open-failed",
            message: format!(
                "failed to reopen owned yt-dlp staging file {}: {source}",
                stage_path.display()
            ),
        })?;

    let mut hasher = Sha256::new();
    let mut byte_size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let count = match source.read(&mut buffer) {
            Err(source) if source.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => {
                return Err(DownloadActionError {
                    code: "ytdlp-materialization-source-read-failed",
                    message: format!(
                        "failed while reading verified yt-dlp source {}: {source}",
                        canonical_source.display()
                    ),
                });
            }
            Ok(count) => count,
        };

        if count == 0 {
            break;
        }

        stage_file
            .write_all(&buffer[..count])
            .map_err(|source| DownloadActionError {
                code: "ytdlp-stage-write-failed",
                message: format!(
                    "failed while writing yt-dlp staging file {}: {source}",
                    stage_path.display()
                ),
            })?;
        hasher.update(&buffer[..count]);
        byte_size += count as u64;
    }

    if byte_size == 0 {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-empty",
            message: "verified yt-dlp source must not be empty".to_owned(),
        });
    }

    let actual = format!("{:x}", hasher.finalize());
    if actual != expected.sha256 {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-hash-mismatch",
            message: "yt-dlp bytes changed or do not match the pinned SHA-256".to_owned(),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        stage_file
            .set_permissions(fs::Permissions::from_mode(0o700))
            .map_err(|source| DownloadActionError {
                code: "ytdlp-stage-permission-failed",
                message: format!(
                    "failed to set executable permissions on owned yt-dlp staging file {}: {source}",
                    stage_path.display()
                ),
            })?;
    }

    stage_file.sync_all().map_err(|source| DownloadActionError {
        code: "ytdlp-stage-sync-failed",
        message: format!(
            "failed to synchronize complete yt-dlp staging file {}: {source}",
            stage_path.display()
        ),
    })?;
    drop(stage_file);

    match fs::hard_link(&stage_path, &artifact.destination) {
        Ok(()) => {
            if !is_same_file(&stage_path, &artifact.destination).map_err(|source| {
                DownloadActionError {
                    code: "ytdlp-publication-identity-check-failed",
                    message: format!(
                        "failed to verify yt-dlp publication identity {}: {source}",
                        artifact.destination.display()
                    ),
                }
            })? {
                return Err(DownloadActionError {
                    code: "ytdlp-publication-identity-mismatch",
                    message: "published yt-dlp destination does not identify the verified staged file"
                        .to_owned(),
                });
            }

            stage.cleanup_checked()?;

            Ok(MaterializedYtDlp {
                path: artifact.destination.clone(),
                byte_size,
                reused_existing: false,
            })
        }
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            let existing = validate_existing_destination(
                &artifact.destination,
                &canonical_source,
                &canonical_bin_root,
                &expected.sha256,
            )?
            .ok_or_else(|| DownloadActionError {
                code: "ytdlp-publication-race-invalid",
                message:
                    "yt-dlp destination appeared during publication but was not a reusable verified file"
                        .to_owned(),
            })?;

            stage.cleanup_checked()?;
            Ok(existing)
        }
        Err(source) => Err(DownloadActionError {
            code: "ytdlp-publication-failed",
            message: format!(
                "failed to atomically publish verified yt-dlp staging file to {}: {source}",
                artifact.destination.display()
            ),
        }),
    }
}

fn validate_digest(digest: &str) -> Result<(), DownloadActionError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-digest-invalid",
            message: "pinned yt-dlp SHA-256 must contain exactly 64 hexadecimal characters"
                .to_owned(),
        });
    }
    Ok(())
}

fn validate_source_under_resource_root(
    resource_root: &Path,
    source: &Path,
    receipt_size: u64,
) -> Result<PathBuf, DownloadActionError> {
    let root_metadata = fs::symlink_metadata(resource_root).map_err(|source| DownloadActionError {
        code: "ytdlp-resource-root-inspection-failed",
        message: format!(
            "failed to inspect yt-dlp package resource root {}: {source}",
            resource_root.display()
        ),
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(DownloadActionError {
            code: "ytdlp-resource-root-unsafe",
            message: "yt-dlp package resource root must be a real directory".to_owned(),
        });
    }

    let canonical_root = fs::canonicalize(resource_root).map_err(|source| DownloadActionError {
        code: "ytdlp-resource-root-canonicalization-failed",
        message: format!(
            "failed to canonicalize yt-dlp package resource root {}: {source}",
            resource_root.display()
        ),
    })?;

    let containment_root = if source.starts_with(resource_root) {
        resource_root
    } else if source.starts_with(&canonical_root) {
        canonical_root.as_path()
    } else {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-escaped",
            message: "verified yt-dlp source must remain beneath the package resource root"
                .to_owned(),
        });
    };

    let relative = source
        .strip_prefix(containment_root)
        .map_err(|source| DownloadActionError {
            code: "ytdlp-materialization-source-escaped",
            message: format!("failed to verify yt-dlp source containment: {source}"),
        })?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-path-invalid",
            message: "verified yt-dlp source must be a non-empty normal child path".to_owned(),
        });
    }

    let mut cursor = canonical_root.clone();
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(part) = component else {
            return Err(DownloadActionError {
                code: "ytdlp-materialization-source-path-invalid",
                message: "verified yt-dlp source contains a non-normal path component".to_owned(),
            });
        };
        cursor.push(part);
        let metadata = fs::symlink_metadata(&cursor).map_err(|source| DownloadActionError {
            code: "ytdlp-materialization-source-inspection-failed",
            message: format!(
                "failed to inspect yt-dlp source component {}: {source}",
                cursor.display()
            ),
        })?;
        if metadata.file_type().is_symlink() {
            return Err(DownloadActionError {
                code: "ytdlp-materialization-source-symlink",
                message: format!(
                    "yt-dlp source path must not traverse a symlink: {}",
                    cursor.display()
                ),
            });
        }
        let final_component = index + 1 == components.len();
        if final_component && !metadata.is_file() {
            return Err(DownloadActionError {
                code: "ytdlp-materialization-source-not-file",
                message: "verified yt-dlp source must be a regular file".to_owned(),
            });
        }
        if !final_component && !metadata.is_dir() {
            return Err(DownloadActionError {
                code: "ytdlp-materialization-source-parent-not-directory",
                message: "yt-dlp source parent must be a directory".to_owned(),
            });
        }
    }

    let canonical_source = fs::canonicalize(source).map_err(|source| DownloadActionError {
        code: "ytdlp-materialization-source-canonicalization-failed",
        message: format!("failed to canonicalize verified yt-dlp source: {source}"),
    })?;

    if !canonical_source.starts_with(&canonical_root) {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-escaped",
            message: "canonical yt-dlp source escaped the package resource root".to_owned(),
        });
    }

    let metadata = fs::metadata(&canonical_source).map_err(|source| DownloadActionError {
        code: "ytdlp-materialization-source-inspection-failed",
        message: format!("failed to inspect canonical yt-dlp source: {source}"),
    })?;
    if !metadata.is_file() {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-not-file",
            message: "canonical yt-dlp source must be a regular file".to_owned(),
        });
    }
    if metadata.len() != receipt_size {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-changed",
            message: "yt-dlp source size changed after T054 validation".to_owned(),
        });
    }

    Ok(canonical_source)
}

fn ensure_bin_root(
    layout: &AppRuntimeLayout,
) -> Result<(PathBuf, PathBuf), DownloadActionError> {
    let root_metadata = fs::symlink_metadata(layout.root()).map_err(|source| DownloadActionError {
        code: "ytdlp-runtime-root-inspection-failed",
        message: format!(
            "failed to inspect prepared runtime root {}: {source}",
            layout.root().display()
        ),
    })?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(DownloadActionError {
            code: "ytdlp-runtime-root-unsafe",
            message: "prepared runtime root must remain a real directory".to_owned(),
        });
    }
    let canonical_root = fs::canonicalize(layout.root()).map_err(|source| DownloadActionError {
        code: "ytdlp-runtime-root-canonicalization-failed",
        message: format!("failed to canonicalize prepared runtime root: {source}"),
    })?;

    let bin_root = layout.bin_root()?;
    match fs::symlink_metadata(&bin_root) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(DownloadActionError {
                    code: "ytdlp-bin-root-unsafe",
                    message: "runtime/bin must be a real directory".to_owned(),
                });
            }
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            match fs::create_dir(&bin_root) {
                Ok(()) => {}
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => {
                    return Err(DownloadActionError {
                        code: "ytdlp-bin-root-creation-failed",
                        message: format!(
                            "failed to create app-owned runtime/bin {}: {source}",
                            bin_root.display()
                        ),
                    });
                }
            }

            let metadata =
                fs::symlink_metadata(&bin_root).map_err(|source| DownloadActionError {
                    code: "ytdlp-bin-root-inspection-failed",
                    message: format!(
                        "failed to inspect created runtime/bin {}: {source}",
                        bin_root.display()
                    ),
                })?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(DownloadActionError {
                    code: "ytdlp-bin-root-unsafe",
                    message: "created runtime/bin must be a real directory".to_owned(),
                });
            }
        }
        Err(source) => {
            return Err(DownloadActionError {
                code: "ytdlp-bin-root-inspection-failed",
                message: format!("failed to inspect runtime/bin {}: {source}", bin_root.display()),
            });
        }
    }

    let canonical_bin = fs::canonicalize(&bin_root).map_err(|source| DownloadActionError {
        code: "ytdlp-bin-root-canonicalization-failed",
        message: format!("failed to canonicalize runtime/bin {}: {source}", bin_root.display()),
    })?;
    if !canonical_bin.starts_with(&canonical_root) {
        return Err(DownloadActionError {
            code: "ytdlp-bin-root-escaped",
            message: "runtime/bin escaped the prepared app-owned runtime root".to_owned(),
        });
    }

    Ok((bin_root, canonical_bin))
}

fn validate_existing_destination(
    destination: &Path,
    canonical_source: &Path,
    canonical_bin_root: &Path,
    expected_sha256: &str,
) -> Result<Option<MaterializedYtDlp>, DownloadActionError> {
    let metadata = match fs::symlink_metadata(destination) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(DownloadActionError {
                code: "ytdlp-destination-inspection-failed",
                message: format!(
                    "failed to inspect existing yt-dlp destination {}: {source}",
                    destination.display()
                ),
            });
        }
    };

    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DownloadActionError {
            code: "ytdlp-destination-unsafe",
            message: "existing yt-dlp destination must be a real regular file".to_owned(),
        });
    }

    let canonical_destination =
        fs::canonicalize(destination).map_err(|source| DownloadActionError {
            code: "ytdlp-destination-canonicalization-failed",
            message: format!(
                "failed to canonicalize existing yt-dlp destination {}: {source}",
                destination.display()
            ),
        })?;
    if !canonical_destination.starts_with(canonical_bin_root)
        || canonical_destination.parent() != Some(canonical_bin_root)
    {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-destination-escaped",
            message: "existing yt-dlp destination escaped runtime/bin".to_owned(),
        });
    }

    if is_same_file(canonical_source, &canonical_destination).map_err(|source| {
        DownloadActionError {
            code: "ytdlp-file-identity-check-failed",
            message: format!("failed to compare yt-dlp source/destination identity: {source}"),
        }
    })? {
        return Err(DownloadActionError {
            code: "ytdlp-materialization-source-destination-alias",
            message: "yt-dlp source and destination must not be hard-link aliases".to_owned(),
        });
    }

    let actual = hash_file(&canonical_destination)?;
    if actual != expected_sha256 {
        return Err(DownloadActionError {
            code: "ytdlp-existing-destination-mismatch",
            message: "existing yt-dlp destination differs from the pinned verified binary"
                .to_owned(),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&canonical_destination)
            .map_err(|source| DownloadActionError {
                code: "ytdlp-destination-inspection-failed",
                message: format!("failed to inspect yt-dlp destination permissions: {source}"),
            })?
            .permissions()
            .mode()
            & 0o111
            == 0
        {
            return Err(DownloadActionError {
                code: "ytdlp-existing-destination-not-executable",
                message: "existing verified yt-dlp destination is not executable".to_owned(),
            });
        }
    }

    Ok(Some(MaterializedYtDlp {
        path: destination.to_path_buf(),
        byte_size: metadata.len(),
        reused_existing: true,
    }))
}

fn hash_file(path: &Path) -> Result<String, DownloadActionError> {
    let mut file = File::open(path).map_err(|source| DownloadActionError {
        code: "ytdlp-file-read-failed",
        message: format!("failed to open yt-dlp file {}: {source}", path.display()),
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = match file.read(&mut buffer) {
            Err(source) if source.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => {
                return Err(DownloadActionError {
                    code: "ytdlp-file-read-failed",
                    message: format!("failed to read yt-dlp file {}: {source}", path.display()),
                });
            }
            Ok(count) => count,
        };
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn create_owned_stage(bin_root: &Path) -> Result<OwnedStage, DownloadActionError> {
    for _ in 0..64 {
        let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = bin_root.join(format!(
            ".pulqva-ytdlp-stage-{}-{sequence}",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        match options.open(&path) {
            Ok(file) => {
                let identity = Handle::from_path(&path).map_err(|source| DownloadActionError {
                    code: "ytdlp-stage-identity-failed",
                    message: format!(
                        "failed to capture yt-dlp staging identity {}: {source}",
                        path.display()
                    ),
                })?;
                drop(file);
                return Ok(OwnedStage {
                    path,
                    identity,
                    active: true,
                });
            }
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(DownloadActionError {
                    code: "ytdlp-stage-creation-failed",
                    message: format!(
                        "failed to create exclusive yt-dlp staging file in {}: {source}",
                        bin_root.display()
                    ),
                });
            }
        }
    }

    Err(DownloadActionError {
        code: "ytdlp-stage-name-exhausted",
        message: "could not allocate a unique yt-dlp staging file name".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppRuntimeLayout, BundledSidecarIdentity, VerifiedBundledSidecarArtifact,
        prepare_app_runtime_directories,
    };
    use std::{
        sync::{Arc, Barrier},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn test_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "pulqva-t055-production-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn fixture(
        name: &str,
        bytes: &[u8],
    ) -> (
        PathBuf,
        PreparedAppRuntimeDirectories,
        VerifiedBundledSidecarArtifact,
        ExpectedYtDlpIdentity,
    ) {
        let root = test_root(name);
        let resource_root = root.join("resources");
        fs::create_dir_all(&resource_root).expect("resource root creation");
        let source = resource_root.join("yt-dlp");
        fs::write(&source, bytes).expect("source fixture write");
        let source = fs::canonicalize(&source).expect("source canonicalization");

        let layout = AppRuntimeLayout::new(root.join("runtime")).expect("layout");
        let prepared =
            prepare_app_runtime_directories(layout.clone()).expect("runtime directory preparation");
        let expected = ExpectedYtDlpIdentity {
            version: "fixture-version".to_owned(),
            sha256: digest(bytes),
        };
        let artifact = VerifiedBundledSidecarArtifact {
            kind: BundledSidecarKind::YtDlp,
            source,
            destination: layout.ytdlp_executable().expect("yt-dlp destination"),
            identity: BundledSidecarIdentity {
                version: expected.version.clone(),
                pinned_source_sha256: Some(expected.sha256.clone()),
            },
            byte_size: bytes.len() as u64,
        };
        (root, prepared, artifact, expected)
    }

    #[test]
    fn production_boundary_publishes_verified_bytes_and_reuses_matching_destination() {
        let bytes = vec![0x42_u8; 180_003];
        let (root, prepared, artifact, expected) = fixture("success-reuse", &bytes);
        let resource_root = root.join("resources");

        let first = materialize_verified_ytdlp_with_identity(
            &prepared,
            &resource_root,
            &artifact,
            &expected,
        )
        .expect("first publication");
        assert_eq!(first.path(), artifact.destination.as_path());
        assert_eq!(first.byte_size(), bytes.len() as u64);
        assert!(!first.reused_existing());
        assert_eq!(fs::read(first.path()).expect("published bytes"), bytes);

        let second = materialize_verified_ytdlp_with_identity(
            &prepared,
            &resource_root,
            &artifact,
            &expected,
        )
        .expect("matching destination reuse");
        assert!(second.reused_existing());
        assert_eq!(second.byte_size(), bytes.len() as u64);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(
                fs::metadata(second.path()).unwrap().permissions().mode() & 0o111,
                0
            );
        }

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_rejects_source_changed_after_t054_receipt() {
        let (root, prepared, artifact, expected) = fixture("source-change", b"original");
        fs::write(&artifact.source, b"changed!").expect("source mutation");

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("changed source must fail");
        assert!(
            matches!(
                error.code,
                "ytdlp-materialization-source-changed" | "ytdlp-materialization-hash-mismatch"
            ),
            "unexpected code: {}",
            error.code
        );
        assert!(!artifact.destination.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_rejects_unsupported_kind_before_copy() {
        let (root, prepared, mut artifact, expected) = fixture("unsupported-kind", b"abc");
        artifact.kind = BundledSidecarKind::Ffmpeg;

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("FFmpeg archive must not enter direct yt-dlp materializer");
        assert_eq!(error.code, "ytdlp-materialization-kind-unsupported");
        assert!(!artifact.destination.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_rejects_identity_mismatch_before_copy() {
        let (root, prepared, mut artifact, expected) = fixture("identity-mismatch", b"abc");
        artifact.identity.version = "untrusted-version".to_owned();

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("identity mismatch must fail");
        assert_eq!(error.code, "ytdlp-materialization-identity-mismatch");
        assert!(!artifact.destination.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_preserves_different_existing_destination() {
        let (root, prepared, artifact, expected) = fixture("existing-different", b"verified");
        fs::create_dir_all(artifact.destination.parent().unwrap()).expect("bin");
        fs::write(&artifact.destination, b"do not replace").expect("existing destination");

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("different destination must fail");
        assert_eq!(error.code, "ytdlp-existing-destination-mismatch");
        assert_eq!(
            fs::read(&artifact.destination).expect("preserved destination"),
            b"do not replace"
        );

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_rejects_source_outside_verified_resource_root() {
        let (root, prepared, mut artifact, expected) = fixture("escaped-source", b"abc");
        let outside = root.join("outside");
        fs::write(&outside, b"abc").expect("outside source");
        artifact.source = fs::canonicalize(&outside).expect("outside canonicalization");

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("escaped source must fail");
        assert_eq!(error.code, "ytdlp-materialization-source-escaped");
        assert!(!artifact.destination.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn production_boundary_rejects_hard_link_source_destination_alias() {
        let (root, prepared, artifact, expected) = fixture("hardlink-alias", b"abc");
        fs::create_dir_all(artifact.destination.parent().unwrap()).expect("bin");
        fs::hard_link(&artifact.source, &artifact.destination).expect("fixture hard link");

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("source/destination hard-link alias must fail");
        assert_eq!(
            error.code,
            "ytdlp-materialization-source-destination-alias"
        );
        assert_eq!(fs::read(&artifact.source).unwrap(), b"abc");
        assert_eq!(fs::read(&artifact.destination).unwrap(), b"abc");

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn competing_production_publishers_converge_on_one_verified_destination() {
        let bytes = vec![0x37_u8; 170_011];
        let (root, prepared, artifact, expected) = fixture("competing", &bytes);
        let gate = Arc::new(Barrier::new(2));

        let mut workers = Vec::new();
        for _ in 0..2 {
            let prepared = prepared.clone();
            let artifact = artifact.clone();
            let expected = expected.clone();
            let resource_root = root.join("resources");
            let gate = gate.clone();
            workers.push(thread::spawn(move || {
                gate.wait();
                materialize_verified_ytdlp_with_identity(
                    &prepared,
                    &resource_root,
                    &artifact,
                    &expected,
                )
            }));
        }

        let results = workers
            .into_iter()
            .map(|worker| worker.join().expect("worker join").expect("publisher success"))
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|result| !result.reused_existing()));
        assert_eq!(fs::read(&artifact.destination).unwrap(), bytes);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn production_boundary_rejects_symlink_destination_without_touching_target() {
        use std::os::unix::fs::symlink;

        let (root, prepared, artifact, expected) = fixture("symlink-destination", b"abc");
        fs::create_dir_all(artifact.destination.parent().unwrap()).expect("bin");
        let target = root.join("unrelated-target");
        fs::write(&target, b"preserve").expect("target");
        symlink(&target, &artifact.destination).expect("destination symlink");

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("symlink destination must fail");
        assert_eq!(error.code, "ytdlp-destination-unsafe");
        assert_eq!(fs::read(&target).unwrap(), b"preserve");

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn production_boundary_rejects_source_replaced_by_symlink_after_t054() {
        use std::os::unix::fs::symlink;

        let (root, prepared, mut artifact, expected) = fixture("symlink-source", b"abc");
        let outside = root.join("outside-source");
        fs::write(&outside, b"abc").expect("outside");
        fs::remove_file(&artifact.source).expect("remove original source");
        symlink(&outside, &artifact.source).expect("replace with symlink");
        artifact.byte_size = 3;

        let error = materialize_verified_ytdlp_with_identity(
            &prepared,
            &root.join("resources"),
            &artifact,
            &expected,
        )
        .expect_err("source symlink replacement must fail");
        assert_eq!(error.code, "ytdlp-materialization-source-symlink");
        assert!(!artifact.destination.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }
}
