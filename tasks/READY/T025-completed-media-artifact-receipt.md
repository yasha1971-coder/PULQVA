# T025 — Add a typed completed-media artifact receipt

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Convert a successful isolated yt-dlp output into a typed completed-media receipt suitable for the
future UI/download handoff.

## Acceptance criteria

- receipt is created only after successful child completion;
- output root is explicit;
- exactly one completed regular file is required for the initial contract;
- symlinks are rejected;
- canonical artifact path must remain inside the canonical output root;
- zero-byte artifacts are rejected;
- receipt includes artifact path and byte size;
- no network activity is added;
- no direct/clearnet fallback exists;
- Windows and Linux filesystem tests cover traversal/symlink/empty/multiple-file rejection;
- all existing checks remain green.

## Out of scope

- FFmpeg;
- AI;
- UI;
- arbitrary user URL execution.
