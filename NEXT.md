# NEXT

## CURRENT: T069 — local intent integration

PR80 / feat/T069-intent-contract. G1 implementation: b005097ee305c9e522a838f2b5e0f5241c23396a.
State closeout parent: a97bddab3334cc85d9a00fe9188b141ec89fd6a5. Re-read live HEAD before edits.
Global verified anchor stays fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0; parent T069 is NOT complete.
Autonomous mode was cancelled. Work only on an owner command. No merge or release.

### DONE: G1 scoped deterministic acceptance

Read the completed original rust-check37128225491/job111217831711 log.
Actual checkout3f856e5c7d83bd10182ad4b17c3b0f15bc888064 is the PR synthetic merge of
b005097e into f05ce421b72badfb25cef0716661db4ecb4bab5b. Git tree25161263beb8d692ed149a70a4cff3bf5ad030fd
matches the G1 implementation tree. Ubuntu24.04.5, pinned Rust1.91.0,
cargo test --workspace --locked. Five public adapter tests and17 core unit tests passed,
including the three existing journey regressions. Discovery suite41 passed; its one
ignored environment helper ran separately through its parent. No new test was launched.

The public request_interpreted_choices adapter preserves query, candidate order and
ChoiceMode, rejects before search, propagates errors and retains the two-choice minimum.
It neither selects nor retrieves. These are fresh fixture tests, not a connected live
model/Tor/file run. A ChoiceMode value is not consent; FileReceipt alone is not verification.
Historical stdout has no per-probe timing contract; do not fabricate matrix_receipt.json
from it or rerun these successful tests merely to rename the evidence.

Current documentation-parent CI listing:14 success,2 failure. Commons37130002218 and
Arti37130002008 remain open. Status alone supplies no new failure diagnosis.

### ONE NEXT ACTION: T069-G2 executable evidence capture

Extend the EXISTING real_server_smoke harness/evaluator in pulqva-intent-server with a
bounded matrix receipt writer and verifier plus deterministic tests. Read only its
current implementation/dependencies and applicable primary sources, then implement
one coherent capture change; do not spend another pass restating the completed G1 trace.
Preserve the old receipt.json format, actual model/prompt/schema/settings and acceptance.
No new launcher, framework, provider or downloader. No real-model/network launch yet.

Before any new expensive integration generation, capture a predeclared manifest and
all probes (id,hypothesis,invariant_set,result,duration_ms,expected/observed,evidence refs)
in matrix_receipt.json, including readiness failure/early-exit SKIP or ERROR records.
Use monotonic durations; unavailable provenance/timing is explicit, never fabricated.
Validate complete/duplicate/missing IDs, provenance and aggregate status. A single PID
or serial order does not prove cache/slot/FS/transport isolation. Do not claim isolation
until the pinned implementation has been checked. Fixes require a new closed generation.
Use the protocol in AGENTS.md; deterministic capture tests must precede live evidence.

This is a prerequisite to the SAME user journey, not a substitute for integration:
local interpreter -> request_interpreted_choices -> real Tor discovery -> explicit choice
-> retrieve_choice on the SAME CommonsSearch -> existing verified-file path.
Keep pending: integration caller, real user selection/Autopilot authorization and ranking,
held-out semantics, local CORS/auth, Windows model, Commons/Arti, mid-flight Tor loss,
file ownership/races, UI/packages/clean machines and package egress.

### G1 post-test primary-source review — 2026-10-03

Rust API Guidelines type-safety and Rust Book Result support retaining semantic types
and propagating errors without a second coordinator. The observed public-API fixture
tests match that narrow design. Rust's test-runner documentation warns about shared state;
these G1 cases use fresh recording fixtures but prove no live-runtime independence.
Keep the adapter and pins; no new model experiment is justified by G1 closeout.
https://rust-lang.github.io/api-guidelines/type-safety.html
https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
https://doc.rust-lang.org/book/ch11-02-running-tests.html

Historical T068 NEXT/state: immutable a97bddab3334cc85d9a00fe9188b141ec89fd6a5 and PR73.
Historical model smoke: attempt9/run36930160538/job111091620945/artifact11260241264,
checkpoint5966597137. Do not repeat it without a new question or promote it to product E2E.
