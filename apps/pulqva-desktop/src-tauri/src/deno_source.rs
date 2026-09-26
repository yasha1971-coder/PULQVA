//! Pinned local Deno archive -> authenticated snapshot -> owned verified stage.
//! No launch authority: path replacement after verification still requires prelaunch checks.
#![allow(dead_code)]
use super::deno_zip::{DenoTarget, archive_identity};
use super::deno_stage::{StagedDeno, extract_to_stage};
use same_file::Handle;
use sha2::{Digest, Sha256};
use std::{fs::{self, File}, io::Read, path::{Component, Path}};
const MAX_SOURCE: u64 = 64 * 1024 * 1024;

fn validate_path(root: &Path, source: &Path) -> Result<(), &'static str> {
    if !root.is_absolute() || fs::canonicalize(root).map_err(|_| "source-root-invalid")? != root {
        return Err("source-root-not-canonical");
    }
    let root_meta = fs::symlink_metadata(root).map_err(|_| "source-root-invalid")?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() { return Err("source-root-invalid"); }
    let relative = source.strip_prefix(root).map_err(|_| "source-outside-resources")?;
    if relative.as_os_str().is_empty() { return Err("source-path-empty"); }
    let mut cursor = root.to_owned();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else { return Err("source-path-non-normal"); };
        cursor.push(name);
        let m = fs::symlink_metadata(&cursor).map_err(|_| "source-path-invalid")?;
        if m.file_type().is_symlink() { return Err("source-symlink"); }
        if components.peek().is_some() && !m.is_dir() { return Err("source-parent-not-directory"); }
        if components.peek().is_none() && !m.is_file() { return Err("source-not-regular"); }
    }
    if fs::canonicalize(source).map_err(|_| "source-path-invalid")? != source {
        return Err("source-path-alias");
    }
    Ok(())
}

fn snapshot(root: &Path, source: &Path, expected_size: u64, expected: [u8; 32])
    -> Result<Vec<u8>, &'static str>
{
    if expected_size == 0 || expected_size > MAX_SOURCE { return Err("source-size-limit"); }
    validate_path(root, source)?;
    let root_id = Handle::from_path(root).map_err(|_| "source-root-identity")?;
    let before = Handle::from_path(source).map_err(|_| "source-identity")?;
    let file = File::open(source).map_err(|_| "source-open-failed")?;
    let opened = Handle::from_file(file.try_clone().map_err(|_| "source-handle-failed")?)
        .map_err(|_| "source-identity")?;
    if before != opened { return Err("source-replaced-before-open"); }
    let metadata = file.metadata().map_err(|_| "source-metadata-failed")?;
    if !metadata.is_file() || metadata.len() != expected_size { return Err("source-size-mismatch"); }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(expected_size as usize + 1).map_err(|_| "source-memory-limit")?;
    file.take(expected_size + 1).read_to_end(&mut bytes).map_err(|_| "source-read-failed")?;
    if bytes.len() as u64 != expected_size { return Err("source-size-mismatch"); }
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    if digest != expected { return Err("source-digest-mismatch"); }
    validate_path(root, source)?;
    if Handle::from_path(root).map_err(|_| "source-root-identity")? != root_id
        || Handle::from_path(source).map_err(|_| "source-identity")? != opened {
        return Err("source-replaced-during-read");
    }
    // Only these hashed bytes are handed to the parser: no subsequent source-path reopen.
    Ok(bytes)
}

/// Paths are backend-owned. Does not download, discover host runtimes or publish globally.
pub(super) fn materialize_deno(resource_root: &Path, source: &Path, stage_root: &Path,
    target: DenoTarget) -> Result<StagedDeno, &'static str>
{
    let (size, hash) = archive_identity(target);
    let bytes = snapshot(resource_root, source, size, hash)?;
    extract_to_stage(stage_root, &bytes, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn root() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("pulqva-deno-source-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&p).unwrap(); fs::canonicalize(p).unwrap()
    }

    #[test]
    fn snapshot_stays_immutable_after_source_changes() {
        let root = root(); let p = root.join("archive"); fs::write(&p, b"original").unwrap();
        let bytes = snapshot(&root, &p, 8, Sha256::digest(b"original").into()).unwrap();
        fs::write(&p, b"modified").unwrap(); assert_eq!(bytes, b"original");
        fs::remove_file(p).unwrap(); fs::remove_dir(root).unwrap();
    }

    #[test]
    fn rejects_size_digest_directory_and_escape_without_mutation() {
        let root = root(); let p = root.join("archive"); fs::write(&p, b"data").unwrap();
        let digest = Sha256::digest(b"data").into();
        for size in [0, 3, 5, MAX_SOURCE + 1] { assert!(snapshot(&root, &p, size, digest).is_err()); }
        assert!(snapshot(&root, &p, 4, [0; 32]).is_err());
        assert!(snapshot(&root, &root, 4, digest).is_err());
        assert!(snapshot(&root, &root.join("missing"), 4, digest).is_err());
        assert!(materialize_deno(&root, &p, &root, DenoTarget::LinuxX64).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert!(snapshot(&root, &root.join("../archive"), 4, digest).is_err());
        assert_eq!(fs::read(&p).unwrap(), b"data"); fs::remove_file(p).unwrap(); fs::remove_dir(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symbolic_link_even_when_bytes_match() {
        let root = root(); let p = root.join("archive"); let link = root.join("link");
        fs::write(&p, b"data").unwrap(); std::os::unix::fs::symlink(&p, &link).unwrap();
        assert!(snapshot(&root, &link, 4, Sha256::digest(b"data").into()).is_err());
        fs::remove_file(link).unwrap(); fs::remove_file(p).unwrap(); fs::remove_dir(root).unwrap();
    }
}
