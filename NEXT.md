# NEXT

## CURRENT: T068-C1 candidate-rejection classification recovery

PR73 / feat/T068-live-tor-e2e; main untouched; task T068 remains ACTIVE.
Entry head 843bfead07632c72e9360024bf9c41e141dd32c6: all 12 associated workflows
completed, 11 success, 1 failure. rust-check 36320828391 and native Windows/Linux
discovery-contract-check 36320828421 passed. The export repair is VERIFIED.
The global fully green anchor remains fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0.
Live run 36320828608: Linux job108624201645 reached response validation and returned
UntrustedCandidate; Windows job108624201776 returned Readiness(Timeout) before
external_search. Logs already read; checkpoints 5856171820 and 5856225229 contain
exact markers. Do not re-download them or conflate these failures with E0432/403.

ONE outcome in this source recovery: expose the exact first failed candidate
predicate through the EXISTING search -> last_error -> fixed-public harness path.
CandidateRejected(CandidateFailure) carries only fixed enum variants: page identity,
namespace/title checks, duplicates, URL length/characters/authority/path/encoding,
digest length/encoding and core construction. CandidateFailure is exported together
with its implementation; no repeat of the missing-export defect.

There is ONE underlying parser. Its standalone public parse_response wrapper keeps
the prior UntrustedCandidate error contract, while CommonsSearch invokes the same
classified implementation directly. No second parse, network request, fallback,
response recording or logging path is added. All original test modules and request
plan are byte-identical to the parent. Existing candidate acceptance predicates,
case-sensitive extension, ASCII requirement, MIME/size/hash limits, ordering and
fail-whole-response behavior are retained. No interpretation of the unknown live
predicate is claimed; this is diagnostic refinement, not an operational fix.

Four new network-free Rust tests cover 21 reachable rejection categories through
the real coordinator, stale-result clearing, safe formatting, reset after success,
positive metadata/hash normalization and a 672-input differential URL corpus
against the previous predicate copied only into cfg(test). CoreContract remains a
defensive category; current earlier checks imply nonempty core fields. Native
compilation/test execution is PENDING, not claimed by source inspection.

Local evidence: 23503-byte baseline Git hash matched the actual repository blob;
asserted edits, complete old-test and request-plan preservation, UTF-8/JSON byte
round-trip, independent resulting source hashes. Uploaded commons.rs: 25849 bytes,
07ffeb80dff4f561da1a497be4c136b13ea244a8; tests: 6825 bytes,
8bdc5cceb04d02e15c77dc180cc43f732e3dea07; lib.rs: 490 bytes,
537b2557c62e0a1bbc51f9259c9e4a1fad6cb950. No local Cargo/rustc is installed.
Runtime public-raw retrieval was unavailable (DNS); connector reads plus verified
byte reconstruction were used, not a claim that local network tests ran.

ONE NEXT ACTION: observe the newly published exact head recorded in PR73, starting
with rust-check/native discovery-contract-check. If those pass, inspect the existing
live run's fixed category to identify the actual candidate predicate; do not skip
unsafe candidates, relax validation, change dependencies/provider, lengthen Tor
deadlines, or retry until lucky. Windows readiness remains a separate open blocker.
At most one source-triggered attempt this phase; no busy polling or next task jump.

Entry/exit review 2026-09-27: official MediaWiki API:Imageinfo and API:Errors_and_warnings
https://www.mediawiki.org/wiki/API:Imageinfo
https://www.mediawiki.org/wiki/API:Errors_and_warnings
These describe untrusted metadata/API semantics, not the actual rejected live value.
No kernel, Tor/TLS client, timeout, Cargo graph, workflow permissions or merge-gate
change. Canary/tri-state/SLO/Chutney/Shadow remain separate, not implemented here.
C1 positive external choices, C2 same-coordinator selected file/bytes/digest, actual
in-flight Tor loss and positive Windows/Linux E2E remain UNVERIFIED. T066 race/
retention, descendant confinement, SDK-content, YouTube/AI/UI/packaging gates remain.
