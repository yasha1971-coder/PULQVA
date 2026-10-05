# NEXT

## T069-G2 — Windows compiler recovery OBSERVE

PR80 feat/T069-intent-contract; parent4349f7a2bac493fcbdf70db086487f47aec169ef.
Existing arti-sidecar-check now parses both scripts on Windows BEFORE setup,
invokes bounded isolated18.10.1 restoration and preserves its receipt even on failure.
Compiler64-file gate, SDK/Rust/Arti/executable pins and Linux build unchanged.
Only automatic PR build wave admitted; no manual dispatch/rerun or live Tor.

ONE NEXT ACTION: observe this new exact-head CI: Windows parser, signed installer,
exact installed version,64 hashes, unchanged final arti.exe SHA256. Missing receipt
or skipped/failing step cannot establish success. Preserve first concrete failure;
no hot fix/retry in this generation. PowerShell not available locally: hosted
parse/runtime/build acceptance PENDING. Local YAML structure and continuity checks
only; not compliant runtime matrix or product acceptance.

Old run37303285675 Windows111740885918 again failed compiler gate before build;
Linux111740886155 succeeded. G2D retained-file wiring remains paused,9 tests
local-only; full intent->choice->retained-file,UI/packages NOT_TESTED.
No merge/release/scheduler change; global verified anchor unchanged.
