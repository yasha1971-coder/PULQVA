//! Authenticated immutable ZIP bytes -> bounded internal writer; filesystem ownership is separate.
#![allow(dead_code)] // T060-B is not wired into production until owned staging is implemented.
use super::ffmpeg_archive_policy::{Kind, Policy};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use zip::{CompressionMethod, ZipArchive};

const MAX_ARCHIVE: usize = 256 * 1024 * 1024;
const MAX_EXECUTABLE: u64 = 512 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct Receipt {
    pub(super) archive_sha256: [u8; 32],
    pub(super) executable_sha256: [u8; 32],
    pub(super) byte_size: u64,
}

// Reject ZIP64/multipart/prefixed/trailing layouts. Check entry count BEFORE library allocation.
fn entry_count(bytes: &[u8]) -> Result<usize, &'static str> {
    if bytes.len() < 22 { return Err("truncated-zip"); }
    let start = bytes.len().saturating_sub(22 + 65_535);
    let end = (start..=bytes.len() - 22).rev().find(|&p| {
        bytes[p..p + 4] == *b"PK\x05\x06"
            && p + 22 + u16::from_le_bytes([bytes[p + 20], bytes[p + 21]]) as usize == bytes.len()
    }).ok_or("missing-zip-end")?;
    let u16_at = |p| u16::from_le_bytes([bytes[p], bytes[p + 1]]) as usize;
    let u32_at = |p| u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
    let count = u16_at(end + 10);
    if u16_at(end + 4) != 0 || u16_at(end + 6) != 0 || u16_at(end + 8) != count
        || count == 0 || count > 16_384
        || u32_at(end + 16).checked_add(u32_at(end + 12)) != Some(end)
        || bytes.get(..4) != Some(b"PK\x03\x04")
    { return Err("unsupported-zip-layout"); }
    Ok(count)
}

/// Expected digest and asset must come from backend pinned metadata, never provider/frontend input.
/// The writer may contain partial bytes on failure; caller must own and discard that staging output.
pub(super) fn extract(
    bytes: &[u8], expected_digest: [u8; 32], asset: &str, output: &mut impl Write,
) -> Result<Receipt, &'static str> {
    if bytes.len() > MAX_ARCHIVE || !asset.ends_with(".zip") { return Err("archive-limit-or-kind"); }
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if digest != expected_digest { return Err("archive-identity-mismatch"); }
    let count = entry_count(bytes)?;
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "invalid-zip")?;
    // zip's name map may collapse duplicates; do not silently accept the collapsed view.
    if archive.len() != count { return Err("ambiguous-entry-count"); }
    let mut policy = Policy::new(asset)?;
    let mut selected = None;
    for index in 0..count {
        let file = archive.by_index_raw(index).map_err(|_| "invalid-member")?;
        if file.encrypted() { return Err("encrypted-member"); }
        let name = std::str::from_utf8(file.name_raw()).map_err(|_| "non-utf8-member")?;
        let mode = file.unix_mode().unwrap_or(0) & 0o170000;
        let kind = match mode {
            0 if file.is_dir() => Kind::Directory,
            0 => Kind::File,
            0o040000 if file.is_dir() => Kind::Directory,
            0o100000 if !file.is_dir() => Kind::File,
            0o120000 => Kind::Link,
            _ => Kind::Special,
        };
        if policy.observe(name, kind, file.size())? {
            if !matches!(file.compression(), CompressionMethod::Stored | CompressionMethod::Deflated) {
                return Err("unsupported-compression");
            }
            selected = Some((index, file.size()));
        }
    }
    policy.finish()?;
    let (index, declared) = selected.ok_or("missing-executable")?;
    let mut file = archive.by_index(index).map_err(|_| "invalid-executable")?;
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|_| "decompression-or-crc-error")?;
        if n == 0 { break; }
        size = size.checked_add(n as u64).ok_or("executable-limit")?;
        if size > declared || size > MAX_EXECUTABLE { return Err("executable-limit"); }
        output.write_all(&buffer[..n]).map_err(|_| "staging-write-failed")?;
        hash.update(&buffer[..n]);
    }
    if size != declared { return Err("executable-size-mismatch"); }
    Ok(Receipt { archive_sha256: digest, executable_sha256: hash.finalize().into(), byte_size: size })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn fixture(name: &str, method: CompressionMethod) -> Vec<u8> {
        let mut z = ZipWriter::new(Cursor::new(Vec::new()));
        z.start_file(name, SimpleFileOptions::default().compression_method(method)).unwrap();
        z.write_all(b"ffmpeg test bytes").unwrap();
        z.finish().unwrap().into_inner()
    }

    #[test]
    fn stored_and_deflated_bytes_are_authenticated_and_extracted() {
        for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
            let bytes = fixture("package/bin/ffmpeg.exe", method);
            let digest = Sha256::digest(&bytes).into();
            let mut out = Vec::new();
            let receipt = extract(&bytes, digest, "package.zip", &mut out).unwrap();
            assert_eq!(out, b"ffmpeg test bytes");
            assert_eq!(receipt.byte_size, out.len() as u64);
            assert_eq!(receipt.archive_sha256, digest);
            assert_eq!(receipt.executable_sha256, <[u8; 32]>::from(Sha256::digest(&out)));
        }
    }

    #[test]
    fn identity_failure_and_unsafe_names_leave_output_untouched() {
        let bytes = fixture("package/bin/ffmpeg.exe", CompressionMethod::Stored);
        let mut out = Vec::new();
        assert_eq!(extract(&bytes, [0; 32], "package.zip", &mut out).unwrap_err(), "archive-identity-mismatch");
        assert!(out.is_empty());
        for name in ["../ffmpeg.exe", "package/bin/ffmpeg.exe:ads", "other/bin/ffmpeg.exe"] {
            let bytes = fixture(name, CompressionMethod::Stored);
            assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.zip", &mut out).is_err());
            assert!(out.is_empty());
        }
    }

    #[test]
    fn rejects_truncation_and_corrupt_payload_even_with_matching_archive_hash() {
        let mut bytes = fixture("package/bin/ffmpeg.exe", CompressionMethod::Stored);
        let pos = bytes.windows(17).position(|w| w == b"ffmpeg test bytes").unwrap();
        bytes[pos] ^= 1;
        assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.zip", &mut Vec::new()).is_err());
        bytes.truncate(bytes.len() - 1);
        assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.zip", &mut Vec::new()).is_err());
    }
}
