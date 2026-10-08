# T069-C1 — interpreted request to one selected retrieval

Lane C, issue86/claim6042809299. Base PR87 df5ac954485920c49e422fdb1eb5e9d3dd43c648;
isolated branch feat/T069-bound-interpreted-journey. A/B/parent refs unchanged.

## Implemented

PendingInterpretedJourney composes existing request_interpreted_choices/retrieve_choice.
It holds read-only choices and an exclusive borrow of the original backend until one
consuming retrieve_selected(index, output_root) or cancellation. Ask and model-proposed
Autopilot only search; they do not authorize retrieval. Reject never calls the backend.
Default Debug excludes query/title/locator/backend. No new downloader or transport.

Ownership review: additive pending_journey.rs module and lib.rs export belong to C.
Existing journey.rs/public signatures, Cargo.lock, model, parser, pins and Kernel stay
unchanged. UI authorization/output ownership remain caller duties. This adapter is
not in-flight cancellation, protection against shared interior state/malicious backends,
publisher verification, or global enforcement on callers using lower-level APIs.
It can live in the backend worker while waiting for a selection; no self-referential
GUI state is proposed. Parent State/NEXT await serialized integration; this card and
INDEX.lane_c_pending are the isolated continuation pointer.

## Current native acceptance — observed 2026-10-07

Checkpoint6043547531 accepted original rust-check37658045135/job112918122618:
11/11 integration and2/2 core compile-fail tests executed and passed. Actual checkout
9fc8df72 and source6a91283 share treee3599ff364fa36e2f6a7b6ea459891cf6f590736.
No rerun. This is scoped synthetic-backend acceptance, not real model/Tor/UI.
Continue with tasks/READY/T069-C2-loopback-journey.md, not the historical pending
notes below. The source/acceptance chronology is preserved.

## Predeclared acceptance / historical pre-CI status

Existing cargo test --workspace --locked runs 11 added ordinary native cases: Ask waits;
Autopilot waits; Reject invokes neither service; selected fixture bytes are saved with
matching presentation/receipt; invalid index blocks retrieval; search failure has no
fallback; too few choices rejects; retrieval failure has no retry; cancel permits a new
request; drop does not download; Debug is private. Fresh owned output per file case.
Two compile-fail doctests require rejection of backend reborrowing and replay of a
consumed request. Positive integration calls exercise the same public API.

Local: source/recovery consistency only. Rust/Cargo absent, compiler host DNS unavailable;
no native test, Tor/model run or owner installation. Native acceptance PENDING, not PASS.
These are ordinary Rust tests, not fabricated matrix timings. No CI/workflow changes.
ONE NEXT ACTION: observe first exact-source rust-check, all11 integration results and
both doctests; preserve first failure before a separate correction. Then integrate the
adapter into the real interpreter/backend caller. No merge/release/scheduler change.

A remains separate: run37642119152 failed with only two successful contracts jobs in
all-jobs response. No admission/media job; cause UNRESOLVED, not an observed Tor error.
WSL recovery CLOSED: owner re-read the existing file, SHA256 again
0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423.
Historical WSL5874201467 and Windows5894551260 remain valid in fixed-fixture scope.
Do not reinstall/replay or call today's wrapper the first retained Linux/Windows file.

## Primary-source decision — 2026-10-07

https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html
https://doc.rust-lang.org/cargo/commands/cargo-test.html
https://genai.owasp.org/llmrisk/llm062025-excessive-agency/

Use exclusive borrowing/consuming selection and downstream authorization, not a model's
policy flag as consent. Post-source review matches those mechanisms; post-native review
PENDING. No new dependency, framework, exhaustive industry review or certification.
