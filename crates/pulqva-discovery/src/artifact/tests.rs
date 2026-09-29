//! Synthetic file/API fixtures, never represented as live downloads.
use super::*;
use pulqva_core::{ChoiceSet, SearchIntent};
use std::{io::Cursor, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";
const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("pulqva c2 hash {}-{}-{}",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, bytes: &[u8]) -> PathBuf {
        let path = self.0.join("selected.webm");
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Temp {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}
fn metadata() -> (Vec<DiscoveredMedia>, ChoiceSet) {
    let pages: Vec<_> = (1..=2).map(|id| serde_json::json!({
        "pageid":id, "ns":6, "title":format!("File:fixture-{id}.webm"),
        "imageinfo":[{"url":format!("https://upload.wikimedia.org/wikipedia/commons/a/ab/{id}.webm"),
            "size":3, "sha1":SHA1_ABC, "mime":"video/webm"}]
    })).collect();
    let body = serde_json::to_vec(&serde_json::json!({"query":{"pages":pages}})).unwrap();
    let media = crate::parse_response(&body).unwrap();
    let choices = ChoiceSet::new(SearchIntent::new("synthetic fixture").unwrap(),
        media.iter().map(|m| m.candidate().clone()).collect()).unwrap();
    (media, choices)
}

#[test]
fn verified_receipt_uses_actual_file_hashes_and_selected_metadata() {
    let root = Temp::new();
    let path = root.file(b"abc");
    let (media, choices) = metadata();
    let selected = choices.select(1).unwrap();
    let result = verify_retained(&media, &selected, media[1].candidate().locator(), &path, 3).unwrap();
    assert_eq!(result.sha1(), SHA1_ABC);
    assert_eq!(result.sha256(), SHA256_ABC);
    assert_eq!(result.page_id(), 2);
    assert_eq!(result.receipt().selected_index(), 1);
    assert_eq!(result.receipt().title(), selected.candidate().title());
    assert_eq!(result.receipt().path(), path);
    assert_eq!(result.receipt().byte_size(), 3);
    assert_eq!(format!("{result:?}"), "VerifiedCommonsFile { .. }");
    assert_eq!(result.into_receipt().byte_size(), 3);
}

#[test]
fn rejects_empty_stale_reordered_foreign_and_wrong_source_before_file_io() {
    let (media, choices) = metadata();
    let selected = choices.select(1).unwrap();
    let source = media[1].candidate().locator();
    let nowhere = Path::new("");
    for retained in [&media[..0], &media[..1]] {
        assert_eq!(verify_retained(retained, &selected, source, nowhere, 3).unwrap_err(),
                   ArtifactVerificationError::SelectionMismatch);
    }
    let mut reordered = media.clone();
    reordered.reverse();
    assert_eq!(verify_retained(&reordered, &selected, source, nowhere, 3).unwrap_err(),
               ArtifactVerificationError::SelectionMismatch);
    let mut foreign = choices.candidates().to_vec();
    foreign[1] = pulqva_core::SearchCandidate::new("Foreign", "https://example.invalid/file").unwrap();
    let foreign = ChoiceSet::new(SearchIntent::new("foreign").unwrap(), foreign).unwrap();
    assert_eq!(verify_retained(&media, &foreign.select(1).unwrap(), source, nowhere, 3).unwrap_err(),
               ArtifactVerificationError::SelectionMismatch);
    assert_eq!(verify_retained(&media, &selected, media[0].candidate().locator(), nowhere, 3).unwrap_err(),
               ArtifactVerificationError::SourceMismatch);
    assert_eq!(verify_retained(&media, &selected, source, nowhere, 4).unwrap_err(),
               ArtifactVerificationError::SizeMismatch);
}

#[test]
fn same_size_corruption_cannot_mint_a_verified_receipt() {
    let root = Temp::new();
    let path = root.file(b"abd");
    let (media, choices) = metadata();
    let error = verify_retained(&media, &choices.select(1).unwrap(),
        media[1].candidate().locator(), &path, 3).unwrap_err();
    assert_eq!(error, ArtifactVerificationError::DigestMismatch);
    assert_eq!(fs::read(&path).unwrap(), b"abd", "verifier never rewrites rejected files");
}

#[test]
fn file_metadata_is_checked_again_instead_of_trusting_old_completion_size() {
    let root = Temp::new();
    let (media, choices) = metadata();
    for bytes in [b"".as_slice(), b"ab", b"abcd"] {
        let path = root.file(bytes);
        assert_eq!(verify_retained(&media, &choices.select(1).unwrap(),
            media[1].candidate().locator(), &path, 3).unwrap_err(), ArtifactVerificationError::SizeMismatch);
    }
    assert_eq!(verify_retained(&media, &choices.select(1).unwrap(),
        media[1].candidate().locator(), &root.0, 3).unwrap_err(), ArtifactVerificationError::UnsupportedArtifact);
    assert_eq!(verify_retained(&media, &choices.select(1).unwrap(),
        media[1].candidate().locator(), &root.0.join("missing"), 3).unwrap_err(), ArtifactVerificationError::Io);
}

#[test]
fn streams_are_bounded_and_truncation_is_not_success() {
    assert_eq!(hash_exact(&mut Cursor::new(b"abc"), 3).unwrap(),
               (SHA1_ABC.to_owned(), SHA256_ABC.to_owned()));
    for bytes in [b"".as_slice(), b"ab"] {
        assert_eq!(hash_exact(&mut Cursor::new(bytes), 3), Err(ArtifactVerificationError::SizeMismatch));
    }
    let mut oversized = Cursor::new(vec![b'a'; 1_000_000]);
    assert_eq!(hash_exact(&mut oversized, 3), Err(ArtifactVerificationError::SizeMismatch));
    assert_eq!(oversized.position(), 4, "never read beyond declared size plus one byte");
    for size in [0, MAX_MEDIA_BYTES + 1, u64::MAX] {
        let mut unread = Cursor::new(b"abc");
        assert_eq!(hash_exact(&mut unread, size), Err(ArtifactVerificationError::InvalidMetadata));
        assert_eq!(unread.position(), 0);
    }
}

#[test]
fn partial_reads_and_interrupted_io_produce_the_same_hashes() {
    struct Partial { bytes: Cursor<&'static [u8]>, interrupt: bool }
    impl Read for Partial {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.interrupt { self.interrupt = false; return Err(io::ErrorKind::Interrupted.into()); }
            let count = output.len().min(1);
            self.bytes.read(&mut output[..count])
        }
    }
    let mut reader = Partial { bytes: Cursor::new(b"abc".as_slice()), interrupt: true };
    assert_eq!(hash_exact(&mut reader, 3).unwrap(), (SHA1_ABC.to_owned(), SHA256_ABC.to_owned()));
}

#[test]
fn io_failure_does_not_leak_messages_paths_or_partial_hashes() {
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("PRIVATE path and provider query"))
        }
    }
    let error = hash_exact(&mut Broken, 3).unwrap_err();
    assert_eq!(error, ArtifactVerificationError::Io);
    assert!(!format!("{error:?}: {error}").contains("PRIVATE"));
    assert!(std::error::Error::source(&error).is_none());
}

#[cfg(unix)]
#[test]
fn observable_symlink_is_rejected_without_following_it() {
    let root = Temp::new();
    let target = root.file(b"abc");
    let link = root.0.join("link.webm");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let (media, choices) = metadata();
    assert_eq!(verify_retained(&media, &choices.select(1).unwrap(),
        media[1].candidate().locator(), &link, 3).unwrap_err(), ArtifactVerificationError::UnsupportedArtifact);
}

#[test]
fn streaming_matches_independent_million_byte_vectors() {
    let mut reader = io::repeat(b'a').take(1_000_000);
    let result = hash_exact(&mut reader, 1_000_000).unwrap();
    assert_eq!(result.0, "34aa973cd4c4daa4f61eeb2bdbad27316534016f");
    assert_eq!(result.1, "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
}
