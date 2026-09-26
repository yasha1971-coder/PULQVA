//! Authenticated immutable Deno ZIP -> bounded writer. No filesystem or process authority.
#![allow(dead_code)] // T065-A: owned staging and real-artifact verification follow separately.
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use zip::{CompressionMethod, ZipArchive};

#[derive(Clone, Copy, Debug)]
pub(super) enum DenoTarget { LinuxX64, WindowsX64 }

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Receipt {
    pub(super) version: &'static str,
    pub(super) target: &'static str,
    pub(super) archive_sha256: [u8; 32],
    pub(super) executable_sha256: [u8; 32],
    pub(super) byte_size: u64,
}

struct Identity {
    target: &'static str,
    name: &'static str,
    archive_size: usize,
    archive_hash: [u8; 32],
    executable_size: u64,
    executable_hash: [u8; 32],
}

fn digest(hex: &str) -> [u8; 32] {
    // Only fixed backend literals enter here.
    let mut result = [0; 32];
    for (i, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("pinned digest");
    }
    result
}

fn identity(target: DenoTarget) -> Identity {
    // CANDIDATE.json and ARTIFACT_VERIFICATION.json, Deno 2.9.7.
    match target {
        DenoTarget::LinuxX64 => Identity {
            target: "x86_64-unknown-linux-gnu", name: "deno", archive_size: 41_596_794,
            archive_hash: digest("c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490"),
            executable_size: 95_830_104,
            executable_hash: digest("ce6a052beb97c2b92de67077e3f3924ba7c9661ede0d8dc2f66a116a3c841f21"),
        },
        DenoTarget::WindowsX64 => Identity {
            target: "x86_64-pc-windows-msvc", name: "deno.exe", archive_size: 42_630_221,
            archive_hash: digest("a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238"),
            executable_size: 97_462_048,
            executable_hash: digest("e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33"),
        },
    }
}

pub(super) fn archive_identity(target: DenoTarget) -> (u64, [u8; 32]) {
    let pinned = identity(target);
    (pinned.archive_size as u64, pinned.archive_hash)
}

pub(super) fn executable_name(target: DenoTarget) -> &'static str {
    identity(target).name
}

/// Caller owns and must discard partial output on error. This receipt authenticates
/// bytes only: not a path, permissions, ownership, environment or launch safety.
pub(super) fn extract(bytes: &[u8], target: DenoTarget, output: &mut impl Write)
    -> Result<Receipt, &'static str>
{
    extract_expected(bytes, &identity(target), output)
}

fn extract_expected(bytes: &[u8], expected: &Identity, output: &mut impl Write)
    -> Result<Receipt, &'static str>
{
    if bytes.len() != expected.archive_size || bytes.len() > 64 * 1024 * 1024
        || <[u8; 32]>::from(Sha256::digest(bytes)) != expected.archive_hash {
        return Err("deno-archive-identity");
    }
    if expected.executable_size == 0 || expected.executable_size > 256 * 1024 * 1024 {
        return Err("deno-executable-limit");
    }
    // Bound count before ZipArchive allocates. No ZIP64, multipart, prefix or trailing data.
    if bytes.len() < 22 || bytes.get(..4) != Some(b"PK\x03\x04") { return Err("deno-zip-layout"); }
    let end = (bytes.len().saturating_sub(22 + 65_535)..=bytes.len() - 22).rev().find(|&p| {
        bytes[p..p + 4] == *b"PK\x05\x06"
            && p + 22 + u16::from_le_bytes([bytes[p + 20], bytes[p + 21]]) as usize == bytes.len()
    }).ok_or("deno-zip-end")?;
    let u16_at = |p| u16::from_le_bytes([bytes[p], bytes[p + 1]]) as usize;
    let u32_at = |p| u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
    if u16_at(end + 4) != 0 || u16_at(end + 6) != 0
        || u16_at(end + 8) != 1 || u16_at(end + 10) != 1
        || u32_at(end + 16).checked_add(u32_at(end + 12)) != Some(end) {
        return Err("deno-zip-layout");
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "deno-invalid-zip")?;
    if archive.len() != 1 { return Err("deno-member-count"); }
    let raw = archive.by_index_raw(0).map_err(|_| "deno-member")?;
    let mode = raw.unix_mode().unwrap_or(0) & 0o170000;
    if raw.name_raw() != expected.name.as_bytes() || raw.is_dir() || raw.encrypted()
        || !matches!(mode, 0 | 0o100000) || raw.size() != expected.executable_size
        || !matches!(raw.compression(), CompressionMethod::Stored | CompressionMethod::Deflated) {
        return Err("deno-member-policy");
    }
    drop(raw);
    let mut file = archive.by_index(0).map_err(|_| "deno-member")?;
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|_| "deno-decompression-or-crc")?;
        if n == 0 { break; }
        size = size.checked_add(n as u64).ok_or("deno-executable-limit")?;
        if size > expected.executable_size { return Err("deno-executable-limit"); }
        output.write_all(&buffer[..n]).map_err(|_| "deno-write")?;
        hash.update(&buffer[..n]);
    }
    let executable_hash: [u8; 32] = hash.finalize().into();
    if size != expected.executable_size || executable_hash != expected.executable_hash {
        return Err("deno-executable-identity");
    }
    Ok(Receipt { version: "2.9.7", target: expected.target,
        archive_sha256: expected.archive_hash, executable_sha256: executable_hash, byte_size: size })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn fixture(names: &[&str], method: CompressionMethod) -> (Vec<u8>, Identity) {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for name in names {
            zip.start_file(*name, SimpleFileOptions::default().compression_method(method)).unwrap();
            zip.write_all(b"verified deno fixture").unwrap();
        }
        let bytes = zip.finish().unwrap().into_inner();
        let expected = Identity { target: "fixture", name: "deno", archive_size: bytes.len(),
            archive_hash: Sha256::digest(&bytes).into(), executable_size: 21,
            executable_hash: Sha256::digest(b"verified deno fixture").into() };
        (bytes, expected)
    }

    #[test]
    fn exact_receipt_for_stored_and_deflated() {
        for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
            let (bytes, expected) = fixture(&["deno"], method);
            let mut output = Vec::new();
            let receipt = extract_expected(&bytes, &expected, &mut output).unwrap();
            assert_eq!(output, b"verified deno fixture");
            assert_eq!(receipt, Receipt { version: "2.9.7", target: "fixture",
                archive_sha256: expected.archive_hash, executable_sha256: expected.executable_hash, byte_size: 21 });
        }
    }

    #[test]
    fn rejects_names_and_extra_members_before_writing() {
        for names in [vec!["../deno"], vec!["/deno"], vec!["dir/deno"], vec!["deno.exe"],
            vec!["deno/"], vec!["deno", "extra"], vec!["C:\\deno"], vec!["deno:stream"]] {
            let (bytes, expected) = fixture(&names, CompressionMethod::Stored);
            let mut output = Vec::new();
            assert!(extract_expected(&bytes, &expected, &mut output).is_err());
            assert!(output.is_empty());
        }
    }

    #[test]
    fn rejects_both_identities_and_size_mismatch() {
        let (mut bytes, mut expected) = fixture(&["deno"], CompressionMethod::Stored);
        expected.executable_hash = [0; 32];
        assert_eq!(extract_expected(&bytes, &expected, &mut Vec::new()).unwrap_err(), "deno-executable-identity");
        expected.executable_size += 1;
        assert!(extract_expected(&bytes, &expected, &mut Vec::new()).is_err());
        bytes[0] ^= 1;
        let mut output = Vec::new();
        assert_eq!(extract_expected(&bytes, &expected, &mut output).unwrap_err(), "deno-archive-identity");
        assert!(output.is_empty());
        assert!(extract(&bytes, DenoTarget::LinuxX64, &mut output).is_err());
        assert!(extract(&bytes, DenoTarget::WindowsX64, &mut output).is_err());
    }

    #[test]
    fn malformed_layout_and_writer_failure() {
        let (mut bytes, mut expected) = fixture(&["deno"], CompressionMethod::Stored);
        assert_eq!(extract_expected(&bytes, &expected, &mut std::io::Cursor::new(&mut [0u8; 1][..])).unwrap_err(), "deno-write");
        bytes.truncate(bytes.len() - 4);
        expected.archive_size = bytes.len();
        expected.archive_hash = Sha256::digest(&bytes).into();
        assert!(extract_expected(&bytes, &expected, &mut Vec::new()).is_err());
    }

    #[test]
    fn rejects_crc_encryption_symlink_and_duplicate_count() {
        for case in 0..4 {
            let (mut bytes, mut expected) = fixture(&["deno"], CompressionMethod::Stored);
            let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
            match case {
                0 => { // Authenticate a damaged fixture to exercise the decompressor CRC gate.
                    let data = bytes.windows(21).position(|b| b == b"verified deno fixture").unwrap();
                    bytes[data] ^= 1;
                },
                1 => { bytes[6] |= 1; bytes[central + 8] |= 1; },
                2 => {
                    bytes[central + 5] = 3; // Unix creator.
                    bytes[central + 38..central + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
                },
                _ => { let end = bytes.len() - 22; bytes[end + 8] = 2; bytes[end + 10] = 2; },
            }
            expected.archive_hash = Sha256::digest(&bytes).into();
            assert!(extract_expected(&bytes, &expected, &mut Vec::new()).is_err());
        }
    }
}
