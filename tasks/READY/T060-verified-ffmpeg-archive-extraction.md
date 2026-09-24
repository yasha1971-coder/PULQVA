# T060 — Safely extract FFmpeg from the verified packaged archive

Parent: DESKTOP FOUNDATION
Status: READY (gated on T059 closeout CI and PR #61 merge)

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
