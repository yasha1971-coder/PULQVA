# G2C2 — positive admission and capture in the existing media path

Date: 2026-10-04. Preparation checkpoint5982628558. Source base3b212513.
Status: implemented;20 local synthetic controls PASS; hosted/native/live acceptance pending.

## Goal / acceptance
Move beyond temporary admitted=false without resuming blind Tor experiments.
One exact-source explicit media generation must use the existing Rust media fixture,
a genuine prior native gate and one immutable measured runtime. Automatic events
and the other three legacy paths remain NOT_TESTED. This is a prerequisite for the
original local-intent -> choices -> selected verified file path, not its completion.

## Applicable current primary sources / decision
Reviewed current official documentation, not an exhaustive state-of-art survey:
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/pass-job-outputs
- https://docs.github.com/en/actions/tutorials/store-and-share-data
- https://docs.python.org/3/library/subprocess.html
- https://docs.python.org/3/library/selectors.html

Released GitHub needs/outputs and same-run immutable artifacts suffice. Do not add
another launcher, external scheduling service, cross-run API client, a boolean
bypass or a new product dependency. Admission waits on actual two-OS contracts;
manual Linux contracts retain the accepted six-probe native format. The admission
validator binds that exact manifest/receipt/raw output to this run, attempt1, exact
checkout and evaluator bytes. Existing source changes invalidate prerequisites.
A newer explicit dispatch is a NEW generation, not a retry in the old generation.
Concurrency serializes dispatches for a source; it is not global duplicate suppression.

Use a prebuilt native fixture rather than cargo run inside the evidence boundary.
Prepare the pinned Arti2.6.0 and yt-dlp2026.08.19 with unchanged preparation commands.
Measure post-build executables before the manifest; remeasure afterwards. Arti's
build version is pinned but its binary is not independently authenticated by a
vendor checksum; dynamic system libraries and build reproducibility remain limited.

Python subprocess documentation warns that communicate buffers output and killing
one parent does not itself specify descendant cleanup. Use POSIX owned process group,
selectors-based bounded pipe draining, a minimal environment and a fresh owned
home/temp. No source edits, extra network probes or retries inside this adapter.
Linux-only positive capture is explicit; Windows deterministic contracts remain.
GitHub workflow setup downloads are CI preparation, not claims about product egress.

## Post-local-test review
The20-case synthetic matrix confirms valid and negative admission, stale/foreign/
failed/incomplete prerequisites, exact execution checks, pre-manifest before launch,
no inherited token/proxy, timeout/output cap, cleanup, nonzero/fail-closed output,
identity drift, occupied namespace and prep failure. Original sidecar preparation
is checked against a separate SHA derived from the old live steps; other three
whole-step baselines remain unchanged.11 related trigger contracts and11 existing
native-collector wrapper tests passed. YAML parses locally. No native local compiler.

Observed harmless child processes behaved as predicted by the selected released
Python interfaces. No transport/model/pin/budget upgrade is warranted. Hosted behavior
and the actual live outcome must still be verified after publication; no manufactured
live PASS. The20 local tests use synthetic native-output prerequisites, clearly
separate from actual Rust execution and its artifact.

## Limitations / next boundary
One live journey probe: its internal Tor retries are dependent. Fixed-public runtime
output only; private requests never enter this harness. This is not an OS egress,
hostile filesystem/process sandbox, process-group escape defense or independence
signature. Host death may prevent final receipt. A process group is not product
Tor-loss supervision. The fixed fixture checks size then deletes the media file:
its marker is not a retained content-verified file. Subsequent integration must
reuse the existing discovery verifier and the local intent/choice chain.

Next: observe exact-head hosted controls; only then one separately dispatched live
media generation with complete original artifacts. Do not leave temporary denial
as the final design, but do not mistake this new positive path for release readiness.
