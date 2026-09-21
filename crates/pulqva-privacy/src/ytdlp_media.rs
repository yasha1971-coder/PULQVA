use std::{
    error::Error,
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
};

use crate::YtDlpLaunchPlan;

/// Validated network source accepted by the yt-dlp request planner.
///
/// Only HTTP(S) URLs are accepted. Local files, stdin markers, custom schemes,
/// and other non-network locators are rejected before any process can exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YtDlpMediaSourceUrl {
    value: String,
}

impl YtDlpMediaSourceUrl {
    pub fn parse(value: impl Into<String>) -> Result<Self, YtDlpMediaSourceError> {
        let value = value.into();

        if value.is_empty() {
            return Err(YtDlpMediaSourceError::Empty);
        }

        if value.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
            return Err(YtDlpMediaSourceError::WhitespaceOrControl);
        }

        let (scheme, remainder) = value
            .split_once("://")
            .ok_or(YtDlpMediaSourceError::MissingSchemeSeparator)?;

        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(YtDlpMediaSourceError::UnsupportedScheme);
        }

        let authority_end = remainder
            .find(|ch: char| matches!(ch, '/' | '?' | '#'))
            .unwrap_or(remainder.len());

        if remainder[..authority_end].is_empty() {
            return Err(YtDlpMediaSourceError::MissingAuthority);
        }

        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YtDlpMediaSourceError {
    Empty,
    MissingSchemeSeparator,
    UnsupportedScheme,
    MissingAuthority,
    WhitespaceOrControl,
}

impl fmt::Display for YtDlpMediaSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("yt-dlp media source URL must not be empty"),
            Self::MissingSchemeSeparator => {
                f.write_str("yt-dlp media source URL must include an explicit scheme")
            }
            Self::UnsupportedScheme => {
                f.write_str("yt-dlp media source URL scheme must be http or https")
            }
            Self::MissingAuthority => {
                f.write_str("yt-dlp media source URL must include a non-empty authority")
            }
            Self::WhitespaceOrControl => {
                f.write_str("yt-dlp media source URL must not contain whitespace or control characters")
            }
        }
    }
}

impl Error for YtDlpMediaSourceError {}

/// Pure-data media request layered on top of a Tor-gated yt-dlp launch plan.
///
/// This type does not spawn yt-dlp or perform network/filesystem I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YtDlpMediaRequestPlan {
    launch: YtDlpLaunchPlan,
    source: YtDlpMediaSourceUrl,
    output_root: PathBuf,
    metadata_only: bool,
}

impl YtDlpMediaRequestPlan {
    pub fn new(
        launch: YtDlpLaunchPlan,
        source: YtDlpMediaSourceUrl,
        output_root: impl Into<PathBuf>,
    ) -> Result<Self, YtDlpMediaRequestError> {
        let output_root = output_root.into();

        if output_root.as_os_str().is_empty() {
            return Err(YtDlpMediaRequestError::MissingOutputRoot);
        }

        Ok(Self {
            launch,
            source,
            output_root,
            metadata_only: false,
        })
    }

    pub fn new_metadata_only(
        launch: YtDlpLaunchPlan,
        source: YtDlpMediaSourceUrl,
        output_root: impl Into<PathBuf>,
    ) -> Result<Self, YtDlpMediaRequestError> {
        let output_root = output_root.into();

        if output_root.as_os_str().is_empty() {
            return Err(YtDlpMediaRequestError::MissingOutputRoot);
        }

        Ok(Self {
            launch,
            source,
            output_root,
            metadata_only: true,
        })
    }

    pub fn is_metadata_only(&self) -> bool {
        self.metadata_only
    }

    pub fn executable(&self) -> &Path {
        self.launch.executable()
    }

    pub fn source(&self) -> &YtDlpMediaSourceUrl {
        &self.source
    }

    pub fn output_root(&self) -> &Path {
        &self.output_root
    }

    /// Deterministic argv for the future yt-dlp process.
    ///
    /// The Tor-only base arguments are copied first without modification, then
    /// the explicit output root and validated HTTP(S) source are appended.
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments = self.launch.arguments();

        if self.metadata_only {
            arguments.push(OsString::from("--skip-download"));
            arguments.push(OsString::from("--dump-single-json"));
            arguments.push(OsString::from("--no-playlist"));
        }

        arguments.push(OsString::from("--paths"));
        arguments.push(self.output_root.as_os_str().to_os_string());
        arguments.push(OsString::from(self.source.as_str()));
        arguments
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YtDlpMediaRequestError {
    MissingOutputRoot,
}

impl fmt::Display for YtDlpMediaRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOutputRoot => f.write_str("yt-dlp output root must not be empty"),
        }
    }
}

impl Error for YtDlpMediaRequestError {}

#[cfg(test)]
mod tests {
    use super::{
        YtDlpMediaRequestError, YtDlpMediaRequestPlan, YtDlpMediaSourceError,
        YtDlpMediaSourceUrl,
    };
    use crate::{
        TorSocksEndpoint, YtDlpLaunchPlan,
        arti_ready::certify_tor_ready,
    };
    use std::{
        ffi::OsString,
        net::{IpAddr, Ipv4Addr},
        path::Path,
    };

    fn launch_plan() -> YtDlpLaunchPlan {
        let ready = certify_tor_ready(
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
        );
        YtDlpLaunchPlan::new("runtime/yt-dlp", ready)
    }

    #[test]
    fn accepts_explicit_http_and_https_network_sources() {
        assert_eq!(
            YtDlpMediaSourceUrl::parse("https://example.com/watch?v=1")
                .expect("https source is supported")
                .as_str(),
            "https://example.com/watch?v=1"
        );
        assert!(
            YtDlpMediaSourceUrl::parse("http://example.com/media")
                .is_ok()
        );
    }

    #[test]
    fn rejects_non_network_or_unsupported_source_forms() {
        assert_eq!(
            YtDlpMediaSourceUrl::parse("").unwrap_err(),
            YtDlpMediaSourceError::Empty
        );
        assert_eq!(
            YtDlpMediaSourceUrl::parse("file:///tmp/video.mp4").unwrap_err(),
            YtDlpMediaSourceError::UnsupportedScheme
        );
        assert_eq!(
            YtDlpMediaSourceUrl::parse("ftp://example.com/video").unwrap_err(),
            YtDlpMediaSourceError::UnsupportedScheme
        );
        assert_eq!(
            YtDlpMediaSourceUrl::parse("example.com/video").unwrap_err(),
            YtDlpMediaSourceError::MissingSchemeSeparator
        );
        assert_eq!(
            YtDlpMediaSourceUrl::parse("https:///video").unwrap_err(),
            YtDlpMediaSourceError::MissingAuthority
        );
        assert_eq!(
            YtDlpMediaSourceUrl::parse("https://example.com/a b").unwrap_err(),
            YtDlpMediaSourceError::WhitespaceOrControl
        );
    }

    #[test]
    fn requires_explicit_output_root() {
        let source =
            YtDlpMediaSourceUrl::parse("https://example.com/video").expect("valid source");

        assert_eq!(
            YtDlpMediaRequestPlan::new(launch_plan(), source, "")
                .unwrap_err(),
            YtDlpMediaRequestError::MissingOutputRoot
        );
    }

    #[test]
    fn metadata_only_request_is_explicit_and_never_requests_media_payload() {
        let launch = launch_plan();
        let source =
            YtDlpMediaSourceUrl::parse("https://example.com/video").expect("valid source");
        let request = YtDlpMediaRequestPlan::new_metadata_only(launch, source, "downloads")
            .expect("metadata request is valid");

        assert!(request.is_metadata_only());
        assert_eq!(
            request.arguments(),
            vec![
                OsString::from("--ignore-config"),
                OsString::from("--proxy"),
                OsString::from("socks5h://127.0.0.1:19050"),
                OsString::from("--skip-download"),
                OsString::from("--dump-single-json"),
                OsString::from("--no-playlist"),
                OsString::from("--paths"),
                OsString::from("downloads"),
                OsString::from("https://example.com/video"),
            ]
        );
    }

    #[test]
    fn preserves_tor_base_arguments_and_appends_request_deterministically() {
        let launch = launch_plan();
        let base = launch.arguments();
        let source =
            YtDlpMediaSourceUrl::parse("https://example.com/video?id=42").expect("valid source");
        let request =
            YtDlpMediaRequestPlan::new(launch, source, "downloads").expect("output root is set");

        let arguments = request.arguments();

        assert_eq!(&arguments[..base.len()], base.as_slice());
        assert_eq!(
            arguments,
            vec![
                OsString::from("--ignore-config"),
                OsString::from("--proxy"),
                OsString::from("socks5h://127.0.0.1:19050"),
                OsString::from("--paths"),
                OsString::from("downloads"),
                OsString::from("https://example.com/video?id=42"),
            ]
        );
        assert_eq!(request.executable(), Path::new("runtime/yt-dlp"));
        assert_eq!(request.output_root(), Path::new("downloads"));
    }
}
