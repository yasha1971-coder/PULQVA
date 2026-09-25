//! Typed runtime arguments for the pinned yt-dlp candidate (ADR-0006).
//!
//! This module performs no filesystem, process or network activity. It is not
//! yet wired into YtDlpLaunchPlan; T064-B must do that and test the full argv.

use std::{
    error::Error,
    ffi::{OsStr, OsString},
    fmt,
    path::{Component, Path, PathBuf},
};

/// A syntactically explicit native path to the intended bundled Deno executable.
///
/// This is NOT proof that a file exists, is executable, is owned by PULQVA, or
/// matches the candidate hash. Before launch, the materialization boundary must
/// check the actual file, its identity/ownership and replacement races. It must
/// also enforce the owned environment/cache policy. A valid path alone grants
/// neither network access nor a Tor readiness capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledDenoPath(PathBuf);

impl BundledDenoPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, BundledDenoPathError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(BundledDenoPathError::Missing);
        }
        let text = path.to_str().ok_or(BundledDenoPathError::NonUnicode)?;
        if text.chars().any(char::is_control) {
            return Err(BundledDenoPathError::InvalidComponent);
        }
        if !path.is_absolute() {
            return Err(BundledDenoPathError::NotAbsolute);
        }
        // Do not silently accept network/device namespaces as bundled paths.
        #[cfg(windows)]
        if !matches!(
            path.components().next(),
            Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), std::path::Prefix::Disk(_))
        ) {
            return Err(BundledDenoPathError::NotLocal);
        }
        #[cfg(not(windows))]
        if text.starts_with("//") {
            return Err(BundledDenoPathError::NotLocal);
        }
        // Path::components normalizes interior dots, so inspect the spelling too.
        if text.split(|c| c == '/' || (cfg!(windows) && c == '\\'))
            .any(|part| part == "." || part == "..")
            || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
            || text.ends_with('/') || (cfg!(windows) && text.ends_with('\\'))
        {
            return Err(BundledDenoPathError::InvalidComponent);
        }
        #[cfg(windows)]
        for part in path.components() {
            if let Component::Normal(value) = part {
                let value = value.to_str().ok_or(BundledDenoPathError::NonUnicode)?;
                if value.chars().any(|c| "<>:\"|?*".contains(c))
                    || value.ends_with('.') || value.ends_with(' ')
                {
                    return Err(BundledDenoPathError::InvalidComponent);
                }
            }
        }
        let leaf = if cfg!(windows) { "deno.exe" } else { "deno" };
        if path.file_name() != Some(OsStr::new(leaf)) {
            return Err(BundledDenoPathError::WrongExecutableName);
        }
        Ok(Self(path))
    }

    pub fn executable(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundledDenoPathError {
    Missing,
    NonUnicode,
    NotAbsolute,
    NotLocal,
    InvalidComponent,
    WrongExecutableName,
}

impl fmt::Display for BundledDenoPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Missing => "bundled Deno path is missing",
            Self::NonUnicode => "bundled Deno path must be Unicode",
            Self::NotAbsolute => "bundled Deno path must be absolute",
            Self::NotLocal => "bundled Deno path must not use a network/device namespace",
            Self::InvalidComponent => "bundled Deno path contains an invalid component",
            Self::WrongExecutableName => "bundled Deno path must name the platform Deno executable",
        })
    }
}

impl Error for BundledDenoPathError {}

/// No implicit host-runtime or caller-supplied argument variant exists.
///
/// Disabled is deliberate, not a request to use yt-dlp's default Deno discovery.
/// The caller must integrate this fragment with ignore-config, a verified Tor
/// route, plugin suppression, and typed request arguments before launching.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum YtDlpJsRuntime {
    #[default]
    Disabled,
    BundledDeno(BundledDenoPath),
}

impl YtDlpJsRuntime {
    /// Produces only a runtime/component-policy fragment, not a full launch plan.
    /// No shell quoting or splitting is performed; the path is one OS argument.
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments = vec![OsString::from("--no-js-runtimes")];
        if let Self::BundledDeno(runtime) = self {
            let mut selection = OsString::from("deno:");
            selection.push(runtime.executable().as_os_str());
            arguments.push(OsString::from("--js-runtimes"));
            arguments.push(selection);
        }
        arguments.push(OsString::from("--no-remote-components"));
        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(if cfg!(windows) { r"C:\PULQVA bundle" } else { "/opt/PULQVA bundle" })
    }

    fn leaf() -> &'static str {
        if cfg!(windows) { "deno.exe" } else { "deno" }
    }

    #[test]
    fn disabled_clears_default_discovery_and_remote_components() {
        assert_eq!(YtDlpJsRuntime::default().arguments(), vec![
            OsString::from("--no-js-runtimes"),
            OsString::from("--no-remote-components"),
        ]);
    }

    #[test]
    fn explicit_deno_has_exact_order_and_one_unquoted_path_argument() {
        let path = root().join("space & $literal").join(leaf());
        let runtime = BundledDenoPath::new(path.clone()).unwrap();
        assert_eq!(runtime.executable(), path.as_path());
        let mut selection = OsString::from("deno:");
        selection.push(path.as_os_str());
        assert_eq!(YtDlpJsRuntime::BundledDeno(runtime).arguments(), vec![
            OsString::from("--no-js-runtimes"),
            OsString::from("--js-runtimes"),
            selection,
            OsString::from("--no-remote-components"),
        ]);
    }

    #[test]
    fn rejects_missing_and_relative_paths() {
        assert_eq!(BundledDenoPath::new("").unwrap_err(), BundledDenoPathError::Missing);
        for path in [PathBuf::from(leaf()), PathBuf::from("runtime").join(leaf())] {
            assert_eq!(BundledDenoPath::new(path).unwrap_err(), BundledDenoPathError::NotAbsolute);
        }
    }

    #[test]
    fn rejects_parent_and_dot_segments_without_normalizing_them() {
        for part in [".", ".."] {
            assert_eq!(BundledDenoPath::new(root().join(part).join(leaf())).unwrap_err(),
                BundledDenoPathError::InvalidComponent);
        }
    }

    #[test]
    fn rejects_controls_and_nul() {
        for part in ["line\nbreak", "tab\tname", "nul\0byte"] {
            assert_eq!(BundledDenoPath::new(root().join(part).join(leaf())).unwrap_err(),
                BundledDenoPathError::InvalidComponent);
        }
    }

    #[test]
    fn rejects_directory_only_wrong_runtime_and_trailing_separator() {
        for path in [root(), root().join("node"), root().join("denort")] {
            assert_eq!(BundledDenoPath::new(path).unwrap_err(), BundledDenoPathError::WrongExecutableName);
        }
        let mut directory = root().join(leaf()).into_os_string();
        directory.push(std::path::MAIN_SEPARATOR.to_string());
        assert_eq!(BundledDenoPath::new(PathBuf::from(directory)).unwrap_err(),
            BundledDenoPathError::InvalidComponent);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_unicode_without_lossy_conversion() {
        use std::os::unix::ffi::OsStringExt;
        let path = PathBuf::from(OsString::from_vec(b"/opt/\xff/deno".to_vec()));
        assert_eq!(BundledDenoPath::new(path).unwrap_err(), BundledDenoPathError::NonUnicode);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_double_slash_namespace() {
        assert_eq!(BundledDenoPath::new("//host/share/deno").unwrap_err(), BundledDenoPathError::NotLocal);
    }

    #[cfg(windows)]
    #[test]
    fn rejects_network_device_and_drive_relative_paths() {
        for path in [r"\\server\share\deno.exe", r"\\?\C:\bundle\deno.exe", r"\\.\C:\bundle\deno.exe"] {
            assert!(BundledDenoPath::new(path).is_err());
        }
        for path in [r"C:bundle\deno.exe", r"\bundle\deno.exe"] {
            assert_eq!(BundledDenoPath::new(path).unwrap_err(), BundledDenoPathError::NotAbsolute);
        }
    }

    #[cfg(windows)]
    #[test]
    fn rejects_stream_wildcard_and_ambiguous_windows_components() {
        for path in [r"C:\bundle:stream\deno.exe", r"C:\bad*\deno.exe", r"C:\bundle.\deno.exe", r"C:\bundle \deno.exe"] {
            assert_eq!(BundledDenoPath::new(path).unwrap_err(), BundledDenoPathError::InvalidComponent);
        }
    }
}
