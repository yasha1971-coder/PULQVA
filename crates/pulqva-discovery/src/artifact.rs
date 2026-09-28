//! C2 post-download verification. No sockets, subprocesses, or file writes.
//!
//! SHA-1 is only compatibility with untrusted Commons metadata; SHA-256 identifies
//! the bytes read locally. Neither authenticates a publisher or proves Tor routing.
//! The caller must retain exclusive ownership of the output directory. This is a
//! content snapshot, not a race-proof path capability or a media safety scanner.
use crate::{CommonsSearch, CommonsTransport, DiscoveredMedia, MAX_MEDIA_BYTES};
use pulqva_core::{FileReceipt, SelectedCandidate};
use pulqva_privacy::CompletedDownloadResult;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{fmt, fs::{self, File}, io::{self, Read}, path::{Component, Path}};

/// Constructed only after selection, source, size and content checks succeed.
/// Do not treat a plain core FileReceipt as equivalent to this verified snapshot.
///
/// ```compile_fail
/// use pulqva_discovery::VerifiedCommonsFile;
/// let forged = VerifiedCommonsFile {
///     receipt: todo!(), page_id: 1, sha1: String::new(), sha256: String::new()
/// };
/// ```
pub struct VerifiedCommonsFile {
    receipt: FileReceipt,
    page_id: u64,
    sha1: String,
    sha256: String,
}
impl VerifiedCommonsFile {
    pub fn receipt(&self) -> &FileReceipt { &self.receipt }
    pub fn page_id(&self) -> u64 { self.page_id }
    pub fn sha1(&self) -> &str { &self.sha1 }
    pub fn sha256(&self) -> &str { &self.sha256 }
    pub fn into_receipt(self) -> FileReceipt { self.receipt }
}
impl fmt::Debug for VerifiedCommonsFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Paths, titles, provider identifiers and digests are not default logs.
        f.write_str("VerifiedCommonsFile { .. }")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactVerificationError {
    SelectionMismatch, SourceMismatch, InvalidMetadata, UnsupportedArtifact,
    SizeMismatch, DigestMismatch, Io, Receipt,
}
impl fmt::Display for ArtifactVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Commons artifact verification: {self:?}")
    }
}
impl std::error::Error for ArtifactVerificationError {}

impl<T: CommonsTransport> CommonsSearch<T> {
    /// Verify an existing typed download against the retained search metadata.
    /// Does not launch a download or change the current fail-closed retrieve().
    /// Index + candidate equality is value binding, NOT a search-session nonce.
    pub fn verify_completed_download(
        &self,
        selection: &SelectedCandidate,
        completed: &CompletedDownloadResult,
    ) -> Result<VerifiedCommonsFile, ArtifactVerificationError> {
        verify_retained(self.last_results(), selection, completed.source_url(),
                        completed.artifact_path(), completed.byte_size())
    }
}

// Private input seam for deterministic fixtures. Public callers must supply the
// existing CompletedDownloadResult; there is no public arbitrary-path constructor.
fn verify_retained(
    retained: &[DiscoveredMedia],
    selection: &SelectedCandidate,
    completed_source: &str,
    path: &Path,
    completed_size: u64,
) -> Result<VerifiedCommonsFile, ArtifactVerificationError> {
    use ArtifactVerificationError::*;
    let media = retained.get(selection.index()).ok_or(SelectionMismatch)?;
    if !media.matches_selection(selection) { return Err(SelectionMismatch); }
    if completed_source != media.candidate().locator() { return Err(SourceMismatch); }
    let expected = media.declared_size();
    if expected == 0 || expected > MAX_MEDIA_BYTES
        || media.declared_sha1().len() != 40
        || !media.declared_sha1().bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(InvalidMetadata);
    }
    if completed_size != expected { return Err(SizeMismatch); }
    if path.components().any(|part| matches!(part, Component::ParentDir)) {
        return Err(UnsupportedArtifact);
    }
    // Reject observable symlinks/special files; opening and checking metadata is
    // not an atomic no-follow operation. Existing T066 ownership/race limits stay.
    let entry = fs::symlink_metadata(path).map_err(|_| Io)?;
    if !entry.file_type().is_file() { return Err(UnsupportedArtifact); }
    if entry.len() != expected { return Err(SizeMismatch); }
    let mut file = File::open(path).map_err(|_| Io)?;
    let before = file.metadata().map_err(|_| Io)?;
    if !before.is_file() { return Err(UnsupportedArtifact); }
    if before.len() != expected { return Err(SizeMismatch); }
    let (sha1, sha256) = hash_exact(&mut file, expected)?;
    let after = file.metadata().map_err(|_| Io)?;
    if !after.is_file() || after.len() != expected { return Err(SizeMismatch); }
    if !sha1.eq_ignore_ascii_case(media.declared_sha1()) { return Err(DigestMismatch); }
    let receipt = FileReceipt::new(selection, path, expected).map_err(|_| Receipt)?;
    Ok(VerifiedCommonsFile { receipt, page_id: media.page_id(), sha1, sha256 })
}

// One pass, fixed 64 KiB buffer; read at most expected+1 bytes to detect overflow.
// This is a byte/memory bound, not a guarantee of local filesystem I/O latency.
fn hash_exact(reader: &mut impl Read, expected: u64)
    -> Result<(String, String), ArtifactVerificationError>
{
    use ArtifactVerificationError::*;
    if expected == 0 || expected > MAX_MEDIA_BYTES { return Err(InvalidMetadata); }
    let mut sha1 = Sha1::new();
    let mut sha256 = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let want = (expected - total + 1).min(buffer.len() as u64) as usize;
        let count = match reader.read(&mut buffer[..want]) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(Io),
        };
        if count == 0 { break; }
        total += count as u64;
        if total > expected { return Err(SizeMismatch); }
        sha1.update(&buffer[..count]);
        sha256.update(&buffer[..count]);
    }
    if total != expected { return Err(SizeMismatch); }
    Ok((lower_hex(sha1.finalize().as_ref()), lower_hex(sha256.finalize().as_ref())))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests;
