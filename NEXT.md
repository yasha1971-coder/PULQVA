# NEXT

## CURRENT: T068-C1 diagnostic recovery — safe categories across the existing path

PR73 / feat/T068-live-tor-e2e; main untouched. Last fully verified anchor:
fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0. Do not mark T068 complete.
Parent 82bb76ac8ce9f9c003ef6cf727227b964c5c52e2: 11/12 associated workflows
successful; live discovery 36317718659 failed on both OSes. Linux passed readiness
then returned SearchFailed; Windows stopped during readiness. Exact logs were
already inspected in PR checkpoint 5855788840. Neither cause was established.

ONE recovery outcome implemented in source: preserve the safe failure category.
CommonsSearch.last_error retains the latest invoked discover/search failure before
the existing coordinator maps it to JourneyError. Every invoked search clears
old choices; successful searches clear the error. An input rejected before the
coordinator calls search does not update that snapshot; do not treat it as a trace.
The HTTPS adapter now classifies TorReadinessError variants, reqwest timeout /
connect-or-TLS / body / other, observed numeric HTTP status, and content-type /
content-encoding rejection. Existing parser categories distinguish API rejection,
invalid response, untrusted candidates, too few results and excessive bytes.
No raw source chain, query, URL, response body, header, certificate or identifier
is retained in the diagnostic value. Production library does not log it; only the
fixed-public-fixture harness prints stage plus safe category on failure.

This is a minimal taxonomy, not invented precision: ConnectOrTls cannot identify
a circuit/exit/TLS root cause. API errors and warnings remain grouped as
RemoteRejected; setup/cancellation/liveness can still return Transport. A request
budget expiration is classified as Network(Timeout), including the existing
supervision expiry exit; timeout amounts and success criteria are unchanged.

Added 3 Rust test functions: 10-case coordinator error-preservation/reset matrix,
local-validation replacement without fetch, and readiness redaction categories.
Existing controlled TLS/SOCKS tests now assert the corresponding categories and
HTTP 302/403/429 remain numeric. All original tests and network refusal conditions
remain. New tests are NOT claimed passing before native execution.

Local checks: exact baseline Git hashes, asserted source edits, UTF-8/JSON byte
round-trip, independent new blob hashes. Query/parser/media validation and full
route/client policy blocks compare byte-for-byte unchanged. No local rustc/Cargo;
no local Rust test/compilation or live Tor request occurred. Four uploaded source
blob IDs match their locally computed hashes. Publication SHA/run IDs: PR73.

ONE NEXT ACTION: inspect the new exact-head deterministic discovery-contract-check
AND live commons-tor-discovery-check; read the bounded PULQVA_COMMONS_FAILURE line
on each failed OS. Distinguish code-regression evidence, operational failure and
unresolved attribution; do not guess 403, change provider, relax TLS or add blind
retries. A diagnostic live failure is not proof of a code defect or E2E success.
Use the observed category for the next targeted recovery. No second generator.

Test-pyramid discussion retained: code correctness and live operational availability
are separate evidence. This commit does NOT change required checks, add a schedule,
install Chutney/Shadow or implement tri-state canary/SLO accounting. Such changes
must not relabel local tests as C1 live discovery or an unfinished E2E as complete.
C2 selected-file retrieval, actual Tor-loss integration, positive native full E2E,
T066 race/retention, descendant confinement, SDK-content and packaging remain open.

Current primary-source review: decisions/T068-C1-safe-diagnostics.md.
