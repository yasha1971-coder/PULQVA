//! Owned temporary output for extraction. No runtime publication or source-file opening.
#![allow(dead_code)]
use super::ffmpeg_zip::Receipt;
use same_file::Handle;
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Seek, SeekFrom}, path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering}};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_OUTPUT: u64 = 512 * 1024 * 1024;

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
    fn create(root: &Path) -> Result<Self, &'static str> {
        if !root.is_absolute() || fs::canonicalize(root).map_err(|_| "stage-root-invalid")? != root {
            return Err("stage-root-not-canonical");
        }
        let root_id = Handle::from_path(root).map_err(|_| "stage-root-invalid")?;
        if !same_regular(root, &root_id, true) { return Err("stage-root-invalid"); }
        for _ in 0..64 {
            let directory = root.join(format!(".ffmpeg-stage-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
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
            let path = directory.join("ffmpeg.part");
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

pub(super) struct StagedFfmpeg { owned: OwnedStage, receipt: Receipt }
impl StagedFfmpeg {
    pub(super) fn path(&self) -> &Path { &self.owned.path }
    pub(super) fn receipt(&self) -> &Receipt { &self.receipt }
}

/// Root and asset/digest are backend-owned inputs. Result owns output until dropped.
pub(super) fn extract_to_stage(root: &Path, bytes: &[u8], digest: [u8; 32], asset: &str)
    -> Result<StagedFfmpeg, &'static str>
{
    let mut stage = OwnedStage::create(root)?;
    let output = stage.file.as_mut().ok_or("stage-file-missing")?;
    let receipt = if asset.ends_with(".zip") {
        super::ffmpeg_zip::extract(bytes, digest, asset, output)?
    } else if asset.ends_with(".tar.xz") {
        super::ffmpeg_tar_xz::extract(bytes, digest, asset, output)?
    } else { return Err("unsupported-archive-kind"); };
    stage.verify(&receipt)?;
    Ok(StagedFfmpeg { owned: stage, receipt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!("pulqva-stage-test-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); fs::canonicalize(path).unwrap()
    }
    fn zip() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("package/bin/ffmpeg.exe", SimpleFileOptions::default()).unwrap();
        writer.write_all(b"verified executable fixture").unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn successful_stage_is_verified_then_removed_on_drop() {
        let root = root(); let bytes = zip();
        let stage = extract_to_stage(&root, &bytes, Sha256::digest(&bytes).into(), "package.zip").unwrap();
        assert_eq!(fs::read(stage.path()).unwrap(), b"verified executable fixture");
        assert_eq!(stage.receipt().byte_size, 27);
        let path = stage.path().to_owned(); drop(stage);
        assert!(!path.exists()); assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn errors_clean_only_owned_stage_and_preserve_existing_files() {
        let root = root(); fs::write(root.join("keep"), b"user data").unwrap();
        let bytes = zip();
        assert!(extract_to_stage(&root, &bytes, [0; 32], "package.zip").is_err());
        assert!(extract_to_stage(&root, b"bad zip", Sha256::digest(b"bad zip").into(), "package.zip").is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert_eq!(fs::read(root.join("keep")).unwrap(), b"user data");
        fs::remove_file(root.join("keep")).unwrap(); fs::remove_dir(root).unwrap();
    }

    #[test]
    fn reread_rejects_receipt_that_does_not_match_disk() {
        let root = root(); let mut stage = OwnedStage::create(&root).unwrap();
        stage.file.as_mut().unwrap().write_all(b"changed").unwrap();
        assert!(stage.verify(&Receipt { archive_sha256: [0; 32], executable_sha256: [0; 32], byte_size: 7 }).is_err());
        drop(stage); assert_eq!(fs::read_dir(&root).unwrap().count(), 0); fs::remove_dir(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn substituted_file_is_preserved_by_cleanup() {
        let root = root(); let stage = OwnedStage::create(&root).unwrap();
        let directory = stage.directory.clone(); let path = stage.path.clone(); let saved = directory.join("saved");
        fs::rename(&path, &saved).unwrap(); fs::write(&path, b"foreign").unwrap();
        drop(stage); assert_eq!(fs::read(&path).unwrap(), b"foreign");
        fs::remove_file(path).unwrap(); fs::remove_file(saved).unwrap();
        fs::remove_dir(directory).unwrap(); fs::remove_dir(root).unwrap();
    }
}
