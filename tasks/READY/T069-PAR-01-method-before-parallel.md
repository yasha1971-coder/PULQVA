# T069-PAR-01 — independent workstreams, method first

Owner task: GitHub issue #86, 2026-10-07. Parent integration route is PR80;
existing dependency/desktop correction is PR85. This card does not start agents,
activate a scheduler, merge source, or make an application-ready claim.

## Allocation

- A: retained-commons profile in existing media workflow/capture/readback tests.
- B: OBSERVE existing PR85 at its LIVE head; do not repeat its FFmpeg migration
  or desktop Listener expectation correction.
- C: missing local-intent/caller/explicit-selection contracts in pulqva-core;
  preserve the existing public interface and model policy.

Issue #86 defines exact ownership, inputs and acceptance. Workers must claim a lane
with session/owner, branch, exact base/HEAD, input digests, touched paths and checkpoint.
Use separate worktrees/clones, build/cache/HOME/temp/Tor state/output namespaces.
Worktree separation alone does not establish resource or network isolation.
At most three independently claimed lanes; one atomic change per worker invocation.
Shared State/NEXT/AGENTS/recovery/Cargo.lock/export changes have one integrator.
A shared dependency change invalidates the affected downstream admission. Unrelated
isolated work may proceed, but a failed lane and its dependants cannot be accepted.
Kernel/privacy failures block all affected lanes. Release acceptance stays serial.

## Comparison acceptance

Before the main runtime, predeclare and measure comparison arms in the existing
matrix manifest's `comparison` object (schema 1). `subject_sha256` identifies the
explicit treatment. All 13 COMPARISON_CONTROLS and every additional control must
have measured SHA256 identities and match. Record control evidence references.
No missing/unknown condition, caveat or absent arm can authorize comparison.
Each declared probe belongs to exactly one arm; no unbound or foreign probes.
Functional acceptance on different OSes is separate, not a matched speed ranking.

MatrixRecorder now rejects invalid declared comparisons at construction, and verify
checks the frozen comparison binding. Collectors remain responsible for measuring
actual conditions and final identities. Merely hashing a label or claiming isolation
is not measurement. Historical single-object manifests cannot acquire comparative
acceptance; their original bytes and scope stay unchanged. A comparison must not
omit its declaration to avoid this gate. The codec cannot infer a hidden comparison
or authenticate dishonest metadata; collector-specific enforcement still needs tests.

On mismatch: INVALID -> close/preserve evidence -> correct method/environment in a
separate task -> fresh generation -> repeat affected comparative cells only. Never
repair or change conditions inside a generation. Uncontrollable live Tor variation
cannot be explained away into a matched performance result.

## Validation and handoff

Pre/post local review 2026-10-07: Git worktree documentation, GitHub matrix/needs/
concurrency guidance and NIST comparative-design guidance. Established mechanisms
selected; no new agent platform, model, dependency or runtime launcher.

- https://git-scm.com/docs/git-worktree
- https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/run-job-variations
- https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency
- https://www.itl.nist.gov/div898/handbook/pri/section3/pri33.htm

Local method pilot: 39/39 assertions passed (17 existing codec contracts + 22 new
comparison contracts). Separate manifest, matrix_receipt and raw outcomes are saved;
new tests reject every required unequal control, unknown/missing conditions, invalid
arms/coverage and post-run relabelling. Synthetic Python scope only, not native/CI,
actual concurrent agents or full product evidence. Existing rust-check now places
this pilot before its native work; other workflow admission is not globally rewired.

ONE NEXT ACTION: observe this method change's hosted contract result at its exact
source before integration. Then claim A/C independently while B observes its already
published change. Existing product queue/evidence are not marked complete by this card.
