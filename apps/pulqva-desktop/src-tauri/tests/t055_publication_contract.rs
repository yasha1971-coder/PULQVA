//! T055 PREPARE: executable proof of publication primitives, NOT the production materializer.
//! The test-only helper assumes an owned, stable temporary directory. It does not implement
//! capability binding, alias/containment validation, Windows reparse defense, or live IPC.

use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Barrier, atomic::{AtomicU64, Ordering}},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "pulqva-t055-proof-{}-{time}-{sequence}", std::process::id()
        ));
        // Never adopt or clean a directory that we did not exclusively create.
        fs::create_dir(&path).expect("exclusive fixture directory creation");
        Self(path)
    }

    fn path(&self) -> &Path { &self.0 }
}

impl Drop for TestDirectory {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

struct TestStage {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for TestStage {
    fn drop(&mut self) {
        self.file.take();
        // Test-owned stable directory only; production cleanup must retain stronger ownership.
        let _ = fs::remove_file(&self.path);
    }
}

fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn stage_for_proof(
    directory: &Path,
    mut source: impl Read,
    expected: &str,
) -> io::Result<TestStage> {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = directory.join(format!(".pulqva-stage-{sequence}"));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    // The guard exists only after exclusive creation succeeds.
    let mut stage = TestStage { path, file: Some(file) };
    let file = stage.file.as_mut().unwrap();
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut byte_size = 0_u64;
    loop {
        let count = match source.read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 { break; }
        file.write_all(&buffer[..count])?;
        hasher.update(&buffer[..count]);
        byte_size += count as u64;
    }
    if byte_size == 0 || format!("{:x}", hasher.finalize()) != expected {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "copied-byte digest mismatch"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o700))?;
    }
    file.sync_all()?;
    // Closing before publication also makes test cleanup deterministic on Windows.
    stage.file.take();
    Ok(stage)
}

fn has_staging_files(directory: &Path) -> bool {
    fs::read_dir(directory).unwrap().any(|entry| {
        entry.unwrap().file_name().to_string_lossy().starts_with(".pulqva-stage-")
    })
}

#[test]
fn publishes_complete_verified_bytes_and_removing_stage_keeps_destination() {
    let root = TestDirectory::new();
    let payload = vec![0x5a_u8; 200_003]; // Cross multiple bounded-buffer reads.
    let stage = stage_for_proof(root.path(), Cursor::new(&payload), &digest(&payload)).unwrap();
    let destination = root.path().join("yt-dlp");
    assert!(!destination.exists());
    fs::hard_link(&stage.path, &destination).expect("no-clobber publication");
    assert_eq!(fs::read(&destination).unwrap(), payload);
    drop(stage);
    assert!(!has_staging_files(root.path()));
    assert_eq!(fs::read(destination).unwrap(), payload);
}

#[test]
fn publication_cannot_replace_an_existing_destination() {
    let root = TestDirectory::new();
    let destination = root.path().join("yt-dlp");
    fs::write(&destination, b"existing unrelated bytes").unwrap();
    let stage = stage_for_proof(root.path(), Cursor::new(b"new binary"), &digest(b"new binary")).unwrap();
    assert!(fs::hard_link(&stage.path, &destination).is_err());
    drop(stage);
    assert_eq!(fs::read(destination).unwrap(), b"existing unrelated bytes");
    assert!(!has_staging_files(root.path()));
}

#[test]
fn exclusive_stage_creation_does_not_truncate_a_preexisting_file() {
    let root = TestDirectory::new();
    let occupied = root.path().join("occupied-stage");
    fs::write(&occupied, b"not owned by new operation").unwrap();
    assert!(OpenOptions::new().write(true).create_new(true).open(&occupied).is_err());
    assert_eq!(fs::read(occupied).unwrap(), b"not owned by new operation");
}

#[test]
fn digest_failure_removes_own_stage_but_preserves_other_files() {
    let root = TestDirectory::new();
    let sentinel = root.path().join("unrelated-file");
    fs::write(&sentinel, b"preserve").unwrap();
    let error = match stage_for_proof(root.path(), Cursor::new(b"changed"), &digest(b"expected")) {
        Ok(_) => panic!("mismatching bytes must not become publishable"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(!has_staging_files(root.path()));
    assert!(!root.path().join("yt-dlp").exists());
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
}

#[test]
fn a_source_changed_after_an_earlier_receipt_is_not_trusted() {
    let root = TestDirectory::new();
    let source = root.path().join("packaged-source");
    fs::write(&source, b"original").unwrap();
    let earlier_receipt = digest(&fs::read(&source).unwrap());
    fs::write(&source, b"changed after receipt").unwrap();
    assert!(stage_for_proof(root.path(), File::open(&source).unwrap(), &earlier_receipt).is_err());
    assert!(!has_staging_files(root.path()));
    assert!(!root.path().join("yt-dlp").exists());
    assert_eq!(fs::read(source).unwrap(), b"changed after receipt");
}

struct FailingReader { supplied_first_chunk: bool }
impl Read for FailingReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if !self.supplied_first_chunk {
            self.supplied_first_chunk = true;
            output[0] = b'a';
            return Ok(1);
        }
        Err(io::Error::other("injected read failure"))
    }
}

#[test]
fn interrupted_copy_leaves_no_partial_destination_or_owned_stage() {
    let root = TestDirectory::new();
    assert!(stage_for_proof(
        root.path(), FailingReader { supplied_first_chunk: false }, &digest(b"abc")
    ).is_err());
    assert!(!has_staging_files(root.path()));
    assert!(!root.path().join("yt-dlp").exists());
}

#[test]
fn concurrent_publishers_have_one_winner_and_never_mix_bytes() {
    let root = TestDirectory::new();
    let first = vec![b'a'; 140_001];
    let second = vec![b'b'; 150_007];
    let stage_a = stage_for_proof(root.path(), Cursor::new(&first), &digest(&first)).unwrap();
    let stage_b = stage_for_proof(root.path(), Cursor::new(&second), &digest(&second)).unwrap();
    let destination = root.path().join("yt-dlp");
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for stage in [stage_a, stage_b] {
        let target = destination.clone();
        let gate = barrier.clone();
        workers.push(thread::spawn(move || {
            gate.wait();
            let outcome = fs::hard_link(&stage.path, target);
            drop(stage);
            outcome
        }));
    }
    let outcomes: Vec<_> = workers.into_iter().map(|worker| worker.join().unwrap()).collect();
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_err()).count(), 1);
    let published = fs::read(destination).unwrap();
    assert!(published == first || published == second);
    assert!(!has_staging_files(root.path()));
}

#[cfg(unix)]
#[test]
fn existing_symlink_and_dangling_stage_are_never_clobbered() {
    use std::os::unix::fs::symlink;
    let root = TestDirectory::new();
    let target = root.path().join("unrelated-target");
    let destination = root.path().join("yt-dlp");
    fs::write(&target, b"preserve target").unwrap();
    symlink(&target, &destination).unwrap();
    let stage = stage_for_proof(root.path(), Cursor::new(b"abc"), &digest(b"abc")).unwrap();
    assert!(fs::hard_link(&stage.path, &destination).is_err());
    assert!(fs::symlink_metadata(&destination).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(&target).unwrap(), b"preserve target");
    let dangling = root.path().join("dangling-stage");
    symlink(root.path().join("missing-target"), &dangling).unwrap();
    assert!(OpenOptions::new().write(true).create_new(true).open(&dangling).is_err());
    assert!(!root.path().join("missing-target").exists());
}

#[cfg(unix)]
#[test]
fn executable_permissions_are_applied_only_to_the_owned_stage() {
    use std::os::unix::fs::PermissionsExt;
    let root = TestDirectory::new();
    let source = root.path().join("package-file");
    fs::write(&source, b"abc").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).unwrap();
    let stage = stage_for_proof(root.path(), File::open(&source).unwrap(), &digest(b"abc")).unwrap();
    assert_eq!(fs::metadata(&stage.path).unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(fs::metadata(&source).unwrap().permissions().mode() & 0o777, 0o640);
}
