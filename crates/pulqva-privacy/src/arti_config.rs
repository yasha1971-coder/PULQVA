use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use crate::TorSocksEndpoint;

/// Pure-data renderer for PULQVA's validated Arti 2.6.0 configuration contract.
///
/// Rendering does not write files, spawn processes, open sockets, or bootstrap Tor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtiConfigSpec {
    socks: TorSocksEndpoint,
    cache_dir: PathBuf,
    state_dir: PathBuf,
}

impl ArtiConfigSpec {
    pub fn new(
        socks: TorSocksEndpoint,
        cache_dir: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            socks,
            cache_dir: cache_dir.into(),
            state_dir: state_dir.into(),
        }
    }

    pub fn socks_endpoint(&self) -> TorSocksEndpoint {
        self.socks
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn render(&self) -> Result<String, ArtiConfigRenderError> {
        let cache_dir = utf8_path(&self.cache_dir, "cache_dir")?;
        let state_dir = utf8_path(&self.state_dir, "state_dir")?;

        Ok(format!(
            "[application]\n\
defer_bootstrap = true\n\
watch_configuration = false\n\
\n\
[proxy]\n\
socks_listen = {}\n\
dns_listen = 0\n\
\n\
[storage]\n\
cache_dir = {{ literal = \"{}\" }}\n\
state_dir = {{ literal = \"{}\" }}\n",
            self.socks.port(),
            escape_toml_basic_string(cache_dir),
            escape_toml_basic_string(state_dir),
        ))
    }
}

fn utf8_path<'a>(
    path: &'a Path,
    field: &'static str,
) -> Result<&'a str, ArtiConfigRenderError> {
    path.to_str()
        .ok_or(ArtiConfigRenderError::NonUtf8Path { field })
}

fn escape_toml_basic_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => {
                use fmt::Write as _;
                write!(&mut escaped, "\\u{:04X}", c as u32)
                    .expect("writing to String cannot fail");
            }
            c => escaped.push(c),
        }
    }

    escaped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtiConfigRenderError {
    NonUtf8Path { field: &'static str },
}

impl fmt::Display for ArtiConfigRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUtf8Path { field } => {
                write!(f, "Arti configuration path {field} must be valid UTF-8")
            }
        }
    }
}

impl Error for ArtiConfigRenderError {}

#[cfg(test)]
mod tests {
    use super::ArtiConfigSpec;
    use crate::TorSocksEndpoint;
    use std::path::Path;

    fn canonical_spec() -> ArtiConfigSpec {
        ArtiConfigSpec::new(
            TorSocksEndpoint::new(19050).expect("canonical SOCKS port is non-zero"),
            "runtime/cache",
            "runtime/state",
        )
    }

    #[test]
    fn keeps_typed_endpoint_and_paths_explicit() {
        let spec = canonical_spec();

        assert_eq!(spec.socks_endpoint().port(), 19050);
        assert_eq!(spec.cache_dir(), Path::new("runtime/cache"));
        assert_eq!(spec.state_dir(), Path::new("runtime/state"));
    }

    #[test]
    fn canonical_render_matches_verified_fixture_byte_for_byte() {
        let rendered = canonical_spec().render().expect("canonical paths are UTF-8");
        let fixture = include_str!("../../../sidecars/arti/pulqva.toml");

        assert_eq!(rendered.as_bytes(), fixture.as_bytes());
    }

    #[test]
    fn render_keeps_fail_closed_configuration_flags() {
        let rendered = canonical_spec().render().expect("canonical paths are UTF-8");

        assert!(rendered.contains("defer_bootstrap = true"));
        assert!(rendered.contains("watch_configuration = false"));
        assert!(rendered.contains("dns_listen = 0"));
        assert!(rendered.contains("socks_listen = 19050"));
    }

    #[test]
    fn render_escapes_toml_path_characters() {
        let spec = ArtiConfigSpec::new(
            TorSocksEndpoint::new(19050).expect("port is non-zero"),
            r#"runtime\cache "quoted""#,
            r#"runtime\state"#,
        );

        let rendered = spec.render().expect("test paths are UTF-8");

        assert!(rendered.contains(r#"cache_dir = { literal = "runtime\\cache \"quoted\"" }"#));
        assert!(rendered.contains(r#"state_dir = { literal = "runtime\\state" }"#));
    }
}
