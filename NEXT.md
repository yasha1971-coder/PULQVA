# NEXT

## CURRENT: T068-C1 Commons original-URL provenance compatibility

PR73 / feat/T068-live-tor-e2e; main unchanged; T068 ACTIVE.
Parent 5dea95d7c03f6a2c10b25b261f734e63ab45a924: all 12 associated
workflows completed, 11 success. rust-check36325638292 and native
contract-check36325638342 passed. Native preflight/evidence is VERIFIED on both
OSes: 24 Python tests, locked contracts, preserved code=PASS/live=FAIL receipts
with independently matched artifact hashes (PR73 OBSERVE/CLOSE checkpoint).
Do not rebuild generic evidence infrastructure or re-download old logs/artifacts.
Global verified anchor stays fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0.

Existing live run36325638211: Ubuntu108637766074 rejected
CandidateRejected(UrlQueryOrFragment); Windows108637766318 returned
Readiness(Timeout) before search. The actual suppressed URL was not retained.

Primary-source research 2026-09-27 found a concrete provider incompatibility:
MediaWiki File::getUrl calls appendRequestProvenance, which conditionally adds
utm_source, utm_campaign, utm_content. ApiQueryImageInfo::getInfo explicitly sets
generator=imageinfo and format=original. This real supported URL form is rejected
by our blanket query ban. This does not prove the exact earlier suppressed suffix.
https://doc.wikimedia.org/mediawiki-core/master/php/File_8php_source.html
https://github.com/wikimedia/mediawiki/blob/master/includes/Api/ApiQueryImageInfo.php
ApiQueryImageInfo source blob observed: b95c00ba26314d281cd11072b27119bca918fbe8.

ONE correction: recognize ONLY the complete, unique public triple
utm_source=commons.wikimedia.org, utm_campaign=imageinfo, utm_content=original,
in any order; strip it, then apply the UNCHANGED strict media URL validator.
Original-input ASCII/control/2048-byte limits still apply. Arbitrary queries,
fragments, duplicate/missing keys, alternate values or encoded aliases fail closed.
Both duplicate checking and selected locators use the normalized URL. No new
network request, canned result, fallback or response logging is introduced.
MIME/size/digest/page checks, query plan and all earlier tests are unchanged.
The private helper has no public type/export or new dependency.

Six new Rust tests cover the provider-derived minimal reproducer, all six query
orders and idempotence, 24 unsupported suffix cases, 13 unsafe base addresses,
full input cap, controls/non-ASCII, post-normalization duplicates and coordinator
selection/reset without extra fetches. Fixtures are synthetic, not recorded live.
No local Rust compiler is available: compilation/test execution remains PENDING.
Local checks verify baseline bytes against Git blob07ffeb80dff4f561da1a497be4c136b13ea244a8,
asserted replacements, unchanged original validator/test modules/query plan,
UTF-8/JSON byte round-trip and independently computed new source blob hashes.

ONE NEXT ACTION: inspect the new exact-head native preflight/contracts first,
then the live result and per-OS receipts. Confirm provenance_tests actually ran.
Only a successful live run can confirm whether this addresses the observed Linux
failure; do not call the operational issue fixed from source inspection.
Windows readiness remains a separate recovery; no blind rerun or timeout increase.
Source head/run IDs and publication state belong to the latest PR73 checkpoint.

No Tor/TLS, workflow, permissions, kernel, Cargo graph or main changes.
C1 actual validated choices; C2 same-coordinator selected file/bytes/digest;
actual in-flight Tor loss and positive Windows/Linux E2E remain UNVERIFIED.
T066 cleanup/races, confinement, SDK-content, YouTube/AI/UI/packaging remain open.
