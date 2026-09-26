//! Owned temporary output for extraction. No runtime publication or source-file opening.
#![allow(dead_code)]
use super::deno_zip::{Receipt, DenoTarget};
use same_file::Handle;
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Seek, SeekFrom}, path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering}};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_OUTPUT: u64 = 256 * 1024 * 1024;

struct OwnedStage {
    root: PathBuf,
    root_id: Handle,
    directory: PathBuf,
    directory_id: Handle,
    path: PathBuf,
    file_id: Option<Handle>,
    file: Option<File>,
}

fn same_regular(path: &Path, id: &Handle, directory: bool) -> bool {
    let Ok(m) = fs::symlink_metadata(path) else { return false; };
    !m.file_type().is_symlink() && (if directory { m.is_dir() } else { m.is_file() })
        && Handle::from_path(path).is_ok_and(|current| &current == id)
}

impl OwnedStage {
    fn create(root: &Path, name: &str) -> Result<Self, &'static str> {
        if !root.is_absolute() || fs::canonicalize(root).map_err(|_| "stage-root-invalid")? != root {
            return Err("stage-root-not-canonical");
        }
        let root_id = Handle::from_path(root).map_err(|_| "stage-root-invalid")?;
        if !same_regular(root, &root_id, true) { return Err("stage-root-invalid"); }
        for _ in 0..64 {
            let directory = root.join(format!(".deno-stage-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&directory) {
                Ok(()) => {},
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("stage-directory-create-failed"),
            }
            let directory_id = Handle::from_path(&directory).map_err(|_| "stage-directory-identity-failed")?;
            let path = directory.join(name);
            let mut stage = Self { root: root.to_owned(), root_id, directory, directory_id, path,
                file_id: None, file: None };
            if !stage.parents_owned() { return Err("stage-parent-ownership-lost"); }
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)] {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let file = options.open(&stage.path).map_err(|_| "stage-file-create-failed")?;
            stage.file_id = Some(Handle::from_file(file.try_clone().map_err(|_| "stage-handle-failed")?)
                .map_err(|_| "stage-file-identity-failed")?);
            stage.file = Some(file);
            return Ok(stage);
        }
        Err("stage-name-exhausted")
    }

    fn parents_owned(&self) -> bool {
        same_regular(&self.root, &self.root_id, true)
            && same_regular(&self.directory, &self.directory_id, true)
    }

    fn verify(&mut self, receipt: &Receipt) -> Result<(), &'static str> {
        if !self.parents_owned() || !self.file_id.as_ref().is_some_and(|id| same_regular(&self.path, id, false)) {
            return Err("stage-ownership-lost");
        }
        let file = self.file.as_mut().ok_or("stage-file-missing")?;
        file.sync_all().map_err(|_| "stage-sync-failed")?;
        file.seek(SeekFrom::Start(0)).map_err(|_| "stage-seek-failed")?;
        let mut hash = Sha256::new();
        let mut count = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer).map_err(|_| "stage-read-failed")?;
            if n == 0 { break; }
            count += n as u64;
            if count > MAX_OUTPUT { return Err("stage-size-limit"); }
            hash.update(&buffer[..n]);
        }
        let digest: [u8; 32] = hash.finalize().into();
        if count != receipt.byte_size || digest != receipt.executable_sha256 { return Err("stage-content-mismatch"); }
        if !self.parents_owned() || !self.file_id.as_ref().is_some_and(|id| same_regular(&self.path, id, false)) {
            return Err("stage-ownership-lost");
        }
        Ok(())
    }
}

impl Drop for OwnedStage {
    fn drop(&mut self) {
        self.file.take();
        if !self.parents_owned() { return; }
        if let Some(id) = &self.file_id {
            if !same_regular(&self.path, id, false) { return; }
            if fs::remove_file(&self.path).is_err() { return; }
        }
        // Never recursive cleanup: an unexpected file prevents directory removal.
        if self.parents_owned() { let _ = fs::remove_dir(&self.directory); }
    }
}

pub(super) struct StagedDeno { owned: OwnedStage, receipt: Receipt }
impl StagedDeno {
    pub(super) fn path(&self) -> &Path { &self.owned.path }
    pub(super) fn receipt(&self) -> &Receipt { &self.receipt }
}

/// Backend-owned inputs only. No runtime activation; result owns output until dropped.
pub(super) fn extract_to_stage(root: &Path, bytes: &[u8], target: DenoTarget)
    -> Result<StagedDeno, &'static str>
{
    stage_with(root, super::deno_zip::executable_name(target), |output| {
        super::deno_zip::extract(bytes, target, output)
    })
}

fn stage_with(root: &Path, name: &str, extract: impl FnOnce(&mut File) -> Result<Receipt, &'static str>)
    -> Result<StagedDeno, &'static str>
{
    // Fixed backend filename; this private helper also supports synthetic tests.
    if !matches!(name, "deno" | "deno.exe") { return Err("stage-name-invalid"); }
    let mut stage = OwnedStage::create(root, name)?;
    let receipt = extract(stage.file.as_mut().ok_or("stage-file-missing")?)?;
    stage.verify(&receipt)?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        stage.file.as_ref().ok_or("stage-file-missing")?
            .set_permissions(fs::Permissions::from_mode(0o700)).map_err(|_| "stage-permissions")?;
    }
    stage.verify(&receipt)?;
    Ok(StagedDeno { owned: stage, receipt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!("pulqva-deno-stage-test-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); fs::canonicalize(path).unwrap()
    }
    fn receipt() -> Receipt {
        Receipt { version: "fixture", target: "fixture", archive_sha256: [0; 32],
            executable_sha256: Sha256::digest(b"verified").into(), byte_size: 8 }
    }
    #[test]
    fn verified_receipt_matches_disk_and_drop_cleans() {
        let root = root();
        let stage = stage_with(&root, "deno", |f| { f.write_all(b"verified").unwrap(); Ok(receipt()) }).unwrap();
        assert_eq!(stage.receipt(), &receipt());
        assert_eq!(fs::read(stage.path()).unwrap(), b"verified");
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(stage.path()).unwrap().permissions().mode() & 0o777, 0o700);
        }
        drop(stage); assert_eq!(fs::read_dir(&root).unwrap().count(), 0); fs::remove_dir(root).unwrap();
    }
    #[test]
    fn partial_failure_and_bad_receipt_clean_only_owned_files() {
        let root = root(); fs::write(root.join("keep"), b"user").unwrap();
        assert!(stage_with(&root, "deno", |f| { f.write_all(b"partial").unwrap(); Err("injected") }).is_err());
        assert!(stage_with(&root, "deno", |f| { f.write_all(b"modified").unwrap(); Ok(receipt()) }).is_err());
        assert!(extract_to_stage(&root, b"bad zip", DenoTarget::LinuxX64).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert_eq!(fs::read(root.join("keep")).unwrap(), b"user");
        fs::remove_file(root.join("keep")).unwrap(); fs::remove_dir(root).unwrap();
    }
    #[test]
    fn rejects_invalid_root_and_name() {
        assert!(stage_with(Path::new("relative"), "deno", |_| Ok(receipt())).is_err());
        let root = root();
        assert!(stage_with(&root, "../deno", |_| Ok(receipt())).is_err());
        assert!(stage_with(&root.join("missing"), "deno", |_| Ok(receipt())).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0); fs::remove_dir(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn substituted_output_is_rejected_and_never_deleted() {
        let root = root(); let mut stage = OwnedStage::create(&root, "deno").unwrap();
        let directory = stage.directory.clone(); let path = stage.path.clone(); let saved = directory.join("saved");
        fs::rename(&path, &saved).unwrap(); fs::write(&path, b"foreign").unwrap();
        assert_eq!(stage.verify(&receipt()).unwrap_err(), "stage-ownership-lost");
        drop(stage); assert_eq!(fs::read(&path).unwrap(), b"foreign");
        fs::remove_file(path).unwrap(); fs::remove_file(saved).unwrap();
        fs::remove_dir(directory).unwrap(); fs::remove_dir(root).unwrap();
    }
}
