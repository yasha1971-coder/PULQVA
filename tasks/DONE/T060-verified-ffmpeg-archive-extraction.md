# T060 — Safely extract FFmpeg from the verified packaged archive

Parent: DESKTOP FOUNDATION
Status: DONE — extraction implementation verified; documentation closeout/merge pending

## Verified closeout — 2026-09-25

Head `24a7a066f938bc7d6336deb39cc56c84a132b891` passed all 11 workflows. Desktop run 36095498238
passed Windows/Linux checks; extraction code is unchanged from the audited implementation.
Media diagnostic run 36095497965 passed: Linux job 107946800673 emitted
PULQVA_YTDLP_TOR_MEDIA_OK; Windows job 107946800507 emitted
PULQVA_YTDLP_TOR_MEDIA_WINDOWS_FAIL_CLOSED_OK after readiness timeout.
Windows live media retrieval is NOT proven by this result.
The earlier Linux exit-1 cause remains undetermined and was not reproduced.
Diagnostic capture tests passed on both platforms. No claim that the original cause was fixed.

Scope completed: authenticated source -> bounded extraction -> owned staging receipt.
Not included: runtime publication, executable activation, full packaged-app journey.
Existing filesystem race/crash cleanup limitations remain.
Next work after merge: T061 Deno/EJS compatibility assessment.

## Acceptance audit — 2026-09-25

Audited implementation: `d2f746e46f6e933454e0bc28adc189ccb702ce03`, PR #62.
All 11 workflows succeeded. Desktop run 36064257789 explicitly passed the real pinned
archive adapter test on Windows and Linux, in addition to compilation and boundary tests.

| Criterion | Implementation / evidence |
| --- | --- |
| Backend kind/version/platform binding | ffmpeg_source::extract_verified_ffmpeg; real integration rejects wrong kind/version/destination |
| ZIP and tar.xz | ffmpeg_zip / ffmpeg_tar_xz; both real pinned platform archives match independent Python hash and size |
| Source containment and authenticated consumption | ffmpeg_source::snapshot validates paths/type/identity and hashes one bounded immutable snapshot; size/digest/path rejection tests |
| Exactly one expected member | ffmpeg_archive_policy; duplicate/missing/path/type tests; ZIP rejects collapsed entry counts |
| Bounded expansion | metadata count/member/total limits, ZIP actual selected-byte limit, tar global byte budget and XZ decoder memory limit |
| Owned output and typed identity | ffmpeg_stage exclusive creation; sync and same-handle rehash; receipt binds archive/executable digests, size and owned path; outer result binds version/platform |
| Preserve existing files / cleanup | stage error and ownership tests; nonrecursive identity-checked Drop cleanup |
| Cross-platform deterministic checks | desktop boundary tests and explicit real archive integration step succeeded on both OSes |
| No executable/network/frontend operation | extraction modules use local parsing/hash/filesystem only; no live prelaunch caller |

Limits remain explicit: this is not a guarantee against hostile concurrent ancestor replacement,
all Windows reparse behaviors or process crashes. Drop cleanup is best-effort; identity-acquisition
failures can leave an owned orphan. Source authentication guarantees the bytes passed to parsers.
Later executable publication must revalidate stage ownership and content.

Audit result: extraction scope has supporting implementation and green CI evidence.
Extraction implementation is complete at the verified head below. This closeout document still requires its own CI and merge.
No runtime publication, prelaunch activation, packaged-app launch or Deno/EJS test is claimed.

## Historical implementation checkpoints

Head `c22dc198182db1058de0474f847de30a9822cffb` passed all 11 workflows. Current phase
adds verified-artifact/layout binding, bounded same-handle source snapshot + digest validation,
containment/type/identity revalidation, and typed extraction-to-staging entrypoint. Real archive
tests cover this entrypoint on both OSes. Compile/tests must pass before closeout audit.

Head `50bacca94652e5bdf00b6d5cdac95420abe7a05b` passed all 11 workflows including
independent real-archive output comparison on Windows/Linux. Current phase adds owned
staging/lifetime cleanup and disk-content verification against adapter receipts; real-archive
CI now tests that path too. Source-file snapshot/containment and backend plan integration
remain pending. No adversarial filesystem race or crash-cleanup guarantee is claimed.

T060-C head `120f6968dbd59db8041042a611d1117e69743204` passed all 11 workflows.
Current phase adds Windows/Linux real pinned-archive proof to desktop CI: authenticate the
archive, independently hash the chosen member, compare real Rust adapter output hash/size.
No executable is launched. Results must be observed before claiming real compatibility.

## Bounded implementation phases

T059 is merged at `d14b742e9662b1ccb68f64d28500e97efdff5454` with all 11 main workflows green.
Per AGENTS task-splitting rules, common policy, two parsers and authenticated staging are
separate verifiable changes; implementing all safely exceeds a bounded phase.

- T060-A (current): pure metadata policy and tests for member names/types, exact executable
  selection, duplicate selection, bounded declared sizes/count and poisoned failure state.
- T060-B: bounded ZIP adapter over authenticated immutable source bytes to an internal writer.
- T060-C: tar.xz adapter with the same contract and actual decompressed-byte limits.
- Final integration: shared authenticated extraction receipt and ownership/cleanup tests.

T060-B head `ecbe966eb8c3f5f5f9aa63dbf425ba5a147981e9` passed all 11 workflows.
T060-C adds tar.xz to an internal writer, source authentication, bounded decoder memory and
actual decompressed bytes, raw-entry policy, output hashing and XZ footer/trailing checks.
GNU/PAX extension records are intentionally unsupported until real archive compatibility is
inspected. Source-file/staging ownership remains a later integration; parent task NOT DONE.

T060-A head `b6b0dbd8d9b6c0191a659e693b7c0db11952b11f` passed all 11 workflows.
T060-B adds stored/deflate extraction, immutable-source SHA verification, metadata gate,
actual output limits/CRC and hash receipt. It does not own staging; partial writer output on
failure must be discarded by the later owned-stage integration. Real archive compatibility,
tar.xz and source-file containment/ownership tests remain pending.

T060-A does not authenticate archives, decompress data, write files or enforce actual streamed
byte limits. Header-size checks alone are not a decompression-bomb defense.
No production caller uses it until adapters meet the complete contract below.

## Outcome

Provide one backend-only bounded extraction boundary that returns a receipt for the
FFmpeg executable extracted from a T054-verified platform archive into an owned staging area.
The archive SHA authenticates the source archive, not a directly copied executable.

## Acceptance criteria

- accept only the backend-owned FFmpeg archive kind, version and pinned platform identity;
- support the recorded Linux tar.xz and Windows zip source formats;
- revalidate source containment, regular-file type and identity at consumption, avoiding a
  verify-then-reopen gap; extract only bytes covered by the authenticated archive;
- identify exactly one expected platform FFmpeg executable member using backend-owned rules;
- reject traversal/absolute paths, ambiguous duplicate executable members, symbolic/hard links,
  special files and unsafe staging destinations; enforce bounded archive/member expansion;
- stream the selected executable into exclusively owned staging, hash the actual extracted bytes,
  and return a typed receipt binding archive identity, executable identity, size and owned path;
- preserve pre-existing files and clean up only owned incomplete output on error;
- deterministic Windows/Linux tests cover successful extraction and concrete rejection cases;
- never execute the extracted binary, shell commands, network operations or frontend paths.

## Scope limits

Do not wire into live prelaunch, publish into runtime/bin, activate bundles, build installers,
change source archives/digests, or add AI/search providers. Runtime publication and prelaunch
integration are later outcomes. Split extraction by format only if the bounded task cannot be
completed as one verifiable boundary. Preserve Tor and fail-closed invariants.
