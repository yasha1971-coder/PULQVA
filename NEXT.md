# NEXT

## CURRENT: T068-C2 first Linux live request -> choice -> file acceptance

PR73 / feat/T068-live-tor-e2e. Parent 2000e8581a403a31d9001f9bbd9f950012e28ee3.
Main and frozen kernel unchanged. Global verified anchor remains
fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0. No full E2E result is claimed yet.

Parent observation: rust-check36413808729 and discovery-contract-check36413808727
passed. Of 13 associated workflows, 12 passed. In commons36413808736, Linux
job108900033599 passed discovery and positive receipt; Windows108900033893 passed
preflight/build and failed strict live. No new Windows root cause was inferred.
The existing CandidateRetrieval now calls the typed yt-dlp launcher/completion
and artifact verifier; its positive live path has NOT been exercised together.

ONE change: activate that exact core path in a fixed public Linux acceptance
harness real_commons_file. Request 'countdown' -> actual Commons ChoiceSet ->
explicit returned index1 -> retrieve_choice on the same CommonsSearch -> existing
size/SHA-1 verifier -> FileReceipt. No direct URL shortcut or second downloader.
The harness copies the verified bytes to selected.webm outside the Tor runtime,
computes hashes while copying, stops/reaps Arti and removes the owned runtime,
then writes receipt.json. Cancellation/stale-search regression runs AFTER retrieval.
Only the public fixed request and result metadata are retained for this CI fixture.

Existing commons workflow: both harnesses compile in the unchanged full-target
preflight. Linux reuses pinned yt-dlp 2026.08.19 plus repository SHA256SUMS before
execution. Linux live invokes the compiled C2 harness under GNU timeout (300s,
TERM then KILL after10s, process group; no --foreground). A separate Python
hashlib oracle checks exact source SHA, selected metadata, size, SHA-1 and SHA-256
against the SAVED FILE before artifact upload. Bundle must contain exactly two
regular files: receipt.json and selected.webm. Missing file, corrupt content or
invalid receipt cannot pass. Windows continues its existing strict C1 harness.
No continue-on-error, dependency, lockfile, core/library or permission change.

Local checks: 11 new Python tests passed (including actual verifier CLI success
and failure). These use synthetic local fixtures, not downloaded WebM evidence.
YAML, Bash syntax and step-ID compatibility checked; native Rust remains PENDING.
No cargo/rustc exists locally; an official Rust download check failed DNS. Do not
claim Rust compilation from textual checks or that unit tests prove Tor routing.

Important limits: the current production completion uses blocking Child::wait;
the test watchdog is NOT a product supervisor. Mid-flight Arti loss/cancellation,
descendant confinement, bounded download growth and T066 filesystem race/cleanup
limits remain open. SHA-1 is provider compatibility, not publisher authentication;
SHA-256 identifies saved bytes. Selection value binding is not a session nonce.
First positive Linux trace does not prove Windows, UI/AI, packaging or repetition.

ONE NEXT ACTION: inspect the new exact-head all-target compile and live stage.
If Linux succeeds, download its commons-live-result-ubuntu-24.04 ZIP, independently
check the ZIP digest and both file hashes, and read its receipt/source SHA. Do not
announce E2E from a workflow color or stdout marker alone. Then record whether the
first positive trace is verified; a second fresh-state run is a later task.
If code fails, read complete errors once; repair only the established cause.
Publication commit and native run IDs are in the latest PR73 checkpoint.

Owner assistance rule: PR73 comment5864661936. Use a minimal safe owner action
promptly for a confirmed tool barrier, then independently verify the result.
Entry checkpoint5869245117. Prior detailed history remains in Git/PR comments.
Primary-source review 2026-09-28: GNU timeout invocation/process-group semantics;
GitHub artifact upload missing-file failure and retention. Existing project pins
were retained; no latest dependency or new infrastructure was adopted.
https://www.gnu.org/software/coreutils/manual/html_node/timeout-invocation.html
https://docs.github.com/en/actions/tutorials/store-and-share-data
