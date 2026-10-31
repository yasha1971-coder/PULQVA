//! Windows official Tor resources: authenticate before owned runtime launch.
use super::*;
use std::io::Write;

const MEMBERS: [(&str, &str, u64); 3] = [
    ("tor.exe", "1607d98f5d13859077b1b349c55beba76b09acc788f0431fd824d11966fb1074", 10247168),
    ("geoip", "5f0ebc505ad5a5b219f9c053ce6228c837669364349a1a70c236192dd99a1c3d", 9855249),
    ("geoip6", "d6bc775efb8f970afacbb702ea0c80dbf9e2add32232afcfe3f376a972503b56", 16075127),
];
// Expert Bundle 16.0 / Tor 0.4.9.15, signed archive SHA256
// 2ee123e9167318a3742778ff704e75f01d4780f05a14522d451fe011011077c4.
// Qualification and signature evidence are retained separately from runtime data.
fn invalid() -> DownloadActionError {
    DownloadActionError { code: "official-tor-resource-invalid",
        message: "Official Tor resources are missing, linked, or have an invalid identity.".into() }
}
fn plain_path(path: &Path, file: bool) -> Result<(), DownloadActionError> {
    use std::os::windows::fs::MetadataExt;
    if !path.is_absolute() { return Err(invalid()); }
    for ancestor in path.ancestors() {
        let meta = fs::symlink_metadata(ancestor).map_err(|_| invalid())?;
        if meta.file_attributes() & 0x400 != 0 { return Err(invalid()); }
        if ancestor == path {
            if (file && !meta.is_file()) || (!file && !meta.is_dir()) { return Err(invalid()); }
        } else if !meta.is_dir() { return Err(invalid()); }
    }
    Ok(())
}
fn verify(path: &Path, hash: &str, size: u64) -> Result<(), DownloadActionError> {
    plain_path(path, true)?;
    if fs::metadata(path).map_err(|_| invalid())?.len() != size || sha256_file(path)? != hash {
        return Err(invalid());
    }
    Ok(())
}
fn copy_new(source: &Path, destination: &Path, hash: &str, size: u64) -> Result<(), DownloadActionError> {
    verify(source, hash, size)?;
    plain_path(destination.parent().ok_or_else(invalid)?, false)?;
    let mut input = fs::File::open(source).map_err(|_| invalid())?;
    let mut output = fs::OpenOptions::new().create_new(true).write(true).open(destination).map_err(|_| invalid())?;
    std::io::copy(&mut input, &mut output).map_err(|_| invalid())?;
    output.flush().map_err(|_| invalid())?;
    output.sync_all().map_err(|_| invalid())?;
    drop(output);
    verify(destination, hash, size)
}

fn ascii_unavailable() -> DownloadActionError {
    DownloadActionError { code: "official-tor-ascii-runtime-unavailable",
        message: "Windows could not provide a safe ASCII path for the private Tor runtime.".into() }
}
fn checked_ascii_alias(original: &Path, alias: PathBuf) -> Result<PathBuf, DownloadActionError> {
    use std::path::{Component, Prefix};
    if !alias.is_absolute() || !alias.to_str().is_some_and(str::is_ascii)
        || !matches!(alias.components().next(), Some(Component::Prefix(p))
            if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))) {
        return Err(ascii_unavailable());
    }
    plain_path(original, false)?;
    plain_path(&alias, false)?;
    // An 8.3 name must refer to this exact owned directory, never a replacement
    // temp directory. Same filesystem object means the same owner and ACL.
    if fs::canonicalize(original).map_err(|_| ascii_unavailable())?
        != fs::canonicalize(&alias).map_err(|_| ascii_unavailable())? {
        return Err(ascii_unavailable());
    }
    Ok(alias)
}
fn ascii_runtime_dir(runtime: &Path) -> Result<PathBuf, DownloadActionError> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    plain_path(runtime, false)?;
    if runtime.to_str().is_some_and(str::is_ascii) {
        return checked_ascii_alias(runtime, runtime.to_path_buf());
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetShortPathNameW(long: *const u16, short: *mut u16, size: u32) -> u32;
    }
    let input: Vec<u16> = runtime.as_os_str().encode_wide().chain(Some(0)).collect();
    let required = unsafe { GetShortPathNameW(input.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 || required > 32768 { return Err(ascii_unavailable()); }
    let mut output = vec![0u16; required as usize];
    let written = unsafe { GetShortPathNameW(input.as_ptr(), output.as_mut_ptr(), required) };
    if written == 0 || written >= required { return Err(ascii_unavailable()); }
    let alias = PathBuf::from(std::ffi::OsString::from_wide(&output[..written as usize]));
    // Disabled/unavailable 8.3 names are an explicit failure, never permission
    // to use shared TEMP, weaken ACLs, change Windows settings or use clearnet.
    checked_ascii_alias(runtime, alias)
}

pub(super) fn launch(resources: &Path, root: &Path) -> Result<RunningArti, DownloadActionError> {
    plain_path(resources, false)?;
    plain_path(root, false)?;
    let source = resources.join("sidecars/tor/windows-x86_64");
    // Authenticate the complete set before preparing config or spawning a process.
    for (name, hash, size) in MEMBERS { verify(&source.join(name), hash, size)?; }
    let runtime = root.join("official-tor");
    fs::create_dir(&runtime).map_err(|_| invalid())?;
    let runtime = ascii_runtime_dir(&runtime)?;
    for (name, hash, size) in MEMBERS { copy_new(&source.join(name), &runtime.join(name), hash, size)?; }
    let endpoint = TorSocksEndpoint::new(DEFAULT_TOR_SOCKS_PORT).map_err(DownloadActionError::from)?;
    let prepared = pulqva_privacy::prepare_official_tor_runtime(
        &runtime.join("tor.exe"), &runtime.join("torrc"), &runtime.join("state"),
        &runtime.join("geoip"), &runtime.join("geoip6"), endpoint
    ).map_err(|_| invalid())?;
    pulqva_privacy::launch_prepared_official_tor(prepared).map_err(DownloadActionError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!("pulqva-tor-host-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap(); root
    }
    #[test]
    fn rejects_changed_bytes_and_wrong_size_before_copy() {
        let root = fixture(); let source = root.join("source"); let dest = root.join("dest");
        fs::write(&source, b"abc").unwrap();
        let hash = sha256_file(&source).unwrap();
        assert!(copy_new(&source, &dest, &hash, 4).is_err()); assert!(!dest.exists());
        fs::write(&source, b"abd").unwrap();
        assert!(copy_new(&source, &dest, &hash, 3).is_err()); assert!(!dest.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn materialization_verifies_readback_and_never_overwrites() {
        let root = fixture(); let source = root.join("source"); let dest = root.join("dest");
        fs::write(&source, b"abc").unwrap(); let hash = sha256_file(&source).unwrap();
        copy_new(&source, &dest, &hash, 3).unwrap();
        assert_eq!(sha256_file(&dest).unwrap(), hash);
        assert!(copy_new(&source, &dest, &hash, 3).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"abc"); fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn incomplete_package_cannot_spawn_or_prepare_runtime() {
        let root = fixture();
        assert!(launch(&root, &root).is_err()); assert!(!root.join("official-tor").exists());
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod ascii_runtime_tests {
    use super::*;
    fn nonce() -> String {
        format!("{}-{}", std::process::id(), std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }
    #[test]
    fn cyrillic_profile_runtime_uses_same_private_directory_and_cleans_up() {
        let parent = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("Windows user profile"))
            .join("Temp").join(format!("pulqva-ascii-regression-{}", nonce()));
        let runtime = parent.join("Profile Контур with spaces").join("official-tor");
        fs::create_dir_all(&runtime).unwrap();
        let alias = ascii_runtime_dir(&runtime).expect("ASCII alias required; do not disable this gate if volume lacks aliases");
        assert!(alias.is_absolute() && alias.to_str().unwrap().is_ascii());
        assert_eq!(fs::canonicalize(&alias).unwrap(), fs::canonicalize(&runtime).unwrap());
        for name in ["tor.exe", "geoip", "geoip6"] { fs::write(alias.join(name), b"fixture, never executed").unwrap(); }
        let prepared = pulqva_privacy::prepare_official_tor_runtime(
            &alias.join("tor.exe"), &alias.join("torrc"), &alias.join("state"),
            &alias.join("geoip"), &alias.join("geoip6"),
            TorSocksEndpoint::new(19050).unwrap()).unwrap();
        let config = fs::read_to_string(alias.join("torrc")).unwrap();
        assert!(config.is_ascii() && !config.contains("Контур"));
        assert!(config.contains("SocksPort 127.0.0.1:19050"));
        fs::write(alias.join("state").join("control_auth_cookie"), b"fixture only").unwrap();
        assert!(runtime.join("state").join("control_auth_cookie").is_file());
        drop(prepared);
        fs::remove_dir_all(&parent).unwrap();
        assert!(!alias.exists());
    }
    #[test]
    fn rejects_non_ascii_or_different_directory_instead_of_falling_back() {
        let root = std::env::temp_dir().join(format!("pulqva-ascii-negative-{}", nonce()));
        fs::create_dir(&root).unwrap();
        let different = root.join("different"); fs::create_dir(&different).unwrap();
        assert!(checked_ascii_alias(&root, different).is_err());
        assert!(checked_ascii_alias(&root, PathBuf::from("relative")).is_err());
        let unicode = root.join("Контур"); fs::create_dir(&unicode).unwrap();
        assert_eq!(checked_ascii_alias(&unicode, unicode.clone()).unwrap_err().code,
            "official-tor-ascii-runtime-unavailable");
        fs::remove_dir_all(root).unwrap();
    }
}
