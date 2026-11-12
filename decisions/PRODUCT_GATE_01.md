# PRODUCT_GATE_01 — first consumer decision

Date: 2026-11-02. Decision record only; no product code, package, model, network policy or build changes.

## Purpose and comparison

PULQVA is a desktop application intended to turn a natural-language request into real Tor-routed choices and a user-selected, locally verified file.

Compared with the ordinary browser → search → save workflow, the current prototype combines local interpretation, explicit result selection and bounded retrieval in one window. A browser/downloader configured with suitable privacy tools can also retrieve files privately; this review establishes no superiority in anonymity, speed, coverage or safety. PULQVA currently has narrower coverage and a separate manually started model server. A SHA-256 records byte identity; it does not authenticate the publisher, establish copyright permission or prove harmless content.

## Baseline and evidence

- Reviewed application source: [3bf040ea61bc010f7e0b1db88187edfe3db8c35e](https://github.com/yasha1971-coder/PULQVA/commit/3bf040ea61bc010f7e0b1db88187edfe3db8c35e).
- Branch: fix/windows-official-tor-runtime; [PR #101](https://github.com/yasha1971-coder/PULQVA/pull/101), open/draft, unmerged; actual head matched the source above when inspected.
- All 16 existing workflows for that exact source observed SUCCESS. Key runs: [Windows candidate41409015687](https://github.com/yasha1971-coder/PULQVA/actions/runs/41409015687), [desktop41409015705](https://github.com/yasha1971-coder/PULQVA/actions/runs/41409015705), [rust41409015649](https://github.com/yasha1971-coder/PULQVA/actions/runs/41409015649), [continuity41409015623](https://github.com/yasha1971-coder/PULQVA/actions/runs/41409015623). Contract-only successes do not establish live-network acceptance.
- [Artifact13125525451](https://github.com/yasha1971-coder/PULQVA/actions/runs/41409015687/artifacts/13125525451), ZIP SHA-256: `e2b97f7c38ea88cba34bd511da345375d31cb3bd72c29ad315984bf7a23a5a90`.
- Candidate EXE SHA-256: `c832691ade0ccc27d3f55f2d2996f9a39df992323e4bf0978f00c4382081c227`.
- Working copy: `C:\PULQVA-NATIVE\PULQVA-INTENT02A-TEST`; local [manifest](<C:/PULQVA-NATIVE/PULQVA-INTENT02A-TEST/MANIFEST.json>), [build receipt](<C:/PULQVA-NATIVE/PULQVA-INTENT02A-TEST/BUILD-RECEIPT.json>), [37-test frontend log](<C:/PULQVA-NATIVE/results/INTENT02A_20261102/frontend-tests.log>), [network gate](<C:/PULQVA-NATIVE/results/INTENT02A_20261102/network-gate.json>) and [packaging verification](<C:/PULQVA-NATIVE/results/INTENT02A_20261102/owner-candidate/package-verification.json>).
- [Owner testimony provenance](https://github.com/yasha1971-coder/PULQVA/pull/101#issuecomment-6581258919) and [preceding candidate closeout](https://github.com/yasha1971-coder/PULQVA/pull/101#issuecomment-6572509480). Local links require this laptop; they are not published CI artifacts.
- PROJECT_STATE.json/NEXT.md describe the independent T069 lane and older evidence. They were read and preserved; they are not substituted for current PR101 observations. The candidate manifest/checkpoint predates the following testimony; its NOT_TESTED flag means no agent-run live test at staging time.

### Owner attestation preserved without upgrading its provenance

Source: the owner's PRODUCT-GATE-01 message, received in this session. The owner reports:
- Windows GUI → Qwen → Tor Search → Select → Download on source 3bf040ea61bc010f7e0b1db88187edfe3db8c35e.
- File: `Big buck bunny 720p 5mb.webm`.
- Size: **5,709,008 bytes**.
- SHA-256: `188e86965295906e8500983a3d530390ca2c55184175873c01485670cc1ab82e`.
- The owner independently calculated the file hash with Get-FileHash.

This is a real user's reported acceptance, not a CI test or an agent-witnessed execution. No screenshot, raw command output, complete saved path, packet capture, current-run PID/socket cleanup receipt, selected index or result count was supplied with this message. The file was not re-read in this documentation phase. The report supports one successful journey; it is not a search reliability estimate. Earlier cleanup evidence on other runs does not establish cleanup for this particular run.

## Readiness by evidence boundary

PASS is limited to the stated observation; PENDING means an identified unfinished implementation/gate; NOT TESTED means the property lacks acceptance evidence here.

| Property | Status | Evidence and limit |
|---|---|---|
| Exact-source Windows build and deterministic checks | PASS | 16 workflow summaries; Windows receipt; no new build or rerun |
| Frontend guards, explicit AI→search action, session/index choice | PASS | Existing37/37 fixtures, including preserved26; fixtures are not Internet results |
| GUI→native local Qwen→Tor search→selection→new file | PASS — owner report | Above testimony on this laptop and candidate; not independently replayed |
| Saved-file independent SHA calculation | PASS — owner report | Owner Get-FileHash and reported hash/size; raw transcript not supplied |
| Existing package identity and six resource identities | PASS — staging scope | Prior ZIP/EXE/readback/ACL verification retained; USER-TEST / DEBUG-LINEAGE |
| Fully automatic, verified owned model startup | PENDING | Current GUI client expects a pre-existing development endpoint18081 |
| Standalone package with model/runtime and dependency qualification | PENDING | Model/runtime not bundled; installed WebView2 is assumed |
| Current manual run: Tor death, DNS/direct TCP/UDP, endpoint hijack, inference egress | NOT TESTED | Typed IPC and localhost bind do not establish process confinement |
| Current manual run: complete PID/descendant/socket cleanup | NOT TESTED | No run-specific cleanup receipt in the owner's message |
| VIDEO-100 actual100,000,000-byte retrieval bound | PENDING | Current prototype bound8,388,608bytes; successful5,709,008-byte file proves no larger boundary |
| Broad source/language relevance and stable search success rate | NOT TESTED | Current Commons path and successful examples are not broad-source acceptance |
| Independent clean Windows user, offline model operation, current Linux GUI parity | NOT TESTED | Developer laptop success is not clean-machine or cross-platform qualification |
| Consumer release | PENDING | Debug lineage, manual model setup and privacy/package gates remain |

## First-user obstacles, checks, changes and cost uncertainty

The ranges below describe known scope bounds and missing estimates. Person-day or currency ranges are **UNKNOWN**: no timed implementation breakdown or clean-machine measurements justify numerical estimates. This avoids turning an unmeasured plan into a promise.

| Obstacle / measured gap | Falsifiable verification | Smallest proposed change and dependencies | Cost range / uncertainty |
|---|---|---|---|
| Model startup: user must separately run Qwen at127.0.0.1:18081 | Start with no server; opening app must obtain one strict Intent and close with no owned PID/listener | Reuse existing intent-server lifecycle and intent-http/json; bind verified runtime/model identity to owned process, bounded readiness and confinement; keep keyword search | Existing modules available → Windows lifecycle/confinement changes may be necessary; new crate dependencies0 expected, unverified; duration UNKNOWN |
| Package dependency set | Copy only candidate bundle to clean ordinary Windows account; no toolchain/WSL/manual downloads; query succeeds | Manifest/licence/DLL audit, include approved existing model/runtime; explicitly resolve WebView2 prerequisite without silent installation | Current manifest has7 payloads; model inclusion adds about3GB from retained model identity, plus runtime/DLLs; final disk/RAM/download size UNKNOWN |
| Size limit8,388,608 versus target100,000,000bytes | Exact-boundary and one-byte-over streamed retrieval, cancellation and disk cleanup | Separate backend bounded-retrieval task; retain actual-byte accounting, Tor-only path and atomic finalization | No change needed for current≤8MiB user test; VIDEO-100 modification and adversarial tests required; duration UNKNOWN |
| Source coverage and relevance | Frozen public query/source corpus; report usable results, relevance and failures by source/language | Keep Commons; add another adapter only after measured missing demand and policy review | Current baseline one Commons provider; no justified new provider count, effort range or success rate |
| Clean launch and portability | Fresh Windows account/machine with no dev tools; repeat from moved bundle; record readiness/latency/RAM | Package/lifecycle work first; independent acceptance afterward; Linux separately | One owner laptop already works; independent-device coverage0 recorded here; test duration/support burden UNKNOWN |
| Privacy and cleanup | Bounded negative tests of process egress, Tor loss, foreign endpoint, cancel/crash; independent PID/port evidence | Reuse existing supervision/privacy tests; close actual gaps without exemptions, direct fallback or blanket network claims | Positive UI test is insufficient; exact Windows containment delta and time range UNKNOWN; a failed isolation test blocks autonomous model startup |

The existing [interpret_local.rs](https://github.com/yasha1971-coder/PULQVA/blob/3bf040ea61bc010f7e0b1db88187edfe3db8c35e/apps/pulqva-desktop/src-tauri/src/interpret_local.rs) uses the fixed development endpoint and typed validation. Existing [intent-server lifecycle](https://github.com/yasha1971-coder/PULQVA/blob/3bf040ea61bc010f7e0b1db88187edfe3db8c35e/crates/pulqva-intent-server/src/lib.rs) owns a Child, clears its environment, uses explicit loopback, bounded readiness/completion and Drop kill/wait. That file does not itself verify model/runtime hashes, supply the accepted CPU/context/reasoning configuration, prove descendant cleanup, authenticate an unrelated listener or confine external egress. Reuse it; do not call it production-qualified without those checks.

## Consumers and C ABI decision

**Primary consumer: desktop-user.** Evidence is one owner operating the actual Windows window and receiving a file. The immediate unmet need is opening the app without separately managing a model server.

**Possible consumer: embedded-developer.** No named external project, integration owner, language/ABI target, required functions, lifecycle contract or acceptance test was identified in the inspected repository evidence and owner request. This is a scoped absence of evidence, not a claim that such users cannot exist. Existing Rust crates and Tauri IPC already serve the proven consumer.

Market hypotheses remain unvalidated: demand beyond the owner; willingness to accept a roughly3GB model; acceptable CPU latency; desired media sources,100MB files and languages; need for an SDK. No outreach or market-size claim was made.

**Decision: DEFER C ABI.** It removes no demonstrated desktop friction and would add an ABI/versioning/ownership surface without a confirmed consumer. Reconsider only with a named external integration and executable acceptance criteria. NO CHANGE to ABI is the preferred current alternative. Also defer new sources, larger downloads and UI redesign until the model startup barrier is closed.

## Exactly one next READY task

**OWNED-LLM-START-01 — open PULQVA and obtain a validated local Intent without a manually started model server.** READY for a separately authorized bounded vertical slice; not implemented by this document.

Use the existing native Windows runtime/model and intent-server/http/json code in a fresh test candidate. Preserve accepted packages, Tor/downloader, GUI design and8MiB cap. Do not download/replace a model or introduce a second server framework.

Objective Definition of Done:
1. With no pre-existing model server and no admin rights, opening the test app starts exactly one owned verified local model process; no terminal, service installation, WSL or manual server action.
2. Record exact source/EXE/model/runtime identities and accepted fixed flags. Reject changed assets, occupied foreign endpoint, readiness deadline or child death without a successful Intent or network fallback.
3. Existing GUI interpretation of Big Buck Bunny returns a real Rust-validated Intent through the owned endpoint; retain malformed/Reject/timeout failure behavior and working keyword search.
4. Demonstrate inference external-egress denial under the actual Windows launch context, including relevant IPv4/IPv6/DNS/proxy paths; bind-only or an absence-of-observed-traffic claim is insufficient. If unavailable without changing owner security policy, mark BLOCKED with the exact dependency.
5. Close/error cleanup proves owned child/descendant identity gone and no owned listener; do not kill foreign processes. Capture readiness time, inference latency and peak memory without borrowing Linux measurements.
6. Preserve existing deterministic gates and capture one exact-source Windows candidate. If build/runtime proof does not fit the bounded session, checkpoint PENDING rather than declare this DoD complete.

Why now: it removes a confirmed manual user action on the already successful GUI path. Enlarging the download cap or adding a C ABI cannot make the natural-language path start by itself. Complete cold-machine distribution and broader privacy acceptance remain later gates, not automatically authorized follow-up work.

## Closeout

PRODUCT-GATE-01 is a documentation/evidence decision, not a consumer release or a new runtime PASS. The reviewed application remains 3bf040ea61bc010f7e0b1db88187edfe3db8c35e. This documentation-only change requests no build; any documentation commit has its own identity and does not inherit application CI as a new tested binary. Protected T069 state, contracts and working packages remain unchanged. Checkpoint is in PR101 and the existing local INTENT02A checkpoint. STOP after this gate.
