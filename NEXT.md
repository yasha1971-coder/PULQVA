# NEXT

## CURRENT: T068-C1 — first real Commons discovery acceptance LAUNCH

PR73 / feat/T068-live-tor-e2e; main unchanged. Exact parent
fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0 is 11/11 associated workflows green.
Native discovery-contract-check 36314712742 passed on Windows/Linux. Those tests
prove controlled local TLS/response behavior, not live Commons or the entire E2E.
The adopted graph and existing library executor are unchanged in this pass.

ONE atomic outcome: execute CommonsHttpsTransport -> CommonsSearch -> request_choices
against the actual Commons API through an owned pinned Arti process, requiring at
least two externally obtained validated choices. The native acceptance example
real_commons_discovery uses the fixed public full-text request `countdown`, selects
index 1 from the returned set, records bounded JSON evidence, then exercises sticky
cancellation through the same coordinator and requires stale results to be cleared.
No synthetic successful response, hard-coded media URL, alternative HTTP client,
provider expansion or dependency regeneration. This is not AI intent interpretation.

commons-tor-discovery-check compiles the actual library/harness before building the
existing pinned Arti sidecar and runs the same positive criterion on Windows/Linux.
A readiness/search failure fails the job; Windows fail-closed-only is NOT success.
The example explicitly stops/reaps the owned Arti process on returned errors and
cleans its exclusively created temporary root before writing successful evidence.
Artifacts contain public fixture choices/declared metadata only, retained 7 days;
no raw HTTPS reply, secret, keylog, downloaded media or personal request is uploaded.
Output file creation refuses overwrite. These are source properties, not a hostile
filesystem race proof or an assertion that the new native run has already passed.

ONE NEXT ACTION: inspect this exact head's commons-tor-discovery-check (both OSes)
and existing checks. Diagnose failures at readiness/search/validation first; never
replace a missing external candidate with a canned result. If green, inspect the
choice evidence and connect its actual selected locator to existing typed yt-dlp
retrieval in the SAME coordinator. Preserve the selected-file, byte/digest, actual
Tor-loss and same-flow fail-closed gates before declaring parent T068 complete.
Do not introduce another candidate graph generator or a parallel demo downloader.

Local checks: example/workflow UTF-8 and JSON byte round-trip, independently computed
Git blob hashes, YAML parsing, native matrix and read-only permission assertions.
No local Cargo/rustc is installed; Rust compilation and live requests are PENDING.
Source publication SHA and run IDs belong in PR73 checkpoint. No E2E percentage claim.

Entry/exit primary-source review, 2026-09-27:
https://www.mediawiki.org/wiki/API:Etiquette
https://www.mediawiki.org/wiki/API:Search
https://www.mediawiki.org/wiki/API:Imageinfo
Descriptive User-Agent and a serial generator query retained. No latest-version or
exhaustive security-audit claim. API metadata hashes are not downloaded-file proof.
T066 race/retention, descendant confinement, SDK-content and packaging gates remain.
