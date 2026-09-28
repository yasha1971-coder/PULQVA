# NEXT

## CURRENT: T068-C2 bounded artifact verification

PR73 / feat/T068-live-tor-e2e. Entry head
93d84cc1d547908ccc80f34ee79c30a3be7a619a. Main unchanged.
Global verified anchor remains fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0;
Linux C1 and hash dependency adoption have separate scoped evidence below.

The owner imported the native Cargo.lock in 0ce30e4; 93d84cc activated exactly
sha1=0.11.0 and sha2=0.11.0. Recomputed the whole native artifact Git blob locally:
d76600cb071e21703d616b7a6958c849d4e25f72, matching the repository file, 35109 bytes.
SHA-256 c707e6c96369aa3e07a5667cbd1a8c85f68480af042b9fabd409204735cf377d.
No new dependency/lockfile change or temporary generator is needed.

Entry CI completed: 12/13 associated workflows successful. rust-check36386301755
and discovery-contract-check36386301680 passed. commons36386301857 has successful
all-target preflight and builds on both OSes; Linux108812291888 passed live and
positive receipt, Windows108812292103 failed strict live. Do not infer Windows
success from other green checks or restart the continue-on-error policy loop.

ONE implementation outcome: verify an already completed typed download against
CommonsSearch.last_results(). The new artifact module adds
verify_completed_download(selection, CompletedDownloadResult) and an opaque
VerifiedCommonsFile. It checks same-index candidate value, exact source locator,
completion/actual/declared size, regular-file status, bounded streaming SHA-1
against provider metadata and computed local SHA-256. Only then does this module
construct its core FileReceipt. Errors and default Debug omit paths/identifiers.
Existing CandidateRetrieval remains fail-closed; live download wiring is NOT done.

Tests are in a dedicated artifact/tests.rs, not inserted into a sibling module.
Nine tests prepared (8 shared, 1 Unix symlink), plus private-construction doctest.
Cover positive real local file, mismatched selection/source, corrupt same-size
bytes, empty/truncated/oversized files, missing/directory paths, bounded overread,
short/interrupted reads, safe I/O errors, million-byte known answers and symlink.
They call the same private verifier as the public typed wrapper. Synthetic local
fixtures are not real yt-dlp downloads. Four hash vectors were independently
checked with Python; module paths, original lib blob and new blobs checked locally.
No local Rust compiler exists here; these Rust tests are PENDING native execution.

Boundaries: SHA-1 is legacy metadata compatibility, not publisher authentication.
SHA-256 records local content identity, not an externally trusted expected hash.
Index/candidate equality does not encode a search-session nonce. Same-valued
selections from another search cannot be distinguished. Filesystem validation is
a snapshot assuming exclusive output ownership, not atomic no-follow/race safety.
No proof of same-flow Tor routing, cancellation, actual Arti loss or file download
is added by a content verifier. These limitations must not be marketed away.

ONE NEXT ACTION: inspect this change's exact-head compile/tests, including the new
artifact module and doctest, on Windows/Linux. No dependency refresh or blind rerun.
After successful verification, a later atomic step wires the existing typed
Tor/yt-dlp downloader into the coordinator and invokes this verifier before a
success receipt; it must retain downloaded bytes and supervise the owned process.
Strict Windows C1 remains an independent open release gate. Full E2E is NOT done.

Owner assistance is available for confirmed tool barriers: PR73 comment5864661936.
Prepare a minimal safe external action promptly when needed, then read back and
verify its result. Do not assume a barrier without trying the available tools.

Primary-source entry/exit review, 2026-09-28:
https://docs.rs/sha1/latest/sha1/
https://docs.rs/sha2/0.11.0/sha2/
https://www.mediawiki.org/wiki/API:Imageinfo
https://doc.rust-lang.org/std/fs/
https://doc.rust-lang.org/std/io/trait.Read.html
