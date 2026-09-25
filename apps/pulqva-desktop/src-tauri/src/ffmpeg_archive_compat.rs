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
    let resources = std::env::temp_dir().join(format!("pulqva-real-ffmpeg-source-{}", std::process::id()));
    std::fs::create_dir(&resources).unwrap();
    let resources = std::fs::canonicalize(resources).unwrap();
    let source = resources.join(platform.ffmpeg_resource);
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, &bytes).unwrap();
    let layout = super::AppRuntimeLayout::new(&stage_root).unwrap();
    let artifact = super::VerifiedBundledSidecarArtifact {
        kind: super::BundledSidecarKind::Ffmpeg, source,
        destination: layout.ffmpeg_executable().unwrap(),
        identity: super::BundledSidecarIdentity {
            version: super::FFMPEG_SIDECAR_VERSION.trim().to_owned(),
            pinned_source_sha256: Some(expected.clone()),
        },
        byte_size: bytes.len() as u64,
    };
    let prepared = super::PreparedAppRuntimeDirectories { layout };
    let mut invalid = artifact.clone(); invalid.kind = super::BundledSidecarKind::Arti;
    assert!(super::ffmpeg_source::extract_verified_ffmpeg(&prepared, &resources, &invalid).is_err());
    invalid = artifact.clone(); invalid.destination = stage_root.join("wrong");
    assert!(super::ffmpeg_source::extract_verified_ffmpeg(&prepared, &resources, &invalid).is_err());
    invalid = artifact.clone(); invalid.identity.version = "wrong".to_owned();
    assert!(super::ffmpeg_source::extract_verified_ffmpeg(&prepared, &resources, &invalid).is_err());
    assert_eq!(std::fs::read_dir(&stage_root).unwrap().count(), 0);
    let extraction = super::ffmpeg_source::extract_verified_ffmpeg(&prepared, &resources, &artifact).unwrap();
    assert_eq!(extraction.platform, platform.platform);
    assert_eq!(extraction.version, super::FFMPEG_SIDECAR_VERSION.trim());
    let stage = extraction.stage;
    assert_eq!(stage.receipt().executable_sha256, receipt.executable_sha256);
    assert_eq!(stage.receipt().byte_size, receipt.byte_size);
    let staged_path = stage.path().to_owned();
    drop(stage);
    assert!(!staged_path.exists());
    assert_eq!(std::fs::read_dir(&stage_root).unwrap().count(), 0);
    std::fs::remove_dir(stage_root).unwrap();
    std::fs::remove_dir_all(resources).unwrap();
    println!("PULQVA_FFMPEG_COMPAT platform={} sha256={} byte_size={}", platform.platform, actual, receipt.byte_size);
}
