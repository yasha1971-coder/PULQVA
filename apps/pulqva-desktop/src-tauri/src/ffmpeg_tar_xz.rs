//! T060-C: authenticated tar.xz bytes to an internal writer. No filesystem ownership yet.
#![allow(dead_code)]
use super::ffmpeg_archive_policy::{Kind, Policy};
use super::ffmpeg_zip::Receipt;
use sha2::{Digest, Sha256};
use std::io::{self, Cursor, Read, Write};
use xz2::{read::XzDecoder, stream::Stream};

const MAX_SOURCE: usize = 256 * 1024 * 1024;
const MAX_EXPANDED: u64 = 2 * 1024 * 1024 * 1024;
const DECODER_MEMORY: u64 = 256 * 1024 * 1024;

struct Budget<R> { inner: R, remaining: u64 }
impl<R: Read> Read for Budget<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() { return Ok(0); }
        if self.remaining == 0 {
            let mut probe = [0];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::other("tar expansion limit")),
            };
        }
        let limit = out.len().min(self.remaining.min(usize::MAX as u64) as usize);
        let n = self.inner.read(&mut out[..limit])?;
        self.remaining -= n as u64;
        Ok(n)
    }
}

/// Caller supplies backend pinned identity and must discard partial writer contents on failure.
pub(super) fn extract(
    bytes: &[u8], expected: [u8; 32], asset: &str, output: &mut impl Write,
) -> Result<Receipt, &'static str> {
    if bytes.len() > MAX_SOURCE || !asset.ends_with(".tar.xz") { return Err("archive-limit-or-kind"); }
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if digest != expected { return Err("archive-identity-mismatch"); }
    let stream = Stream::new_stream_decoder(DECODER_MEMORY, 0).map_err(|_| "xz-decoder-init")?;
    let decoder = XzDecoder::new_stream(Cursor::new(bytes), stream);
    let mut archive = tar::Archive::new(Budget { inner: decoder, remaining: MAX_EXPANDED });
    let mut policy = Policy::new(asset)?;
    let mut selected_size = 0;
    let mut hash = Sha256::new();
    {
        // Expose extension records to policy instead of implicitly allocating their payloads.
        let entries = archive.entries().map_err(|_| "invalid-tar")?.raw(true);
        for entry in entries {
            let mut entry = entry.map_err(|_| "invalid-tar-member")?;
            let ty = entry.header().entry_type();
            let kind = if ty.is_file() { Kind::File } else if ty.is_dir() { Kind::Directory }
                else if ty.is_symlink() || ty.is_hard_link() { Kind::Link } else { Kind::Special };
            let declared = entry.size();
            let selected = {
                let path = entry.path_bytes();
                let name = std::str::from_utf8(&path).map_err(|_| "non-utf8-member")?;
                policy.observe(name, kind, declared)?
            };
            let mut actual = 0u64;
            let mut buffer = [0u8; 64 * 1024];
            // Consume even nonselected members through the global decompression budget.
            loop {
                let n = entry.read(&mut buffer).map_err(|_| "tar-decompression-error")?;
                if n == 0 { break; }
                actual = actual.checked_add(n as u64).ok_or("member-size-overflow")?;
                if actual > declared { return Err("member-size-mismatch"); }
                if selected {
                    output.write_all(&buffer[..n]).map_err(|_| "staging-write-failed")?;
                    hash.update(&buffer[..n]);
                }
            }
            if actual != declared { return Err("truncated-member"); }
            if selected { selected_size = actual; }
        }
    }
    policy.finish()?;
    // Validate the XZ footer/checksum too: TAR EOF alone does not authenticate decompression.
    let mut reader = archive.into_inner();
    let mut padding = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut padding).map_err(|_| "xz-footer-or-expansion-error")?;
        if n == 0 { break; }
        if padding[..n].iter().any(|b| *b != 0) { return Err("trailing-tar-data"); }
    }
    if reader.inner.total_in() != bytes.len() as u64 { return Err("trailing-xz-data"); }
    Ok(Receipt { archive_sha256: digest, executable_sha256: hash.finalize().into(), byte_size: selected_size })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tar::{Builder, EntryType, Header};
    use xz2::write::XzEncoder;

    fn fixture(kind: EntryType, duplicate: bool) -> Vec<u8> {
        let mut tar = Builder::new(Vec::new());
        for _ in 0..if duplicate { 2 } else { 1 } {
            let payload = if kind.is_file() { &b"ffmpeg fixture"[..] } else { &b""[..] };
            let mut h = Header::new_gnu();
            h.set_path("package/bin/ffmpeg").unwrap();
            h.set_mode(0o755); h.set_size(payload.len() as u64); h.set_entry_type(kind);
            if kind.is_symlink() { h.set_link_name("outside").unwrap(); }
            h.set_cksum();
            tar.append(&h, payload).unwrap();
        }
        let bytes = tar.into_inner().unwrap();
        let mut xz = XzEncoder::new(Vec::new(), 1);
        xz.write_all(&bytes).unwrap(); xz.finish().unwrap()
    }

    #[test]
    fn extracts_valid_tar_xz_with_both_hashes() {
        let bytes = fixture(EntryType::Regular, false);
        let digest = Sha256::digest(&bytes).into();
        let mut out = Vec::new();
        let receipt = extract(&bytes, digest, "package.tar.xz", &mut out).unwrap();
        assert_eq!(out, b"ffmpeg fixture");
        assert_eq!(receipt.archive_sha256, digest);
        assert_eq!(receipt.byte_size, out.len() as u64);
        assert_eq!(receipt.executable_sha256, <[u8; 32]>::from(Sha256::digest(&out)));
    }

    #[test]
    fn rejects_links_duplicates_wrong_identity_and_truncated_xz() {
        for (kind, duplicate) in [(EntryType::Symlink, false), (EntryType::Regular, true)] {
            let bytes = fixture(kind, duplicate);
            assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.tar.xz", &mut Vec::new()).is_err());
        }
        let mut bytes = fixture(EntryType::Regular, false);
        let mut out = Vec::new();
        assert!(extract(&bytes, [0; 32], "package.tar.xz", &mut out).is_err());
        assert!(out.is_empty());
        bytes.truncate(bytes.len() - 6);
        assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.tar.xz", &mut out).is_err());
    }

    #[test]
    fn budget_distinguishes_exact_eof_from_one_extra_byte() {
        let mut exact = Budget { inner: Cursor::new(b"abc"), remaining: 3 };
        assert_eq!(io::copy(&mut exact, &mut io::sink()).unwrap(), 3);
        let mut over = Budget { inner: Cursor::new(b"abcd"), remaining: 3 };
        assert!(io::copy(&mut over, &mut io::sink()).is_err());
    }

    #[test]
    fn rejects_bytes_after_the_xz_stream() {
        let mut bytes = fixture(EntryType::Regular, false);
        bytes.extend_from_slice(b"extra");
        assert!(extract(&bytes, Sha256::digest(&bytes).into(), "package.tar.xz", &mut Vec::new()).is_err());
    }
}
