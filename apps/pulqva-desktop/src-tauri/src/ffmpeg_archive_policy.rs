//! Shared metadata gate for future ZIP/tar adapters. This does not authenticate or extract bytes.
//! Adapters must enforce the same bounds on actual decompressed bytes, not just header sizes.
#![allow(dead_code)] // T060-A: wired into tests; extraction adapters follow in T060-B/C.

const MAX_ENTRIES: u64 = 16_384;
const MAX_MEMBER: u64 = 512 * 1024 * 1024;
const MAX_TOTAL: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    File,
    Directory,
    Link,
    Special,
}

pub(super) struct Policy {
    root: String,
    executable: String,
    entries: u64,
    total: u64,
    selected: bool,
    failed: bool,
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.bytes().any(|c| c < 32 || c == 127 || matches!(c, b'\\' | b':' | b'<' | b'>' | b'"' | b'|' | b'?' | b'*'))
        && path.split('/').all(|c| {
            !c.is_empty() && c != "." && c != ".." && !c.ends_with(['.', ' '])
                && !matches!(c.split('.').next().unwrap_or("").to_ascii_uppercase().as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4"
                    | "COM5" | "COM6" | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2"
                    | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9")
        })
}

impl Policy {
    /// Only call with the backend platform spec's pinned archive asset name.
    pub(super) fn new(asset: &str) -> Result<Self, &'static str> {
        let (root, binary) = if let Some(root) = asset.strip_suffix(".tar.xz") {
            (root, "ffmpeg")
        } else if let Some(root) = asset.strip_suffix(".zip") {
            (root, "ffmpeg.exe")
        } else {
            return Err("unsupported-archive");
        };
        if !safe_path(root) || root.contains('/') {
            return Err("unsafe-archive-root");
        }
        Ok(Self {
            root: root.to_owned(), executable: format!("{root}/bin/{binary}"),
            entries: 0, total: 0, selected: false, failed: false,
        })
    }

    /// Returns true only for the single exact executable member. Any error poisons this scan.
    pub(super) fn observe(&mut self, path: &str, kind: Kind, size: u64) -> Result<bool, &'static str> {
        if self.failed { return Err("scan-already-rejected"); }
        self.failed = true;
        if !matches!(kind, Kind::File | Kind::Directory) { return Err("unsafe-member-type"); }
        let path = if kind == Kind::Directory { path.strip_suffix('/').unwrap_or(path) } else { path };
        if !safe_path(path) { return Err("unsafe-member-path"); }
        if path != self.root && !path.starts_with(&format!("{}/", self.root)) {
            return Err("unexpected-archive-root");
        }
        if path == self.root && kind != Kind::Directory { return Err("root-not-directory"); }
        if self.entries >= MAX_ENTRIES || size > MAX_MEMBER { return Err("archive-limit"); }
        let total = self.total.checked_add(size).filter(|n| *n <= MAX_TOTAL).ok_or("archive-limit")?;
        if kind == Kind::Directory && size != 0 { return Err("directory-has-data"); }
        let selected = path == self.executable;
        if selected && (kind != Kind::File || size == 0 || self.selected) {
            return Err("invalid-or-duplicate-executable");
        }
        self.entries += 1;
        self.total = total;
        self.selected |= selected;
        self.failed = false;
        Ok(selected)
    }

    pub(super) fn finish(self) -> Result<(), &'static str> {
        if self.failed { Err("scan-rejected") }
        else if !self.selected { Err("missing-executable") }
        else { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_formats_select_only_the_exact_executable() {
        for (asset, binary) in [("package.tar.xz", "ffmpeg"), ("package.zip", "ffmpeg.exe")] {
            let mut p = Policy::new(asset).unwrap();
            assert_eq!(p.observe("package/", Kind::Directory, 0), Ok(false));
            assert_eq!(p.observe("package/bin/ffprobe", Kind::File, 20), Ok(false));
            assert_eq!(p.observe(&format!("package/bin/{binary}"), Kind::File, 100), Ok(true));
            assert_eq!(p.finish(), Ok(()));
        }
    }

    #[test]
    fn unsafe_paths_poison_the_scan() {
        for path in ["/package/bin/ffmpeg", "../ffmpeg", "package/../ffmpeg", "package//bin/ffmpeg",
            "package/./bin/ffmpeg", "C:/ffmpeg", "package\\bin\\ffmpeg", "package/bin/ffmpeg:ads",
            "package/bin/ffmpeg.", "package/bin/ffmpeg ", "package/NUL.txt", "package/a\0b",
            "other/bin/ffmpeg", "package2/bin/ffmpeg", "package/bin/ffmpeg/"] {
            let mut p = Policy::new("package.tar.xz").unwrap();
            assert!(p.observe(path, Kind::File, 10).is_err(), "{path:?}");
            assert!(p.observe("package/bin/ffmpeg", Kind::File, 10).is_err());
            assert!(p.finish().is_err());
        }
    }

    #[test]
    fn rejects_links_special_members_and_duplicate_executables() {
        for kind in [Kind::Link, Kind::Special, Kind::Directory] {
            let mut p = Policy::new("package.zip").unwrap();
            assert!(p.observe("package/bin/ffmpeg.exe", kind, 0).is_err());
        }
        let mut p = Policy::new("package.zip").unwrap();
        p.observe("package/bin/ffmpeg.exe", Kind::File, 1).unwrap();
        assert!(p.observe("package/bin/ffmpeg.exe", Kind::File, 1).is_err());
        assert!(p.finish().is_err());
    }

    #[test]
    fn enforces_member_total_and_entry_limits() {
        let mut p = Policy::new("package.zip").unwrap();
        assert!(p.observe("package/huge", Kind::File, MAX_MEMBER + 1).is_err());
        let mut p = Policy::new("package.zip").unwrap();
        for _ in 0..4 { p.observe("package/data", Kind::File, MAX_MEMBER).unwrap(); }
        assert!(p.observe("package/data", Kind::File, 1).is_err());
        let mut p = Policy::new("package.zip").unwrap();
        for _ in 0..MAX_ENTRIES { p.observe("package/empty", Kind::File, 0).unwrap(); }
        assert!(p.observe("package/empty", Kind::File, 0).is_err());
    }

    #[test]
    fn requires_nonempty_executable_and_valid_archive_kind() {
        assert!(Policy::new("package.7z").is_err());
        assert!(Policy::new("../package.zip").is_err());
        assert!(Policy::new("package.zip").unwrap().finish().is_err());
        let mut p = Policy::new("package.zip").unwrap();
        assert!(p.observe("package/bin/ffmpeg.exe", Kind::File, 0).is_err());
    }
}
