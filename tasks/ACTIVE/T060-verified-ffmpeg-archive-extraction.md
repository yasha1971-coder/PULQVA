# T060 — Safely extract FFmpeg from the verified packaged archive

Parent: DESKTOP FOUNDATION
Status: ACTIVE — T060-A verified; T060-B ZIP stream adapter CI pending

## Bounded implementation phases

T059 is merged at `d14b742e9662b1ccb68f64d28500e97efdff5454` with all 11 main workflows green.
Per AGENTS task-splitting rules, common policy, two parsers and authenticated staging are
separate verifiable changes; implementing all safely exceeds a bounded phase.

- T060-A (current): pure metadata policy and tests for member names/types, exact executable
  selection, duplicate selection, bounded declared sizes/count and poisoned failure state.
- T060-B: bounded ZIP adapter over authenticated immutable source bytes to an internal writer.
- T060-C: tar.xz adapter with the same contract and actual decompressed-byte limits.
- Final integration: shared authenticated extraction receipt and ownership/cleanup tests.

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
