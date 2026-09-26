use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{ReadyTorTransport, YtDlpJsRuntime};

/// Pure-data launch plan for the pinned yt-dlp sidecar.
///
/// Construction requires a verified Tor transport capability. There is no
/// direct/clearnet route variant and no shell command string is produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YtDlpLaunchPlan {
    executable: PathBuf,
    proxy_url: String,
    runtime: YtDlpJsRuntime,
}

impl YtDlpLaunchPlan {
    /// Creates a yt-dlp launch plan whose network route is derived exclusively
    /// from a verified Tor transport capability.
    pub fn new(
        executable: impl Into<PathBuf>,
        transport: ReadyTorTransport,
    ) -> Self {
        Self {
            executable: executable.into(),
            proxy_url: transport.proxy_url(),
            runtime: YtDlpJsRuntime::Disabled,
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Selects typed runtime policy; the path is not a verified binary.
    /// Materialization/hash/ownership and environment checks remain prelaunch gates.
    pub fn with_js_runtime(mut self, runtime: YtDlpJsRuntime) -> Self {
        self.runtime = runtime;
        self
    }

    /// Deterministic base arguments for all future yt-dlp requests.
    ///
    /// --ignore-config prevents ambient user/system yt-dlp configuration from
    /// overriding PULQVA's routing contract. The plan deliberately contains no
    /// media URL and performs no process or network activity.
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments = vec![
            OsString::from("--ignore-config"),
            OsString::from("--proxy"),
            OsString::from(&self.proxy_url),
            OsString::from("--no-plugin-dirs"),
        ];
        arguments.extend(self.runtime.arguments());
        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::YtDlpLaunchPlan;
    use crate::{
        TorSocksEndpoint,
        arti_ready::certify_tor_ready,
    };
    use std::{
        ffi::OsString,
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
        path::Path,
    };

    fn ready_v4() -> crate::ReadyTorTransport {
        certify_tor_ready(
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
        )
    }

    #[test]
    fn keeps_executable_explicit() {
        let plan = YtDlpLaunchPlan::new("runtime/yt-dlp", ready_v4());

        assert_eq!(plan.executable(), Path::new("runtime/yt-dlp"));
    }

    #[test]
    fn renders_only_verified_tor_proxy_arguments() {
        let plan = YtDlpLaunchPlan::new("runtime/yt-dlp", ready_v4());

        assert_eq!(
            plan.arguments(),
            vec![
                OsString::from("--ignore-config"),
                OsString::from("--proxy"),
                OsString::from("socks5h://127.0.0.1:19050"),
                OsString::from("--no-plugin-dirs"),
                OsString::from("--no-js-runtimes"),
                OsString::from("--no-remote-components"),
            ]
        );
    }

    #[test]
    fn preserves_verified_ipv6_loopback_route() {
        let ready = certify_tor_ready(
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        );
        let plan = YtDlpLaunchPlan::new("runtime/yt-dlp", ready);

        assert_eq!(
            plan.arguments(),
            vec![
                OsString::from("--ignore-config"),
                OsString::from("--proxy"),
                OsString::from("socks5h://[::1]:19050"),
                OsString::from("--no-plugin-dirs"),
                OsString::from("--no-js-runtimes"),
                OsString::from("--no-remote-components"),
            ]
        );
    }

    #[test]
    fn base_plan_contains_no_media_locator() {
        let plan = YtDlpLaunchPlan::new("runtime/yt-dlp", ready_v4());

        assert_eq!(plan.arguments().len(), 6);
        assert!(
            plan.arguments()
                .iter()
                .all(|argument| !argument.to_string_lossy().contains("http://"))
        );
        assert!(
            plan.arguments()
                .iter()
                .all(|argument| {
                    let text = argument.to_string_lossy();
                    text == "--ignore-config"
                        || text == "--no-plugin-dirs"
                        || text == "--no-js-runtimes"
                        || text == "--no-remote-components"
                        || text == "--proxy"
                        || text.starts_with("socks5h://")
                })
        );
    }
}
