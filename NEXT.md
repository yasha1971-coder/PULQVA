# NEXT

## CURRENT: T068-B recovery — real byte correction and executed offline preflight

Branch feat/T068-live-tor-e2e, PR #73 draft; main untouched.
Parent ec8c4380b74908c71aa4e2c6c7d79f643fe3f1c7 still failed PREPARE on
Windows/Linux in candidate run 36310993392. Linux job 108596747790 and the
actual source prove literal backslash+n remained at line 11. The preceding
claim that read-back proved a newline was incorrect; do not repeat it.

Reproduced SyntaxError locally from bytes matching Git blob abdaee56210398aea3f3548d73eee9ce1de58b44.
Corrected exactly one byte sequence with a required match/change assertion.
Corrected script: 1779 bytes; Git blob 3ed6ee12f5b686727eb3c06e51b4290f3d6fae75;
SHA256 d242b73c12f3af14d58c51cb11655e73793fa78b7e32eb40000a93ed3cb00b71.
Local Python 3.13.5: py_compile passed and FOUR offline regressions passed:
source syntax, historical corruption rejection, JSON round-trip byte identity,
and actual prepare in a fresh directory with spaces/missing examples, checking
copied probe bytes, valid manifest pins and no source/lock changes. No Cargo run.
GitHub blob identity matched locally computed hashes before branch publication.

CI now runs Python syntax/regressions before installing Rust. Upload is gated
on a nonempty T068_PROD so a failed prepare never expands artifact paths to /.
No shipping manifests/locks, Rust executor, kernel, Tor settings or permissions changed.

ONE NEXT ACTION: inspect the new recovery head's candidate native preflight/Cargo
steps; diagnose any red using the exact failing stage. Do not count local Python
success as Windows/Linux Cargo success, live discovery or E2E. No unchanged retries.
New commit/run evidence is in PR #73's checkpoint; current native CI is PENDING.

Already downloaded B0 artifacts are ordinary mounted ZIP files: this recovery
read /mnt/data/t068-win2.zip and /mnt/data/t068-linux2.zip directly, and both full
Cargo.lock members hash to ea282fedb7128d918b428cb30e5563b44075c770cb6672bffa682fe10091f5e5
(33037 bytes). Do not use Files materialization on these mounted artifacts or
invent another graph generator to solve a byte-transfer problem. Adoption is NOT
performed here; the verified existing graph must accompany the real endpoint-private
CommonsTransport implementation after recovery. B/C and real E2E remain unfinished.

Primary sources checked on 2026-09-27:
https://docs.python.org/3.12/library/py_compile.html
https://docs.python.org/3.12/library/pathlib.html
https://docs.github.com/en/rest/git/blobs
