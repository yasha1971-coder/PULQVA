# NEXT

## CURRENT: T068-B recovery — Windows regression assumed LF-only checkout

Branch feat/T068-live-tor-e2e; PR #73 remains draft. Main is untouched.
At 95664cec8afac630db02337750bbcac843fdea3f all 13 workflows completed:
12 success, only t068-production-graph-candidate 36311576973 failed.
Linux candidate 108598413781 passed preflight, prepare, Cargo, compilation and
artifact export. Windows 108598413887 passed Python syntax, JSON byte round-trip
and the actual fresh-directory PREPARE smoke test. Two LF-only byte-marker
assertions failed with count=0. This is NOT the earlier SyntaxError returning.

Exact baseline source/test Git hashes reproduced locally. With Python 3.13.5,
the baseline passes 4 tests with LF and fails the same two assertions with CRLF.
Repair changes only tests: accept one real LF or CRLF statement separator;
retain rejection of literal backslash+n, syntax validation and exact-byte JSON/
hash checks. New regression exercises BOTH newline spellings on EVERY runner.
Local repaired suite: 5/5 with LF, 5/5 with CRLF, including real PREPARE in paths
with spaces. This is local checkout-format coverage, NOT native Windows proof.
Test blob 9e7ca81dc348dfc0cc7891a4b162187bbeee86c5; 5581 bytes;
SHA256 4d507634b679fcafee4d71f60cf6eee0fa4c359c8a35f8cd1a410bdbe900633d.
GitHub create_blob hash matches the locally tested bytes before publication.

ONE NEXT ACTION: inspect the repair head's existing native candidate workflow.
On red, inspect the exact failing stage. On green, use already verified B0 lock
and manifest bytes for the substantive endpoint-private CommonsTransport; do not
create another generator or optional parser task. Publication SHA/run IDs are in
PR #73's checkpoint. Do not merge this unfinished T068 parent or call it E2E.

Existing mounted /mnt/data/t068-win2.zip and /mnt/data/t068-linux2.zip can be read
with Python zipfile; no Files materialization is needed. The full B0 Cargo.lock is
33037 bytes, SHA256 ea282fedb7128d918b428cb30e5563b44075c770cb6672bffa682fe10091f5e5.
Do not normalize lock/artifact bytes for an integrity comparison. Library adoption,
production executor, controlled TLS/body/deadline/liveness tests and same-coordinator
real external request -> choices -> selected file remain uncompleted.

Entry/exit primary-source review, 2026-09-27:
https://docs.python.org/3.12/reference/lexical_analysis.html#physical-lines
https://git-scm.com/docs/gitattributes
Python accepts LF and CRLF on every platform; Git may convert checkout line endings.
No global Git/EOL setting, shipping code, manifest/lock, Tor/kernel or permissions
changed to make the test pass. Last verified anchor remains cd785a3; new CI pending.
