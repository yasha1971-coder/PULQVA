//! CI-only real archive proof; does not execute the materialized binary.
use super::{deno_source::materialize_deno, deno_zip::DenoTarget};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::PathBuf};

#[test]
#[ignore = "requires authenticated official archive and independent reference from CI"]
fn pinned_archive_matches_independent_extractor_and_owned_stage() {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => DenoTarget::LinuxX64,
        ("windows", "x86_64") => DenoTarget::WindowsX64,
        _ => panic!("unsupported proof target"),
    };
    let source = fs::canonicalize(PathBuf::from(std::env::var_os("PULQVA_DENO_FIXTURE").expect("fixture"))).unwrap();
    let resources = source.parent().unwrap();
    let root = resources.join("rust-owned-stage");
    fs::create_dir(&root).unwrap();
    let root = fs::canonicalize(root).unwrap();
    let keep = root.join("keep");
    fs::write(&keep, b"unrelated sentinel").unwrap();
    assert!(materialize_deno(resources, &resources.join("missing.zip"), &root, target).is_err());
    assert!(materialize_deno(resources, resources, &root, target).is_err());
    // A correct archive for the other target must fail before creating output.
    let other = match target { DenoTarget::LinuxX64 => DenoTarget::WindowsX64, DenoTarget::WindowsX64 => DenoTarget::LinuxX64 };
    assert!(materialize_deno(resources, &source, &root, other).is_err());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);

    let stage = materialize_deno(resources, &source, &root, target).expect("real pinned source/ZIP/stage");
    let receipt = stage.receipt();
    assert_eq!(receipt.version, "2.9.7");
    let (archive_size, archive_hash) = super::deno_zip::archive_identity(target);
    assert_eq!(fs::metadata(&source).unwrap().len(), archive_size);
    assert_eq!(receipt.archive_sha256, archive_hash);
    assert_eq!(stage.path().file_name().unwrap(), super::deno_zip::executable_name(target));
    assert_eq!(receipt.byte_size, std::env::var("PULQVA_DENO_REFERENCE_SIZE").unwrap().parse::<u64>().unwrap());
    let mut file = fs::File::open(stage.path()).unwrap();
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 { break; }
        size += n as u64;
        assert!(size <= receipt.byte_size);
        hash.update(&buffer[..n]);
    }
    drop(file);
    let digest: [u8; 32] = hash.finalize().into();
    assert_eq!(size, receipt.byte_size);
    assert_eq!(digest, receipt.executable_sha256);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, std::env::var("PULQVA_DENO_REFERENCE_SHA256").unwrap());
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(stage.path()).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(stage.path().parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
    }
    let path = stage.path().to_owned();
    let directory = path.parent().unwrap().to_owned();
    let platform = receipt.target;
    drop(stage);
    assert!(!path.exists() && !directory.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    assert_eq!(fs::read(&keep).unwrap(), b"unrelated sentinel");
    fs::remove_file(keep).unwrap();
    fs::remove_dir(root).unwrap();
    println!("PULQVA_DENO_COMPAT target={platform} sha256={hex} byte_size={size} cleanup=ok");
}
