//! Opt-in CI proof using actual pinned source archives and independent Python output hashes.
use super::{FFMPEG_SIDECAR_SHA256SUMS, current_sidecar_platform_spec, pinned_sha256_for_asset};

#[test]
#[ignore = "requires pinned real archive prepared by CI"]
fn pinned_archive_matches_independent_extractor() {
    let platform = current_sidecar_platform_spec().unwrap();
    let expected = pinned_sha256_for_asset(FFMPEG_SIDECAR_SHA256SUMS, platform.ffmpeg_digest_asset).unwrap();
    let mut digest = [0u8; 32];
    for (i, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&expected[i * 2..i * 2 + 2], 16).unwrap();
    }
    let path = std::env::var_os("PULQVA_FFMPEG_FIXTURE").expect("CI fixture path is required");
    let bytes = std::fs::read(path).unwrap();
    let mut output = std::io::sink();
    let receipt = if platform.ffmpeg_digest_asset.ends_with(".zip") {
        super::ffmpeg_zip::extract(&bytes, digest, platform.ffmpeg_digest_asset, &mut output)
    } else {
        super::ffmpeg_tar_xz::extract(&bytes, digest, platform.ffmpeg_digest_asset, &mut output)
    }.expect("real pinned archive must satisfy the extraction contract");
    let actual: String = receipt.executable_sha256.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(receipt.archive_sha256, digest);
    assert_eq!(actual, std::env::var("PULQVA_FFMPEG_REFERENCE_SHA256").unwrap());
    assert_eq!(receipt.byte_size, std::env::var("PULQVA_FFMPEG_REFERENCE_SIZE").unwrap().parse::<u64>().unwrap());
    let stage_root = std::env::temp_dir().join(format!("pulqva-real-ffmpeg-stage-{}", std::process::id()));
    std::fs::create_dir(&stage_root).unwrap();
    let stage_root = std::fs::canonicalize(stage_root).unwrap();
    let stage = super::ffmpeg_stage::extract_to_stage(&stage_root, &bytes, digest, platform.ffmpeg_digest_asset).unwrap();
    assert_eq!(stage.receipt().executable_sha256, receipt.executable_sha256);
    assert_eq!(stage.receipt().byte_size, receipt.byte_size);
    let staged_path = stage.path().to_owned();
    drop(stage);
    assert!(!staged_path.exists());
    assert_eq!(std::fs::read_dir(&stage_root).unwrap().count(), 0);
    std::fs::remove_dir(stage_root).unwrap();
    println!("PULQVA_FFMPEG_COMPAT platform={} sha256={} byte_size={}", platform.platform, actual, receipt.byte_size);
}
