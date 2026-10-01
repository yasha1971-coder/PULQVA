//! Opt-in diagnostics for fixed-public-fixture smoke runs. Not a readiness policy.
use std::{io::{self, Read}, sync::{mpsc, Arc, Mutex}, thread, time::Duration};

pub const STDERR_RETAINED_BYTES: usize = 16 * 1024;

#[derive(Default)]
struct Tail {
    bytes: Vec<u8>,
    total: u64,
    eof: bool,
    read_error: Option<String>,
}
impl Tail {
    fn push(&mut self, data: &[u8]) {
        self.total = self.total.saturating_add(data.len() as u64);
        if data.len() >= STDERR_RETAINED_BYTES {
            self.bytes.clear();
            self.bytes.extend_from_slice(&data[data.len() - STDERR_RETAINED_BYTES..]);
        } else {
            let excess = (self.bytes.len() + data.len()).saturating_sub(STDERR_RETAINED_BYTES);
            self.bytes.drain(..excess);
            self.bytes.extend_from_slice(data);
        }
    }
}
struct StderrCapture {
    tail: Arc<Mutex<Tail>>,
    finished: mpsc::Receiver<()>,
}
impl StderrCapture {
    fn start(mut reader: impl Read + Send + 'static) -> io::Result<Self> {
        let tail = Arc::new(Mutex::new(Tail::default()));
        let destination = Arc::clone(&tail);
        let (done, finished) = mpsc::channel();
        // Drain after reaching the retention cap: stopping reads could block the child.
        // A descendant holding the pipe open cannot block the caller's shutdown:
        // settle() has a bound and the receipt explicitly records whether EOF was seen.
        thread::Builder::new().name("intent-stderr".into()).spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        destination.lock().unwrap_or_else(|p| p.into_inner()).eof = true;
                        break;
                    }
                    Ok(n) => destination.lock().unwrap_or_else(|p| p.into_inner()).push(&buffer[..n]),
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        destination.lock().unwrap_or_else(|p| p.into_inner()).read_error = Some(format!("{:?}", e.kind()));
                        break;
                    }
                }
            }
            let _ = done.send(());
        })?;
        Ok(Self { tail, finished })
    }
}

#[derive(Default)]
pub struct ReadinessDiagnostics {
    pub(crate) phase: &'static str,
    pub(crate) elapsed_ms: u64,
    pub(crate) health_attempts: u64,
    pub(crate) health_ok_seen: bool,
    pub(crate) health_bytes: usize,
    pub(crate) health_status: Option<u16>,
    pub(crate) health_sample_hex: String,
    pub(crate) health_error: Option<String>,
    pub(crate) completion_attempts: u64,
    pub(crate) completion_error: Option<String>,
    pub(crate) process_error: Option<String>,
    pub(crate) ready: bool,
    pub(crate) capture_enabled: bool,
    capture: Option<StderrCapture>,
}
impl ReadinessDiagnostics {
    pub(crate) fn capture(&mut self, reader: impl Read + Send + 'static) -> io::Result<()> {
        self.capture = Some(StderrCapture::start(reader)?);
        Ok(())
    }
    pub(crate) fn health_response(&mut self, bytes: &[u8]) {
        self.health_bytes = bytes.len();
        self.health_status = bytes.split(|b| *b == b' ').nth(1)
            .and_then(|b| std::str::from_utf8(b).ok()).and_then(|s| s.parse().ok());
        if self.capture_enabled {
            self.health_sample_hex = bytes.iter().take(512).map(|b| format!("{b:02x}")).collect();
        }
    }
    /// Call after the server owner was dropped. This does not wait indefinitely for pipe EOF.
    pub fn settle_stderr(&self) {
        if let Some(capture) = &self.capture {
            let _ = capture.finished.recv_timeout(Duration::from_millis(100));
        }
    }
    pub fn stderr_bytes(&self) -> Vec<u8> {
        self.capture.as_ref().map(|c| c.tail.lock().unwrap_or_else(|p| p.into_inner()).bytes.clone()).unwrap_or_default()
    }
    pub fn report(&self) -> serde_json::Value {
        let stderr = self.capture.as_ref().map(|c| {
            let t = c.tail.lock().unwrap_or_else(|p| p.into_inner());
            serde_json::json!({"observed_bytes":t.total,"retained_bytes":t.bytes.len(),
                "truncated":t.total > t.bytes.len() as u64,"eof":t.eof,"read_error":t.read_error})
        });
        serde_json::json!({
            "phase":self.phase,"elapsed_ms":self.elapsed_ms,"ready":self.ready,
            "health_attempts":self.health_attempts,"health_ok_seen":self.health_ok_seen,
            "health_response_bytes":self.health_bytes,"health_status":self.health_status,
            "health_read_limit_reached":self.health_bytes == 512,
            "health_sample_hex":self.health_sample_hex,"health_error":self.health_error,
            "completion_attempts":self.completion_attempts,"completion_error":self.completion_error,
            "process_error":self.process_error,
            "completion_io_timeout_ms":500,"health_io_timeout_ms":100,
            "stderr_capture_enabled":self.capture_enabled,"stderr_retention_limit_bytes":STDERR_RETAINED_BYTES,
            "stderr":stderr
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tail_is_bounded_and_preserves_non_utf8_bytes() {
        let input: Vec<u8> = (0..(STDERR_RETAINED_BYTES * 4 + 3)).map(|i| (i % 256) as u8).collect();
        let mut tail = Tail::default();
        for chunk in input.chunks(101) { tail.push(chunk); assert!(tail.bytes.len() <= STDERR_RETAINED_BYTES); }
        assert_eq!(tail.total, input.len() as u64);
        assert_eq!(tail.bytes, input[input.len() - STDERR_RETAINED_BYTES..]);
        assert!(std::str::from_utf8(&tail.bytes).is_err());
    }
    #[test]
    fn capture_drains_beyond_retention_limit_to_eof() {
        let input = vec![0xff; STDERR_RETAINED_BYTES * 8];
        let capture = StderrCapture::start(io::Cursor::new(input.clone())).unwrap();
        capture.finished.recv_timeout(Duration::from_secs(2)).unwrap();
        let tail = capture.tail.lock().unwrap();
        assert!(tail.eof);
        assert_eq!(tail.total, input.len() as u64);
        assert_eq!(tail.bytes.len(), STDERR_RETAINED_BYTES);
        assert!(tail.read_error.is_none());
    }
}
