# NEXT

## T069-G2 — FFmpeg replacement source PREPARED

PR80 feat/T069-intent-contract; parent5fde8951a17f805aebb0260e609b3f615a2ed061.
Original daily release2026-09-20 is404. Candidate monthly release2026-09-30-13-08 exists; both full archives downloaded and independently match API SHA256 and exact sizes. Standard-library tar/ZIP readers hashed ffmpeg/ffprobe without execution. See recovery/evidence/ffmpeg-source-candidate.json. This is identity evidence, not compatibility, matrix or live Tor acceptance. Production pins/workflows unchanged. Historical evidence and global verified anchor unchanged.

ONE NEXT ACTION: T069-G2 FFmpeg source migration: adopt this exact monthly snapshot consistently in existing FFmpeg consumers and SOURCE_PROOF, preserve checks and candidate identities, then run one existing two-OS CI build wave. New archive bytes require fresh compatibility/remux evidence; older green results do not establish acceptance. Before implementation inspect live HEAD and current CI. Retained-file workflow remains pending behind deterministic gates. Owner merge authorization exists, but PR is draft/unmerged and CI blocked. No release/scheduler change.

## T069-G2D-A — scoped recovery checkpoint (2026-10-08)

Owner-dispatched workflow 37769355186, attempt 1, source 3ca7a85ef5de3f9108cc3213da173a6f65356c87, profile retained-commons: completed SUCCESS. Ubuntu/Windows deterministic jobs and admission PASS; Linux live retained Commons request -> 10 choices -> fixed index 1 -> selected.webm readback PASS. Artifact 11546608929 SHA256 b6dc1ca740ef4b00876d477308f033594ea0be5d27712a57df39c91fb8730b7f; retained file 2131934 bytes, SHA256 0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423. Original evidence, prerequisite/admission identity and cleanup accepted within Linux fixture scope; see PR #88 checkpoint. Prior GitHub internal-error run 37642119152 remains failed and preserved. No model intent, human UI choice, Windows live E2E, package or clean-machine acceptance. This does not supersede the independent FFmpeg task or global verified anchor.

Method rule: reject mismatched comparison conditions, verify method before costly run, freeze evidence generation, and make any correction in a separate generation. No blind reruns.

ONE NEXT ACTION for lane A: reconcile its task/INDEX and relevant integration state against live heads before selecting a new technical milestone. Do not repeat the accepted live run. Keep unrelated lane and FFmpeg work isolated.

## PR101 MODEL_AUTOSTART_01 — owner-directed product priority (2026-11-04)

Decision: [decisions/MODEL_AUTOSTART_01.md](decisions/MODEL_AUTOSTART_01.md). Windows AppContainer stdio PING/PONG PASS on actual owner PC (f389cfd); automatic GUI Qwen, pinned CLI/model package, full egress confinement and clean-machine release remain PENDING / NOT TESTED. Do not misreport production module exposure at c01dbe9 as model integration. Preserve existing manual-server prototype; no unrestricted launcher or security bypass.

ONE NEXT ACTION for PR101: MODEL-IPC-01 / phase A — run read-only `tools/windows/model_ipc_assets.ps1` against the owner's local PULQVA root and capture actual CLI/server/GGUF SHA-256 and sizes. Script committed at 69da1812153ead4905a653df5f4efecd3a6e006c; no model was executed, and no qualifying CLI binary is yet known. Only after identities are grounded, create one bounded Windows AppContainer model stdio witness. No GUI/model-package changes until real owner acceptance. The independent T069 queue above is unchanged.


## PR101 MODEL-IPC-01 — PREPARE checkpoint (2026-11-04)

Actual head 16bc9686bc2f3cdad66d77b861eebb66d4b0c1d9; existing-head rust-check 41792945529 and IPC artifact 41792945578 completed success. Owner supplied CLI/GGUF identities; see `recovery/evidence/model-ipc-01-prepare-20261104.json`. Identities are owner-reported, not independently rehashed here. Existing launcher merges stderr/stdout and does not grant model read ACLs. No witness code, new build, GUI/Tor/security change or real Qwen execution in this phase. Global last_verified_commit preserved. Local old checkout Git access returned Permission denied; authoritative source was read through GitHub.

ONE NEXT ACTION for PR101: IMPLEMENT the bounded ignored Windows model CLI witness, production typed decoding, existing exact-head CI artifact and exact owner runbook from this manifest. Freeze final public prompt/schema/hash/flags before launch; fail closed on access denial. Real owner test and egress observation remain NOT TESTED. Do not use old PING/PONG or CI compilation as Qwen evidence.

## PR101 continuity recovery — 2026-11-04

Parent e7ec429e4ab94db56295c35d438087b653d17bf1 failed continuity-guard run 41823433727, job 126552029436: protected history/policy changed; explicit transition required: PROJECT_STATE.json. Comparison with 16bc9686bc2f3cdad66d77b861eebb66d4b0c1d9 found only the added model_ipc_01 object. Restore PROJECT_STATE.json to the exact green-head blob fbed400ac04d73ed8cea2bfa2afbf8d02e87e9be; retain the PREPARE receipt and checkpoint above unchanged. Guard, historical last_verified_commit, runtime code and security policy are unchanged. CI for this recovery is PENDING at publication; this is not MODEL-IPC-01 implementation or Qwen/egress evidence.

ONE NEXT ACTION: observe continuity-guard for the recovery commit; only after success, return directly to the bounded MODEL-IPC-01 IMPLEMENT task. No new PREPARE, model launch or merge in this recovery phase.

## PR101 MODEL-IPC-01 IMPLEMENT — WIP checkpoint (2026-11-04)

Parent ff2cdd1562c78b03a7754204e21d70a727b2cc01: independently observed 17/17 successful workflows.
Added real ignored Qwen witness using existing zero-capability AppContainer spawn, separate stderr,
prelaunch SHA256/size pins, fixed production policy/schema/public request, output/deadline bounds,
strict production decoding, Job cleanup and test-only manifest/receipt. Runbook: tools/windows/MODEL_IPC_01.md.
Protected PROJECT_STATE and global verified anchor unchanged. Local Git checkout access denied;
implementation published via connected GitHub. No local cargo available on PATH, no local build attempted.
New-head compilation and deterministic checks PENDING; Qwen/cleanup/egress NOT TESTED.
This is executable implementation WIP, not another PREPARE and not acceptance.

ONE NEXT ACTION: inspect exact-head hosted compilation and deterministic gates; resolve any
compile error and complete receipt validator/failure attribution before owner witness execution.
No GUI/Tor/policy/working-package changes or merge.

## PR101 MODEL-IPC-01 COMPILE-FIX — 2026-11-04

Branch fix/windows-official-tor-runtime; exact parent 4bb3cbc8ac56a6ff9746cef57b6128b363e6f352; PR #101 OPEN/DRAFT/unmerged.
Observed parent CI: 14/17 workflows PASS; failures desktop-shell-check 41832410580,
windows-appcontainer-ipc-probe-artifact 41832410873 and discovery deterministic contracts 41832410542.
Recovered logs: job126582629464 E0277 LowerHex at witness lines14/20;
job126582632981 E0432/E0433 from dependency-free rustc inclusion of the model witness.
Replace digest-array LowerHex with explicit byte/nibble lowercase hex, preserving leading zeroes;
add empty/all-nibble-range vector checks alongside existing abc vector.
Run the same Windows isolation suite through locked Cargo (compile-only then filtered lib tests),
including model deterministic tests; ignored real host/Qwen probes remain ignored. No test removed.
Local Cargo/rustc unavailable in PATH; no local build attempted. One automatic new-head CI wave
is PENDING at publication. No manual rerun, real model launch or merge.
Protected PROJECT_STATE.json, global last_verified_commit, GUI, Tor, security policy and working binaries unchanged.
Receipt validation/failure-attribution completeness remains outside this compile-only phase.
Real Qwen-in-AppContainer, Windows cleanup and independent network egress remain NOT TESTED.

ONE NEXT ACTION: observe exact publication SHA CI (especially the three failed parent workflows);
retain run IDs and first failure if any. Do not repeat PREPARE or claim witness acceptance from compilation.

## PR101 MODEL-IPC-01 FAILURE-ATTRIBUTION — WIP (2026-11-04)

Exact parent 2019cf7f065c849bc6dcb0623a44dbfb181b1f58, branch fix/windows-official-tor-runtime.
PR101 OPEN/DRAFT/unmerged; observed parent latest CI 17/17 SUCCESS.
Arti config run41838391577 succeeded after the single targeted runner-loss retry
(job126656552825); original failed job126603201172 retained.
COMPILE-FIX is scoped verified; protected global anchor remains unchanged.

Atomic correction: setup/spawn, bounded-output and typed-decoding errors now mark the
active probe ERROR with measured duration and exact error, rather than leaving SKIP.
Cleanup status is independent; run, cleanup and both bounded output-write errors are
retained separately, and original run failure keeps return precedence. Both writes
are attempted. Added deterministic synthetic early-exit/combined-failure/success tests.
No new harness, model launch, production/security/GUI/Tor/binary changes or merge.
Local Cargo/rustc absent in PATH: no local build attempted. Exact new-head hosted tests PENDING.
Full receipt schema/provenance validation remains a separate gate; this is NOT complete
matrix acceptance. Receipt root/generation creation and final receipt write can still
fail without a saved receipt. Real Qwen, Windows host cleanup and network egress NOT TESTED.

ONE NEXT ACTION: observe exact new-head deterministic CI; after acceptance, address
receipt schema/provenance validation before any costly owner witness execution.
