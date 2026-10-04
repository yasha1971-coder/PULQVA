# PULQVA — start or resume here

This is a development recovery entry, NOT a ready application or release.
No chat history is needed to identify the next action.

**Active development:** `feat/T069-intent-contract`, PR **#80**.
The parent branch is `feat/T068-native-windows-tor`. `main` is the merged baseline,
not the current unfinished work queue. Never overwrite a newer branch with a snapshot.

1. Read this branch's `AGENTS.md` and the three `kernel/` contracts.
2. Read `PROJECT_STATE.json` and `NEXT.md` at the SAME current commit.
3. Read `recovery/INDEX.json` and `decisions/INDEX.json`. Run
   `python3 scripts/continuity_guard.py` and `python3 scripts/recovery_guard.py`
   from a full clone. Missing history/evidence is a blocker, not permission to guess.
4. Check live PR #80 HEAD and checks before any write. PR comments may contain a newer
   pending handoff; reconcile it with the sealed record before acting. On conflict stop.

## Current accepted scope

T069-G1 is accepted only as a deterministic Linux interpretation-to-choice adapter.
The existing model attempt 9 passed five fixed-public semantic/policy cases.
Its original ZIP is committed in `recovery/evidence/t069f-attempt9.zip`, with SHA-256
and member digests in the index. It is NOT retroactively a compliant matrix_receipt.
G1's accepted source/checkout/test observation is in State and its source checkpoint.

## Only next product action

**T069-G2:** add executable matrix_receipt capture/verifier to the EXISTING
real_server_smoke harness with deterministic tests. No new expensive model/network
run until evidence capture and isolation are ready. Then continue the SAME chain:
local intent -> real Tor search -> choices -> explicit selection -> verified saved file.
Do not repeat G1, improve the prompt without a cause, or create another downloader.

## Open boundaries

Full model/Tor/choice/file E2E, Commons/Arti failures, model service CORS/auth,
Windows model, Autopilot consent/ranking, Tor-loss/process egress/FS races,
UI integration, packages and clean-machine tests remain open. State lists details.
The global verified anchor is not advanced by this recovery work.

## Authority

The owner explicitly requested continued bounded work on 2026-10-04 (PR80 checkpoint
5981185094). `PROJECT_STATE.execution_authorization` records the limits. Scheduled
continuations are permitted, at most hourly, one atomic mutating task per cycle;
this does NOT grant a legacy live launch, merge, release, deployment or paid action.
Before each write read live HEAD/checkpoint and the current owner/scheduler state.
A later stop/revocation wins over an old bundle. **Offline restoration never starts
or re-enables a scheduler.** A recorded authorization is not proof of scheduler health.
Stop on access/approval denial; never bypass it. Reports only on request.

## Offline recovery

Use the independently retained PULQVA.bundle and restore_receipt.json from a passed
continuity backup. Verify its SHA-256, clone the snapshot branch named in that receipt,
run the recovery validator and read these files. No remote is needed for that drill.
Bundle covers fetched Git refs/history and committed evidence, NOT all GitHub metadata,
external binaries or reproducible build dependencies. Actions alone is not a backup.
Keep an external copy. The contract survives chat loss; no storage system is infallible.
