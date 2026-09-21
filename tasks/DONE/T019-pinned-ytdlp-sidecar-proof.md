# T019 — Pin and prove the yt-dlp standalone sidecar

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Pinned official yt-dlp release 2026.08.19.

Pinned and verified:

- yt-dlp_linux SHA-256:
  58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a;
- yt-dlp.exe SHA-256:
  66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a.

Windows and Linux passed exact --version and local --ignore-config --help checks. No media URL
was supplied.

## Verification

PR #18 verified head:
dea983cc546a2239612b6bf592ca49db4f2cb5f7

All existing privacy and continuity checks passed.
