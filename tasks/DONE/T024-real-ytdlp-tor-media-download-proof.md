# T024 — Prove one bounded real media download through Tor

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Proved one fixed public media object through the full typed Tor-only yt-dlp path using the actual
pinned Arti 2.6.0 and yt-dlp 2026.08.19 binaries.

Linux produced exactly one regular media artifact of 5,510,872 bytes in the isolated output root.
Windows remained bounded/fail-closed when Tor readiness was unavailable. No FFmpeg or direct retry
was used.

## Verification

PR #23 verified head:
`ea371388699f4fc9347815681c065299c31efe23`

All existing checks passed, including `ytdlp-tor-media-check`.
