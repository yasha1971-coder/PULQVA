# ADR-0010 — Recoverable project state and retained evidence

Status: Accepted
Date: 2026-10-03

## Problem
Kernel exists, but main and an unfinished branch have different NEXT files.
A formal continuity PASS did not detect stale state. Two decisions used ADR-0007.
The accepted model artifact has a seven-day Actions expiry. Chat is not a backup.

## Decision
START_HERE.md routes a newcomer to one active branch/PR. main state describes
merged code only; it must expose the development route without pretending that
unmerged work has shipped. Read live refs before writing. Never restore over newer work.

recovery/INDEX.json seals State, NEXT, kernel, the ADR inventory and small accepted
evidence. Product-tree identities bind the continuation record to source. A product
change requires an explicit new pending state and updated index in the same commit;
passing this guard never means all product checks passed. The global verified anchor
and narrowly accepted G1/model scopes remain separate.

Existing continuity CI additionally runs a no-network recovery validator and negative
fixture matrix. It exports the fetched Git refs/history into a bundle, restores that
bundle into a new directory, runs git fsck and checks the continuation record there.
It must not silently fetch missing objects during restore. Outputs are source backups,
not packages or model E2E. Original model receipt bytes are archived without renaming
old evidence into the new matrix protocol. Public-only data; no user query logging.

A repository + independently stored, verified backup is the recovery unit. Actions
retention, a single remote and a comment URL are not long-term storage. Copy backups
outside Actions. No background automation is enabled by this decision.

## Continuation and acceptance
Read Kernel -> State/NEXT -> evidence/ADR index -> current PR HEAD/checks. If sources
conflict, stop and reconcile; do not guess or rerun accepted expensive tests. A fresh
agent must identify branch, G1 scope, G2 next action, blockers and cancelled autonomy
without chat access. At each future milestone retain goal, dated primary-source review,
decision, implementation, tests and post-test comparison in a committed record. Do not
turn a successful test of one component into release acceptance.

## Limits
Hashes detect changes relative to the recorded envelope, not dishonesty by a writer
who changes both. This does not protect a lost account plus every lost backup; nor does
it mirror all GitHub metadata, external dependencies, LFS, model or runtime binaries.
A cold source restore is not a reproducible release build. No absolute-loss guarantee.

## Primary-source review, 2026-10-03
Released Git bundle/mirror facilities were chosen over a new orchestration service.
GitHub distinguishes Git history, platform metadata and external backup storage;
Actions artifacts expire. Same checks apply again after the restore drill.
https://docs.github.com/en/repositories/archiving-a-github-repository/backing-up-a-repository
https://git-scm.com/docs/git-bundle
https://docs.github.com/en/actions/how-tos/manage-workflow-runs/remove-workflow-artifacts
