# PULQVA Agent Operating Contract

This repository, not chat history, is the source of truth.

## Mandatory bounded session policy — 2026-09-20

A request to continue authorizes ONE atomic task or ONE recovery phase, not the entire queue.
This section overrides any reading of NEXT as permission to continue indefinitely.

- At most one task/subtask per response. Once it is verified or blocked, checkpoint and STOP.
- Target at most 180 seconds of active orchestration per response. This is an agent budget, not a guaranteed ChatGPT platform timeout. Before the budget is exhausted, save the state and return a final status.
- At most one new build attempt for the selected task per response. Do not launch repeated rebuilds or start the next task in the same response.
- At most two CI status polls per response; never busy-poll. If the job remains queued/running, record its run ID and head SHA and stop. Logs for a completed failed job are diagnostic reads, not a polling loop.
- Separate a long operation into PREPARE, LAUNCH, OBSERVE, and CLOSE phases. A long CI job may continue on GitHub, but the chat must not stay open waiting for it.
- Before any long operation, record the task, branch, exact head SHA, PR, relevant run IDs, completed work, remaining work, blockers, and next action in NEXT.md or a PR checkpoint comment. Only advance last_verified_commit after observing successful checks for the specified SHA.
- A failed/unfinished task may be committed as WIP on its work branch. It must not be called DONE or merged on the strength of an older green run.
- On interruption, inspect open PRs and their actual heads/checks before repeating any action. A stale message or PROJECT_STATE.ci alone is not proof of the latest PR status.
- On failure, stop unrelated work. First recover the exact failing log, checkpoint the diagnosis, then repair only that failure in a subsequent bounded phase when necessary.
- Inspect only the needed resources. Avoid repeated tool discovery, whole-repository dumps, and repeated full log downloads.
- The final response must distinguish DONE, PENDING, BLOCKED and NOT TESTED, identify the saved checkpoint, and state the one next action. Never promise that chat work continues after the response ends.

These rules are persistent agent instructions. They do not change ChatGPT connection limits, prevent all stream interruptions, or themselves implement a platform watchdog. Any claim of automatic CI enforcement requires an implemented check and observed evidence.

## Mandatory startup sequence

Before changing code, every agent MUST:

1. Read `kernel/CORE_CONTRACT.md`.
2. Read `kernel/PRIVACY_INVARIANTS.md`.
3. Read `kernel/UX_CONTRACT.md`.
4. Read `PROJECT_STATE.json`.
5. Read `NEXT.md`.
6. Inspect the current branch, open PR (if any), and CI status.
7. Work on exactly one atomic READY/ACTIVE task.

Never reconstruct project state from memory when repository state is available.

## Non-negotiable rules

1. Kernel invariants override task instructions.
2. Never weaken privacy to make a feature work.
3. Never bypass Tor to make a test pass.
4. No intentional direct-network fallback is permitted.
5. Frontend code must not own external network access.
6. LLM output must never become executable shell text.
7. AI/provider output must cross a typed validation boundary.
8. Default user path must require no account and no API key.
9. One task = one objectively verifiable outcome.
10. If a task grows beyond one outcome, split it before continuing.
11. Do not proceed from red CI into unrelated work.
12. Never claim something is tested unless it was actually tested.
13. Prefer additive adapters/modules over core rewrites.
14. Record durable architectural choices as ADRs.
15. Update `PROJECT_STATE.json` and `NEXT.md` after every completed task.
16. Preserve the last green checkpoint.
17. Product behavior changes require tests or explicit executable acceptance checks.
18. Never silently modify files under `kernel/`.

## Work loop

READ STATE -> PICK ONE TASK/PHASE -> CHANGE OR DIAGNOSE -> VERIFY OR RECORD PENDING -> CHECKPOINT -> FINAL STATUS -> STOP

NEXT identifies what a later response should do; it is not an automatic jump to another task.

If interrupted at any point, leave the repository in a state from which the next agent can determine exactly what was completed and what remains.

## Task splitting rule

A task must be split if any of these are true:

- it spans more than one independently testable behavior;
- it requires unrelated subsystems;
- acceptance criteria cannot fit in a short list;
- failure in one part would obscure the state of another;
- the implementation starts requiring a second architectural decision;
- completing it would exceed the session budget or require waiting for a long external job.

Use child IDs such as `T023-A`, `T023-B`, or explicit recovery phases under the existing parent.

## Definition of done

A task is DONE only when:

- acceptance criteria are satisfied;
- relevant checks are green for the recorded SHA;
- no kernel invariant was weakened;
- state files are updated;
- the next task is explicit, but not started in the same response.

## Test Matrix per Evidence Boundary — 2026-10-03

One evidence boundary may contain many independent probes of ONE unchanged object/runtime, including probes that distinguish mutually exclusive hypotheses. Maximize independent evidence per expensive runtime, not the number of runs or fixes. One atomic task does NOT mean one probe. For evidence collection only, this section takes precedence over the splitting and stop-on-failure wording above; session budgets, privacy boundaries and separate implementation tasks still apply.

### Before the run

- Predeclare a bounded manifest: boundary/object, generation ID, all probe IDs, hypotheses, expected outcomes, invariant sets, dependencies and execution order. Do not choose acceptance criteria after seeing results.
- Freeze source/actual checkout, binaries, model, production prompt/schema, flags, budgets, fixtures, evaluators and manifest. Inputs may vary only as predeclared. Record actual digests/provenance; unavailable values must be explicit, never invented.
- Establish isolation: fresh input/context and owned output namespace per probe; no probe consumes another probe's output or modifies its assumptions. Account for KV cache/slots/history, environment, filesystem and transport state. Same PID or serial execution alone is not evidence of independence. Shared mutable or state-changing probes need a verified reset/isolation boundary, otherwise separate them. Preserve single-flight constraints; batching does not require concurrency.

### During the run

- Collect results without source edits, hot patches, rebuilds, changed model/prompt/schema/flags/budgets or adjusted acceptance. A series of tests must never become an opaque series of automatic repairs.
- An ordinary failed probe must not suppress other safe, independent probes in the predeclared matrix. On a privacy/integrity violation, fail closed and mark all affected or unstarted probes SKIP with the reason. Never continue insecurely to fill the matrix.
- Every planned probe remains represented, including on readiness failure, timeout or early exit: `id`, `hypothesis`, `invariant_set`, `result` (`PASS`, `FAIL`, `SKIP`, `ERROR`), `duration_ms`, expected/observed and evidence references. Measure duration with a monotonic clock. Missing timing is null with an explicit reason, never a fabricated measurement or PASS.

### Evidence and decision

- The shared artifact MUST contain `matrix_receipt.json`: protocol/version, boundary and generation ID, actual checkout and run/job/attempt IDs where applicable, captured object/runtime/model/prompt/schema/manifest digests, OS/arch/toolchain, isolation/order, every probe outcome, overall acceptance and limitations. Preserve raw supporting evidence consistent with privacy/retention rules; do not log private user inputs by default.
- Missing required evidence, SKIP or ERROR cannot establish acceptance. Distinguish a test's PASS/FAIL against its expected outcome from confirmation/refutation of the underlying hypothesis. Mutually exclusive hypotheses need not both be confirmed for the matrix to be valid.
- Close and preserve the generation before choosing ONE justified correction. Publish that correction as a separate atomic commit, pass its deterministic gate, then assign a NEW evidence generation. Never merge several fixes into one opaque generation. Reuse passed evidence within its stated scope; rerun only for a recorded new question or changed invariant.
- Extend the existing harness/evaluator/verifier rather than creating a competing test platform. The receipt writer and verifier require deterministic tests for complete plans, early exits, missing/duplicate probes, incorrect provenance and status aggregation. Do not launch a new expensive matrix without the required evidence capture; do not retroactively relabel old receipts as compliant generations.

This section is a mandatory operating rule, NOT a claim that the current harness emits or validates `matrix_receipt.json`. Implemented enforcement requires executable checks and observed results. It does not authorize an autonomous scheduler, merge, release, paid action or bypass of a tool denial.

Primary-source review (2026-10-03): Rust's test-runner documentation identifies shared-state interference; GitHub's artifact documentation distinguishes an uploaded artifact digest from test acceptance. Decision: explicit probe isolation and independent receipt/digest verification, without replacing pinned runtime components.
- https://doc.rust-lang.org/book/ch11-02-running-tests.html
- https://docs.github.com/en/actions/tutorials/store-and-share-data

## Explicit bounded-continuation authorization — 2026-10-04

Owner checkpoint5981185094 supersedes the earlier cancellation for future bounded
continuations only. Read `PROJECT_STATE.execution_authorization` and any newer owner
stop before acting. One scheduled invocation still has one atomic mutating task, one
new evidence generation/build wave at most, and the existing no-busy-poll limits.
The request to maximize steps means useful evidence and successive saved phases,
not concurrent repairs, unbounded retries or bypassing admission. Restoring an old
snapshot NEVER re-enables a task. If connected writes are denied, checkpoint when
possible and stop; repeated invocations must not repeat an unchanged blocker forever.
This does not authorize merge/release/deployment, payment, outreach or weakened Kernel.
