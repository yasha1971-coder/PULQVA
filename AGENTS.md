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
