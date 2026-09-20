# PULQVA Agent Operating Contract

This repository, not chat history, is the source of truth.

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

READ STATE -> PICK ONE TASK -> CHANGE -> TEST -> GREEN -> COMMIT -> UPDATE STATE -> NEXT

If interrupted at any point, leave the repository in a state from which the next agent can determine exactly what was completed and what remains.

## Task splitting rule

A task must be split if any of these are true:

- it spans more than one independently testable behavior;
- it requires unrelated subsystems;
- acceptance criteria cannot fit in a short list;
- failure in one part would obscure the state of another;
- the implementation starts requiring a second architectural decision.

Use child IDs such as `T023-A`, `T023-B`.

## Definition of done

A task is DONE only when:

- acceptance criteria are satisfied;
- relevant checks are green;
- no kernel invariant was weakened;
- state files are updated;
- the next task is explicit.
