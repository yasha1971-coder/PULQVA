//! Supervised local-process boundary for PULQVA intent inference.
//!
//! The adapter owns one child process, sends one bounded human request on stdin,
//! accepts one bounded JSON document on stdout, and converts it through the
//! strict T069-B decoder. It contains no network client, proxy, Tor capability,
//! provider SDK, shell invocation, or filesystem authority beyond the explicit
//! executable path supplied by the caller.

use pulqva_core::InterpretedIntent;
use std::{error::Error, fmt, io::{Read, Write}, path::{Path, PathBuf},
          process::{Command, Stdio}, thread, time::{Duration, Instant}};

pub const MAX_HUMAN_REQUEST_BYTES: usize = 4096;
pub const MAX_MODEL_STDOUT_BYTES: usize = 4096;
const POLL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
pub struct LocalIntentProcessPlan {
    executable: PathBuf,
    timeout: Duration,
}

impl LocalIntentProcessPlan {
    pub fn new(executable: impl Into<PathBuf>, timeout: Duration)
        -> Result<Self, LocalIntentProcessError>
    {
        let executable = executable.into();
        if timeout.is_zero() { return Err(LocalIntentProcessError::InvalidPlan); }
        Ok(Self { executable, timeout })
    }
    pub fn executable(&self) -> &Path { &self.executable }
    pub fn timeout(&self) -> Duration { self.timeout }
}

/// Runs one local interpreter invocation without a shell.
///
/// Environment is cleared deliberately. The future model sidecar must receive
/// every required local path/config explicitly; ambient proxy/API credentials
/// are not inherited by this process boundary.
pub fn interpret_with_local_process(
    plan: &LocalIntentProcessPlan,
    human_request: &str,
) -> Result<InterpretedIntent, LocalIntentProcessError> {
    validate_human_request(human_request)?;

    let mut child = Command::new(plan.executable())
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| LocalIntentProcessError::Spawn)?;

    let mut stdin = child.stdin.take().ok_or(LocalIntentProcessError::Protocol)?;
    stdin.write_all(human_request.as_bytes()).map_err(|_| LocalIntentProcessError::Io)?;
    stdin.write_all(b"\n").map_err(|_| LocalIntentProcessError::Io)?;
    drop(stdin);

    let deadline = Instant::now().checked_add(plan.timeout())
        .ok_or(LocalIntentProcessError::InvalidPlan)?;

    loop {
        match child.try_wait().map_err(|_| LocalIntentProcessError::Io)? {
            Some(status) => {
                if !status.success() { return Err(LocalIntentProcessError::ChildFailed); }
                break;
            }
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(LocalIntentProcessError::Timeout);
            }
            None => thread::sleep(POLL),
        }
    }

    let mut stdout = child.stdout.take().ok_or(LocalIntentProcessError::Protocol)?;
    let mut limited = stdout.by_ref().take((MAX_MODEL_STDOUT_BYTES + 1) as u64);
    let mut bytes = Vec::new();
    limited.read_to_end(&mut bytes).map_err(|_| LocalIntentProcessError::Io)?;
    if bytes.len() > MAX_MODEL_STDOUT_BYTES {
        return Err(LocalIntentProcessError::OutputTooLarge);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| LocalIntentProcessError::Utf8)?;
    pulqva_intent_json::parse_interpreted_intent_json(text.trim_end_matches(['\r','\n']))
        .map_err(|_| LocalIntentProcessError::InvalidOutput)
}

fn validate_human_request(value: &str) -> Result<(), LocalIntentProcessError> {
    if value.trim().is_empty() || value.len() > MAX_HUMAN_REQUEST_BYTES
        || value.chars().any(|c| c == '\0')
    {
        return Err(LocalIntentProcessError::InvalidInput);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalIntentProcessError {
    InvalidPlan, InvalidInput, Spawn, Io, Timeout, ChildFailed,
    Protocol, OutputTooLarge, Utf8, InvalidOutput,
}
impl fmt::Display for LocalIntentProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPlan => "invalid local intent process plan",
            Self::InvalidInput => "invalid human intent input",
            Self::Spawn => "failed to spawn local intent process",
            Self::Io => "local intent process I/O failed",
            Self::Timeout => "local intent process timed out",
            Self::ChildFailed => "local intent process failed",
            Self::Protocol => "local intent process protocol failed",
            Self::OutputTooLarge => "local intent output exceeded its byte limit",
            Self::Utf8 => "local intent output was not UTF-8",
            Self::InvalidOutput => "local intent output failed strict decoding",
        })
    }
}
impl Error for LocalIntentProcessError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_timeout_without_starting_any_process() {
        assert_eq!(
            LocalIntentProcessPlan::new("unused", Duration::ZERO).unwrap_err(),
            LocalIntentProcessError::InvalidPlan
        );
    }

    #[test]
    fn rejects_blank_nul_and_oversized_human_input_before_spawn() {
        let plan = LocalIntentProcessPlan::new("definitely-not-executed", Duration::from_secs(1)).unwrap();
        for value in [" ".to_owned(), "a\0b".to_owned(), "x".repeat(MAX_HUMAN_REQUEST_BYTES + 1)] {
            assert_eq!(interpret_with_local_process(&plan, &value).unwrap_err(),
                       LocalIntentProcessError::InvalidInput);
        }
    }

    #[test]
    fn missing_executable_is_a_fixed_safe_category() {
        let plan = LocalIntentProcessPlan::new(
            "pulqva-this-executable-does-not-exist-7fbc5c", Duration::from_millis(50)
        ).unwrap();
        assert_eq!(interpret_with_local_process(&plan, "find countdown video").unwrap_err(),
                   LocalIntentProcessError::Spawn);
    }
}
