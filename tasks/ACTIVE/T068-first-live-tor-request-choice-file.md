# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A/B0 and controlled B verified; C1 positive discovery is NOT verified.
PR73 / feat/T068-live-tor-e2e. Verified anchor fe1fb03; main unchanged.

## Parent outcome — NOT complete
Natural-language request -> at least two real externally obtained candidates ->
explicit selection -> existing Tor-gated retrieval -> actual verified local file.
Use the same T067 coordinator, not a parallel demonstration or canned locator.

## Acceptance retained
- No account/API key or frontend external network. Tor default, remote DNS, fail closed.
- Bounded time/output, owned output and objectively verified file bytes/digest.
- Selection belongs to presented results; failures do not produce success receipts.
- Positive native Windows/Linux evidence is distinct from fail-closed evidence.
- Local correctness tests are not proof of external availability or completed E2E.

## Existing evidence
Native B run 36314712742 passed controlled SOCKS/TLS/response tests on both OSes.
C1 parent 82bb76a: 11/12 workflows successful; live run 36317718659 failed.
Linux 108615474190 passed readiness and then failed search; Windows 108615474292
failed readiness. Logs discarded useful source categories, so cause was unknown.
Original fixed-public countdown query and >=2 choices acceptance remain unchanged.

## Current atomic recovery
Safe Readiness / Network / numeric HTTP / response-validation error categories
survive through CommonsSearch.last_error and are reported by the existing harness.
No raw source-message retention/logging, generic override or diagnostic request.
Three added Rust tests cover coordinator preservation/reset, invalid-input replacement
and readiness redaction; existing local TLS cases require precise HTTP status and
network category. This is SOURCE implemented; first native tests remain pending.
Exact baseline/new blob checks ran locally; no local rustc/Cargo is installed.

## Limits
ConnectOrTls is intentionally not an asserted circuit/TLS diagnosis. API errors and
warnings still share RemoteRejected. Setup/cancellation/liveness can remain Transport.
Raw source chains are not exposed. No retry/policy/version/canary-gate change here.
Temporary candidate generators remain retired; existing Cargo.lock stays exact.

## ONE next action
Inspect the new exact-head contract tests and live discovery run. On live failure,
read the category rather than infer cause from elapsed time or generic SearchFailed.
Repair only the observed failure next; do not count a diagnostic failure as C1 PASS.
C2 selected-file integration, same-flow Tor unavailable, actual in-flight Arti loss,
full native positive E2E, T066 limitations, confinement, SDK-content and packaging
remain open. Sources and detailed boundaries: NEXT.md and
 decisions/T068-C1-safe-diagnostics.md. No partial E2E merge.
