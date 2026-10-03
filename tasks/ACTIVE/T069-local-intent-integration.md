# T069 — local intent integration

Status: ACTIVE, parent task incomplete. PR80 / feat/T069-intent-contract.
Goal: validated local intent -> real Tor search -> choices -> explicit selection -> verified file.
This task card reconciles the current PR queue with the existing continuity guard; it does
not move T068/T066 blockers to DONE or change the global verified anchor.

## G1 — scoped acceptance recorded 2026-10-03

Implementation b005097ee305c9e522a838f2b5e0f5241c23396a.
Original native run37128225491/job111217831711, actual synthetic checkout
3f856e5c7d83bd10182ad4b17c3b0f15bc888064; tree25161263beb8d692ed149a70a4cff3bf5ad030fd.
Observed five public adapter tests PASS,17 core unit tests PASS.
Reject-before-search; exact query/candidate order; Ask/Autopilot retained as data;
no automatic retrieval; original search errors and TooFewChoices preserved.
The log was read, not rerun. This does not establish live E2E or runtime isolation.

## G2 — next bounded subtask, not implemented

Add matrix_receipt.json capture/verification to the existing real_server_smoke harness,
with deterministic complete-plan, early-exit, duplicate/missing-probe, provenance and
aggregation tests. Keep old receipts, model/prompt/schema/flags and production boundaries.
Do not launch a new expensive runtime before capture and isolation requirements are met.
See NEXT.md and AGENTS.md for the acceptance contract and one next action.

## Not accepted by either subtask

Real model-to-Commons caller; user authorization/ranking; full selected-file trace;
local CORS/auth; Windows model; Commons/Arti failures; Tor-loss/egress/FS confinement;
UI, packages and clean-machine acceptance. No merge/release/autonomous schedule.
