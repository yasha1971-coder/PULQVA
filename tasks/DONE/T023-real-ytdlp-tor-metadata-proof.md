# T023 — Prove a real pinned yt-dlp metadata-only request through Tor

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added explicit metadata-only request mode and proved it with the actual pinned Arti 2.6.0 and
yt-dlp 2026.08.19 binaries.

The request requires `ReadyTorTransport`, uses the verified `socks5h://` route, is bounded, and
adds `--skip-download --dump-single-json --no-playlist`. Successful execution creates no media
payload and there is no direct fallback.

## Verification

PR #22 verified head:
`ebc5da77ccc203095c102c6dcdf80ea4f2170c9f`

All existing checks passed, including `ytdlp-tor-metadata-check`.
