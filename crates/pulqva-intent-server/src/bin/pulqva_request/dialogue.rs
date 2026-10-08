//! Bounded presentation/selection used by the real caller; no network client here.
use pulqva_core::{CandidateRetrieval, CandidateSearch, FileReceipt, PendingInterpretedJourney};
use std::{fs, io::{self, BufRead, Read, Write}, path::Path};

fn line(reader: &mut impl BufRead, limit: usize) -> Result<Option<String>, &'static str> {
    let mut bytes = Vec::new();
    reader.take((limit + 3) as u64).read_until(b'\n', &mut bytes).map_err(|_| "input read failed")?;
    if bytes.is_empty() { return Ok(None); }
    if bytes.last() == Some(&b'\n') { bytes.pop(); if bytes.last() == Some(&b'\r') { bytes.pop(); } }
    if bytes.len() > limit { return Err("input exceeds byte limit"); }
    String::from_utf8(bytes).map(Some).map_err(|_| "input is not UTF-8")
}

pub(super) fn read_request(reader: &mut impl BufRead) -> Result<String, &'static str> {
    let request = line(reader, 4096)?.ok_or("request is missing")?;
    if request.trim().is_empty() || request.chars().any(char::is_control) {
        return Err("request is empty or contains controls");
    }
    Ok(request)
}

pub(super) fn create_private_dir(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    builder.create(path) // Never reuse/erase an existing directory, including symlinks.
}

pub(super) fn present_and_select<B: CandidateSearch + CandidateRetrieval>(
    pending: PendingInterpretedJourney<'_, B>, reader: &mut impl BufRead,
    writer: &mut impl Write, output_root: &Path,
) -> Result<Option<FileReceipt>, &'static str> {
    let count = pending.choices().candidates().len();
    for (index, candidate) in pending.choices().candidates().iter().enumerate() {
        // Debug-escaped strings cannot insert terminal control sequences or fake
        // new choice lines. Display titles only; never expose locators as commands.
        writeln!(writer, "{}. {:?}", index + 1, candidate.title()).map_err(|_| "presentation failed")?;
    }
    writeln!(writer, "Select 1-{count}, or q to cancel:").map_err(|_| "presentation failed")?;
    writer.flush().map_err(|_| "presentation failed")?;
    let selected = line(reader, 32)?.unwrap_or_default();
    let selected = selected.trim();
    if selected.is_empty() || selected == "q" {
        pending.cancel(); return Ok(None);
    }
    if !selected.bytes().all(|c| c.is_ascii_digit()) { return Err("invalid selection"); }
    let index = selected.parse::<usize>().ok().and_then(|n| n.checked_sub(1))
        .filter(|&n| n < count).ok_or("selection is outside the displayed choices")?;
    create_private_dir(output_root).map_err(|_| "output must be a new writable directory")?;
    // Existing CommonsSearch::retrieve performs downloaded-content verification.
    // No copy to runtime and no success receipt on a retrieval error. On failure,
    // any partial output remains for the owner; it is never announced as saved.
    pending.retrieve_selected(index, output_root).map(Some).map_err(|_| "selected retrieval failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulqva_core::{ChoiceMode, Interpretation, InterpretedIntent, JourneyError,
        SearchCandidate, SearchIntent, SelectedCandidate};
    use std::{io::Cursor, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

    struct Backend { calls: usize, fail: bool, title: String }
    impl CandidateSearch for Backend {
        fn search(&mut self, _: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
            Ok(vec![SearchCandidate::new("First", "fixture:a").unwrap(),
                SearchCandidate::new(self.title.clone(), "fixture:b").unwrap()])
        }
    }
    impl CandidateRetrieval for Backend {
        fn retrieve(&mut self, selected: &SelectedCandidate, root: &Path) -> Result<FileReceipt, JourneyError> {
            self.calls += 1;
            if self.fail { return Err(JourneyError::RetrievalFailed); }
            let path = root.join("selected.bin");
            let bytes = selected.candidate().locator().as_bytes();
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
            file.write_all(bytes).unwrap(); file.sync_all().unwrap();
            FileReceipt::new(selected, path, bytes.len() as u64)
        }
    }
    struct Root(PathBuf);
    impl Root {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!("pulqva-c3-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            create_private_dir(&path).unwrap(); Self(path)
        }
        fn out(&self) -> PathBuf { self.0.join("output") }
    }
    impl Drop for Root { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
    fn backend() -> Backend { Backend { calls: 0, fail: false, title: "Second".into() } }
    fn pending(b: &mut Backend, mode: ChoiceMode) -> PendingInterpretedJourney<'_, Backend> {
        PendingInterpretedJourney::start(b, Interpretation::Intent(InterpretedIntent::new("different human topic", mode).unwrap())).unwrap()
    }

    #[test] fn c3_explicit_second_choice_retains_selected_bytes() {
        let root = Root::new(); let mut b = backend(); let mut display = Vec::new();
        let receipt = present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(b"2\n"), &mut display, &root.out()).unwrap().unwrap();
        assert_eq!(receipt.selected_index(), 1); assert_eq!(receipt.title(), "Second");
        assert_eq!(fs::read(receipt.path()).unwrap(), b"fixture:b"); assert_eq!(b.calls, 1);
        let display = String::from_utf8(display).unwrap();
        assert!(display.contains("2. \"Second\"")); assert!(!display.contains("fixture:"));
    }
    #[test] fn c3_model_autopilot_still_requires_and_uses_the_callers_choice() {
        let root = Root::new(); let mut b = backend();
        let receipt = present_and_select(pending(&mut b, ChoiceMode::Autopilot), &mut Cursor::new(b"1\n"), &mut Vec::new(), &root.out()).unwrap().unwrap();
        assert_eq!(receipt.selected_index(), 0); assert_eq!(fs::read(receipt.path()).unwrap(), b"fixture:a"); assert_eq!(b.calls, 1);
    }
    #[test] fn c3_cancel_eof_and_blank_never_create_output_or_download() {
        for text in [b"q\n".as_slice(), b"", b"\n"] {
            let root = Root::new(); let mut b = backend();
            assert!(present_and_select(pending(&mut b, ChoiceMode::Autopilot), &mut Cursor::new(text), &mut Vec::new(), &root.out()).unwrap().is_none());
            assert_eq!(b.calls, 0); assert!(!root.out().exists());
        }
    }
    #[test] fn c3_invalid_selection_never_creates_output_or_downloads() {
        for text in ["0\n", "3\n", "-1\n", "+1\n", "1.0\n", "9999999999999999999999999999999999\n", "2;run\n"] {
            let root = Root::new(); let mut b = backend();
            assert!(present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(text), &mut Vec::new(), &root.out()).is_err());
            assert_eq!(b.calls, 0); assert!(!root.out().exists());
        }
    }
    #[test] fn c3_existing_output_is_not_reused_or_overwritten() {
        let root = Root::new(); create_private_dir(&root.out()).unwrap();
        fs::write(root.out().join("keep"), b"unchanged").unwrap(); let mut b = backend();
        assert!(present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(b"1\n"), &mut Vec::new(), &root.out()).is_err());
        assert_eq!(fs::read(root.out().join("keep")).unwrap(), b"unchanged"); assert_eq!(b.calls, 0);
    }
    #[test] fn c3_retrieval_failure_is_once_and_never_announced_as_saved() {
        let root = Root::new(); let mut b = backend(); b.fail = true; let mut display = Vec::new();
        assert_eq!(present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(b"1\n"), &mut display, &root.out()), Err("selected retrieval failed"));
        assert_eq!(b.calls, 1); assert!(!root.out().join("selected.bin").exists());
        assert!(!String::from_utf8(display).unwrap().contains("Saved"));
    }
    #[test] fn c3_untrusted_title_is_escaped_and_cannot_forge_choice_lines() {
        let root = Root::new(); let mut b = backend(); b.title = "Second\n9. injected\u{1b}[2J".into(); let mut display = Vec::new();
        present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(b"q\n"), &mut display, &root.out()).unwrap();
        let text = String::from_utf8(display).unwrap(); assert_eq!(text.lines().count(), 3); assert!(!text.contains('\u{1b}'));
    }
    #[test] fn c3_failed_presentation_stops_before_selection_and_filesystem() {
        struct Broken;
        impl Write for Broken { fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::ErrorKind::BrokenPipe.into()) } fn flush(&mut self) -> io::Result<()> { Ok(()) } }
        let root = Root::new(); let mut b = backend();
        assert!(present_and_select(pending(&mut b, ChoiceMode::Ask), &mut Cursor::new(b"1\n"), &mut Broken, &root.out()).is_err());
        assert_eq!(b.calls, 0); assert!(!root.out().exists());
    }
    #[test] fn c3_request_is_human_utf8_not_a_fixed_query_and_preserves_next_line() {
        let mut input = Cursor::new("найди птиц\r\n2\n");
        assert_eq!(read_request(&mut input).unwrap(), "найди птиц");
        assert_eq!(line(&mut input, 32).unwrap().unwrap(), "2");
    }
    #[test] fn c3_request_limits_empty_controls_and_invalid_utf8_fail() {
        for bytes in [Vec::new(), b" \n".to_vec(), b"bad\0request\n".to_vec(), vec![0xff], vec![b'x'; 4097]] {
            assert!(read_request(&mut Cursor::new(bytes)).is_err());
        }
        assert_eq!(read_request(&mut Cursor::new(format!("{}\r\n", "x".repeat(4096)))).unwrap().len(), 4096);
    }
    #[test] fn c3_selection_bytes_are_bounded_even_without_a_newline() {
        let root = Root::new(); let mut b = backend(); let mut input = Cursor::new(vec![b'1'; 100000]);
        assert!(present_and_select(pending(&mut b, ChoiceMode::Ask), &mut input, &mut Vec::new(), &root.out()).is_err());
        assert_eq!(input.position(), 35); assert_eq!(b.calls, 0); assert!(!root.out().exists());
    }
}
