# T069-C3 — executable human request to real Commons backend

Claim6055453068 in #86. Isolated feat/T069-real-commons-caller from accepted C2
288e32eb718f4c0ee3f0d3a2e206b52d753dda7c (PR90/checkpoint6053667433).

## One implemented outcome

The existing intent-server package now contains the actual `pulqva_request` backend
executable. Human request on stdin -> owned IntentServer::request_choices -> strict
interpretation -> actual CommonsHttpsTransport/CommonsSearch over owned Linux Arti
-> displayed titles -> separately entered one-based index -> the SAME backend's
existing retrieval/content verifier -> retained output outside disposable runtime.
No fixed query, predefined URL/index, second downloader, model flag as approval,
raw-request fallback or automatic retry. Empty/q/EOF cancels without output creation.
Errors and setup diagnostics are fixed categories. Titles are escaped for terminal
presentation. Input byte caps apply before allocation growth; output is exclusively
created only after selection. Tor/model owners stop on ordinary error paths. Runtime
cleanup cannot delete the disjoint selected output. Abrupt kills, in-flight watchdogs,
malicious filesystem races and descendant confinement are not established here.

This is a user-driven DEVELOPMENT backend entry, not a replacement product goal or
ready Tauri GUI. It requires trusted operator-supplied already-verified pinned binary
and model paths, plus fresh absolute output/runtime directories. It does not itself
validate release signatures or binary pins. No installer/downloader is introduced.
Do not ask the owner to run it before native admission and an approved immutable
model/Tor evidence generation. Fixed existing ports19081/19050 must be free; no retry.
Linux owned Arti only. Non-Linux live invocation fails explicitly; Windows must retain
its official-Tor ADR, not silently adopt Arti. Windows GUI integration remains pending.

## Method before the real generation

11 caller unit tests (synthetic backend; not real Commons/model) cover explicit first/
second selection, proposed Autopilot awaiting selection, cancellation/EOF/blank,
invalid/overflow indices, existing output preservation, terminal retrieval failure,
escaped presentation, presentation failure, human UTF-8/CRLF, input caps and invalid
encoding, and bounded selection reads. Every filesystem case owns a new namespace.
3 binary integration cases actually invoke the development executable: help, missing
configuration, invalid request before child/runtime creation (last one Linux only).
Existing cargo test --workspace --locked discovers the binary and all14 Linux cases.
No new generic test framework. These are native prerequisites, not retrospective
matrix receipts or performance comparisons. Native execution is PENDING in CI.

No local Rust/Cargo is available on PATH/standard locations. Local source checks are
not substitute native acceptance. Registry package versions, checksums and features
are unchanged: only two existing internal package edges are added to intent-server
(discovery/privacy), mirrored in the existing root Cargo.lock. The existing desktop
lock has no intent-server edge and remains untouched. A/B/parent refs are untouched.
Single integration writer for this isolated lock/seal delta; no kernel/source rewrite.
C2 card is reconciled with its recorded native acceptance. Shared parent State/NEXT
remain untouched until serialized integration; this card/INDEX.lane_c_pending is the
current C-lane pointer. Historical Windows/WSL file evidence and laptop recovery stay
closed; no reinstallation, writable remount or old-test replay.

## Primary-source review — 2026-10-08

- https://doc.rust-lang.org/cargo/reference/cargo-targets.html
- https://doc.rust-lang.org/std/io/trait.BufRead.html
- https://doc.rust-lang.org/std/io/trait.Read.html#method.take
- https://genai.owasp.org/llmrisk/llm062025-excessive-agency/

Use an actual Cargo binary target with production calls, not another unused adapter.
Bound input before read growth; explicit downstream selection, not LLM authority.
Post-source review retains these decisions; post-native/live review remains pending.
No upgrade, exhaustive worldwide survey, new framework or certification is claimed.

ONE NEXT ACTION: observe the exact new-head automatic rust-check, actual checkout/
tree, all11 binary-unit +3 binary-integration outcomes and existing C1/C2 regressions.
Preserve any first failure before a separate correction. Then prepare ONE bounded
new full model+Tor caller generation with the existing manifest/matrix collector;
no blind replay of A's unrelated failed dispatch37642119152. Its missing-downstream
cause remains unresolved. Real C3 file, GUI, native Windows integration, packages,
clean-machine/privacy acceptance are NOT TESTED. No merge/release/scheduler.
