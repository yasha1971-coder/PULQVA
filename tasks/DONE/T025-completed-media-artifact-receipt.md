# T025 — Add a typed completed-media artifact receipt

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added `CompletedMediaArtifactReceipt` and completion-gated validation.

A receipt is created only after successful yt-dlp child completion and only for exactly one
non-empty regular file contained within the canonical output root. Symlinks, parent traversal,
path escape, empty output, multiple files, and zero-byte artifacts are rejected.

## Verification

PR #24 verified head:
`3e03bd96302f7458947c661f38a65a762e5129b3`

All existing checks passed.
