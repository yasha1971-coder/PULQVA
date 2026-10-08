# T069-C2 — real interpreter call to explicit selected-file journey

Lane C / issue #86 / claim6049510580, 2026-10-08. One atomic connection.
Base C1: 6a91283b04dc13bf74ac0738a719f75aa54ec9e3; isolated branch
feat/T069-loopback-journey. Parent/A/B refs must not be advanced by this task.

## Accepted starting point

C1's original run37658045135/job112918122618 accepted 11 integration cases and
2 compile-fail contracts at treee3599ff364fa36e2f6a7b6ea459891cf6f590736,
checkpoint6043547531. Do not repeat that acceptance or the historical laptop runs.
A/PR88 failed dispatch37642119152 remains separate and unresolved. B/PR85's
FFmpeg/desktop correction remains accepted within checkpoint6035299889 scope.
The three lanes are task ownership, NOT three simultaneously running agents.

## One product outcome

The existing IntentServer now exposes request_choices. It checks its owned child
is still running, caps inference at the existing completion budget, and calls the
existing HTTP interpreter with the compiled production policy/schema. The response
crosses the unchanged strict JSON/core boundary before the accepted C1 pending
journey can search. The same backend remains borrowed until explicit selection or
cancellation. No second downloader, interpreter protocol, model or process launcher.

The new request_loopback_choices in pulqva-intent-http composes existing production
functions. It rejects blank/NUL/oversized human input before HTTP, returns fixed safe
error categories, and refuses raw-input fallback after interpretation failure.
Model-proposed Autopilot still has no execution authority. Search/retrieval errors
remain terminal, not instructions for automatic re-inference or retry.

Ownership extension: HTTP library module/export/tests and supervised-server method/
tests are C's caller integration. No A media/collector/workflow changes, B sidecar/
desktop changes, core/parser/schema/policy/Cargo.lock changes. Shared recovery seal
has this integrator only; parent State/NEXT remain unchanged for later integration.
This card and INDEX.lane_c_pending supersede their stale C1 pending wording.

## Method declared before native execution

14 actual loopback tests in t069_loopback_journey.rs; each owns a listener bound to
127.0.0.1:0, captured wire bytes and a joinable bounded fixture worker. Output cases
use an exclusively created namespace. No shared environment mutation or external
network. Positive model response differs from human text to detect raw passthrough.
The request must carry the complete unchanged schema/policy/flags and exactly two
roles. No model weights run; JSON responses and download backend are synthetic.

Expected cases (all must pass, zero ignored):
- wire/schema/policy -> interpreted query -> waits -> explicit index -> actual saved fixture bytes;
- model Autopilot waits and cancel does not download;
- semantic Reject: zero search/retrieval;
- malformed model text: zero search/retrieval, no raw fallback;
- extra authority field: strict parser rejection before search;
- HTTP503: one inference attempt, zero backend calls;
- empty inference choices: no fabricated search results;
- inference deadline: bounded terminal error, no search or retry;
- invalid human text: no HTTP or backend calls;
- URL disguised as query: core rejection before search;
- search failure: no repeated inference;
- invalid selected index: no retrieval/file;
- retrieval failure: one attempt, no second inference/file;
- diagnostics do not echo private user/model text.

Two tests in t069_owned_journey.rs exercise the actual public IntentServer method:
owned child -> real loopback client/parser -> pending choices -> explicitly saved
fixture bytes; zero completion budget must not grant backend access. The existing
server_fixture models a child, not llama.cpp. The existing nonzero-port child API
has a reserve/release-before-spawn gap; bind/spawn conflict fails, never retries.
No claim that unit fixtures prove hostile local-service isolation.

The existing cargo test --workspace --locked discovers all16 new tests. This is
deterministic native prerequisite coverage, not a reconstructed matrix or live
model/Tor evidence. No per-case duration is invented from ordinary Cargo output.
Before ANY later expensive model/Tor run, use the existing separately frozen
manifest/matrix_receipt capture; these unit results cannot be relabelled as that.

## Current status / bounded handoff

IMPLEMENTED, native execution PENDING. Rust/Cargo are not on this session's PATH;
no owner installation or old baseline replay requested. Local source/recovery
checks are separate from native tests. Publish one candidate and observe one
automatic hosted CI wave; do not mix repairs into its test generation.

ONE NEXT ACTION: inspect the exact published head's original rust-check for all14
loopback plus2 owned-server tests and existing regressions; inspect the first failure
without rerun. Once accepted, connect an actual privacy-gated Commons backend to
this public server method in the real user caller. Actual model+Tor+human GUI choice,
Windows native integration, package/clean-machine and security acceptance remain OPEN.
The backend trait is not itself evidence of Tor routing or file authenticity.

## Applicable primary review — 2026-10-08

https://doc.rust-lang.org/cargo/commands/cargo-test.html
https://doc.rust-lang.org/std/net/struct.TcpListener.html
https://genai.owasp.org/llmrisk/llm062025-excessive-agency/

Use actual integration targets, separately reserved listeners, and downstream caller
authorization. Current pre/post-source review supports using the pinned components;
no upgrade or additional platform is justified. Post-native review remains pending.
This is not a standards certification or exhaustive state-of-art survey.

No merge/release, live dispatch, scheduler/agent activation or new laptop action.
