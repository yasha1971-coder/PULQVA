# ARCHITECTURE-CHOICE-01 — minimize complexity without weakening the privacy kernel

Date: 2026-11-03
Status: DECISION COMPLETE (static review); Windows inference confinement NOT TESTED
Baseline: PR #101, `fix/windows-official-tor-runtime`, commit `acd9c8f133ee6afb6183f709524998e13c8b905d`.
Type: ADR only; no implementation or release authorization.

## North star and drift test

PULQVA must let a nontechnical user **download one package, open it without a terminal, account or API key, describe a file naturally, see real choices discovered through Tor, explicitly select and retrieve a locally byte-verified file**. Explicit Autopilot and Windows/Linux parity remain contractual future requirements.

The normative constraints are `kernel/CORE_CONTRACT.md`, `kernel/PRIVACY_INVARIANTS.md`, `kernel/UX_CONTRACT.md`, and `AGENTS.md`. Especially: no direct-network fallback; UI has no external network authority; AI produces typed validated data, not commands; privacy regressions block release.

Optimize **verified user value per necessary component**, not lines of code or process count in isolation.

## Evidence — keep different proof levels separate

- Owner-confirmed Windows E2E (app baseline `3bf040ea61bc010f7e0b1db88187edfe3db8c35e`): local Qwen → validated Rust Intent → Commons results via existing Tor backend → manual selection → downloaded file 5,709,008 bytes → independently computed SHA-256 `188e86965295906e8500983a3d530390ca2c55184175873c01485670cc1ab82e`. This confirms a user journey, not egress isolation, cleanup, coverage or clean-machine distribution.
- `interpret_local.rs`: fixed manually started development endpoint `127.0.0.1:18081`; GUI does not own the model. `pulqva-intent-http`: bounded HTTP/JSON envelope, pinned system prompt and typed Rust validation.
- `pulqva-intent-server`: currently owns one `Child`, but does not establish full process-tree cleanup, endpoint authenticity or model egress denial.
- Commit `acd9c8f`: `windows_isolation.rs` is **`#[cfg(all(windows, test))]`**, not wired to GUI. It implements a candidate zero-network-capability AppContainer, limited inherited anonymous pipes, suspended creation → Job assignment → resume, and cleanup handling. No model asset ACL/identity verification, framing protocol, llama-server pipe inference, real Windows IPv4/IPv6/DNS/proxy denial or actual Qwen run is qualified.
- For commit `acd9c8f`, 16 PR workflow runs completed SUCCESS; compilation/tests cannot be upgraded to independent Windows sandbox acceptance.
- The repo's global `PROJECT_STATE.json`/`NEXT.md` still hold separate T069 scope. This ADR does not rewrite or override that workstream.

## Options and decision

| Option | Component/UX cost | Trust / failure boundary | Disposition |
| --- | --- | --- | --- |
| A: manual localhost `llama-server` | Existing proven development path; violates ZERO CONFIG | Separate process; network egress and foreign-listener ownership unproved | **PRESERVE as fixed control**, not release |
| B: owned `llama-server` in AppContainer with host↔child IPC | Candidate Windows launcher exists; bridging HTTP-only server to bounded pipes may add complexity | Separate OS-backed boundary possible, real denial and resource access untested | **PREFERRED NEXT EXPERIMENT**, not approved integration |
| C: native `llama.cpp` in trusted GUI/Rust process | Removes HTTP/server but adds FFI, model lifecycle and grammar integration | Shares host privileges and crash domain; eliminating HTTP does not prove denied egress | **DEFER** absent approved threat-model change |
| D: dedicated sandboxed native inference worker with bounded pipe protocol | Potentially less protocol mediation than B but requires new native worker and semantic-equivalence tests | Separate sandbox possible | Reconsider **only if B's measured protocol cost is excessive** |

A model of approximately 3 GB remains approximately that size under any of these arrangements. Neither C/C++ minimalism nor a C ABI makes the GGUF disappear. No named embedded C ABI consumer exists; defer a public C SDK.

**DECISION:** keep A unmodified; test B by the smallest independent, falsifiable Windows IPC+egress probe. Do not start a speculative HTTP bridge, GUI rewrite, new native inference engine, release or system firewall change. If B proves incompatible at reasonable complexity, produce a new measured architecture decision before authorizing D or C.

## Exactly one next bounded action

**WIN-ISOLATION-02-PROBE**: with a frozen source/binary and a *real non-admin Windows witness*, prove or refute a bounded two-way host↔AppContainer pipe exchange on the existing candidate launcher. Predeclare a test matrix and capture an `AGENTS.md`-compliant `matrix_receipt.json`:

1. Attest actual AppContainer token/SID and zero network capabilities.
2. Send one length-bounded request and response; malformed/oversize/EOF/timeout fail closed.
3. Attempt external IPv4, IPv6, DNS and proxy egress, preserving diagnostics and cause of each denial; never enable exemptions or alter owner security policy.
4. Prove Job termination/child and descendants cleanup on normal exit, crash, timeout, and no interference with foreign PIDs.
5. Record executable/model/DLL SHA/provenance and ACL observations; distinguish fixture child from real Qwen.

If a trustworthy Windows witness is unavailable, return PENDING/NOT TESTED rather than claiming success. **Do not** claim owned-model-start or release DONE based on this probe. Work in one bounded phase, checkpoint and STOP per `AGENTS.md`.

## Product gates after this decision (not automatic task authorization)

- G0 **PASS (owner report):** existing Windows path through verified saved file; preserve exact known-good package.
- G1 **BLOCKED:** owned, confined local Qwen startup without terminal, foreign endpoint or leftover child.
- G2 **PENDING:** one self-contained Windows bundle + qualified DLL/model/WebView2 requirements + independent clean account test.
- G3 **PENDING:** useful multilingual source discovery; current Commons coverage is narrow; evaluate with a frozen query corpus.
- G4 **PENDING:** VIDEO-100 (100,000,000 bytes) instead of current 8,388,608-byte cap, with +1-byte rejection and cleanup tests.
- G5 **PENDING:** clean Linux parity and explicit user-authorized Autopilot.

Release admission must measure cold open → validated query → useful results → verified file, zero manual prerequisites, privacy test outcomes and crash/cleanup; do not invent rates, costs or time-to-fix.

## Sources

- Prior consumer decision: `decisions/PRODUCT_GATE_01.md`.
- Candidate: `crates/pulqva-intent-server/src/windows_isolation.rs`.
- Model interface: `crates/pulqva-intent-http/src/lib.rs`, `apps/pulqva-desktop/src-tauri/src/interpret_local.rs`.
- Accepted workflow: https://github.com/yasha1971-coder/PULQVA/actions/runs/41596875127
- Microsoft: https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer
- Microsoft: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects

Final status: **ADR DONE**; Windows AppContainer real IPC/egress **NOT TESTED**; next one action **WIN-ISOLATION-02-PROBE**. No merge/release, no changes to kernel, Tor, downloader, GGUF or accepted Windows bundle.
