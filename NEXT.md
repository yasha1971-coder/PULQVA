# NEXT

## CURRENT: T068 engineering-loop recovery — preflight and safe step evidence

PR73 / feat/T068-live-tor-e2e; main untouched; T068 remains ACTIVE.
Entry head 1680dc85f0b2afe4e8b0f34bbdd6ba45065514c6 has 11/12 successful
workflows. rust-check36322839033 and native discovery-contract-check36322839013
passed. Public run36322839116 failed on both OSes: Ubuntu job108629865617 returned
CandidateRejected(UrlQueryOrFragment); Windows job108629865710 returned
Readiness(Timeout) before external_search. Logs were already read and summarized
in checkpoint5856546895; do not fetch them again without a new question.
Global verified anchor stays fe1fb03dd88e9dea01a4d70dd69082cb1ba8b4c0.

ONE implementation outcome: improve the existing discovery CI feedback loop.
The existing native workflow now requires Python evidence-control tests and the
same locked core/discovery contract command BEFORE Arti compilation/live search.
A prior failing step stops those later stages; no continue-on-error or retry.
The positive live criterion and its receipt upload remain unchanged.

The finalizer consumes GitHub step OUTCOMES, checks actual git HEAD against the
expected source SHA, and writes separate code/live verdicts plus allowlisted step
statuses to stage-status.json. Upload runs on failure when that file was produced.
Unknown/missing/stale/skipped/cancelled evidence cannot pass. No raw log/context
outputs, URLs, requests, titles, bodies or provider identifiers are persisted.
Existing safe protocol categories remain in the harness log. This artifact names
the failing workflow stage; it is NOT protocol-level diagnostics, cross-job CI
aggregation, attestation or file E2E evidence. Missing artifacts remain incomplete.
Code PASS does not erase live FAIL; a successfully uploaded diagnostic does not
turn the original failed job green. Sibling jobs still require direct inspection.

24 standard-library Python tests passed LOCALLY, including a real CLI invocation
in a temporary Git repository with spaces in its path. Workflow prerequisite/order
and evidence guards have narrow regression tests. YAML parsing, Bash syntax and
context placement were checked separately. An invalid runner context placement
was caught and corrected before publication, not by another failed CI run.
Native Windows/Linux execution of this NEW workflow is still PENDING. No local
Rust compiler/test or public network request is claimed. Exact publication/run IDs
belong in the PR checkpoint. tools/ci/README.md records scope and source review.

ONE NEXT ACTION: inspect this change's exact-head guard/contracts results and
per-OS stage-status artifacts, including FAILED jobs. Check source/checkout SHA,
run/attempt and actual child outcomes. Do not expect this process-only change to
fix Linux UrlQueryOrFragment or Windows readiness. Once this control is verified,
resume the now-localized URL issue with a provider-contract-derived reproducer;
Windows readiness stays an independent recovery. No blind rebuild or graph repair.

No product Rust, parser acceptance, provider, Tor/TLS policy, timeout amount,
Cargo graph, kernel, permission or merge-gate changes. Public operational tests
remain strict, not relabeled inconclusive or successful. No new canary schedule,
Chutney/Shadow or automatic GitHub collector was introduced. C1 real validated
choices, C2 same-coordinator selected file/bytes/digest, actual in-flight Tor loss
and positive Windows/Linux E2E remain open. T066 ownership/races, confinement,
SDK-content, YouTube/AI/UI and clean-machine packaging gates remain unresolved.
