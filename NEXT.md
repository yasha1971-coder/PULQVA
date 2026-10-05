# NEXT

## T069-G2 — Windows compiler recovery PREPARE

PR80 branch feat/T069-intent-contract, source base f2c7350b6d6684c74f2212e067edf52644127dc5.
Latest provenance checkpoint5993164002 supersedes old G2D publish queue.

Prepared scripts/restore_arti_windows_compiler.ps1; NOT connected to workflow.
Fixed official18.10.1 installer, valid Microsoft Authenticode signer required,
isolated fresh path,120s download/1200s installer bound, exact installed version,
existing64 compiler hashes before exporting compiler environment. SDK/Rust/Arti
and executable pins unchanged. Receipt records failures and measured installer digest;
that digest is not a preauthenticated content pin. BuildTools equivalence is unproved.
No local PowerShell runtime: parsing, installer execution and Windows acceptance NOT_TESTED.

ONE NEXT ACTION: review/parse candidate on Windows, then wire bounded recovery
and evidence upload into existing arti-sidecar-check workflow as one atomic phase.
Do not rerun unchanged failing build. Retained-file wiring paused pending red-boundary
reconciliation; G2D9 tests remain local-only. No live Tor, merge/release or scheduler change.
Global verified anchor unchanged. Separate no-jobs workflow37292162029 remains unexplained.
