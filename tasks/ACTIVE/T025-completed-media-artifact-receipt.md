# T025 — Add a typed completed-media artifact receipt

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Convert a successful isolated yt-dlp output into a typed completed-media receipt suitable for the
future UI/download handoff.

## Acceptance criteria

- receipt is created only after successful child completion;
- launched child retains the exact explicit output root from `YtDlpMediaRequestPlan`;
- exactly one completed regular file is required;
- symlinks anywhere in the output tree are rejected;
- output roots containing parent-directory traversal are rejected;
- canonical artifact path must remain inside the canonical output root;
- zero-byte artifacts are rejected;
- receipt includes canonical artifact path and byte size;
- failed child completion yields no receipt;
- no network activity is added;
- Windows and Linux privacy tests cover traversal/symlink/empty/multiple/zero-byte rejection;
- all existing checks remain green.

## Out of scope

- FFmpeg;
- AI;
- UI;
- arbitrary user URL execution.
