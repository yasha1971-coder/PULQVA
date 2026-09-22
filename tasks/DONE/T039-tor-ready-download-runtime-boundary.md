# T039 — Add the backend-only Tor-ready download runtime boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only Tor-ready download runtime boundary.

A prepared T038 runtime can launch only its prepared Arti child through the existing controlled
`launch_prepared_arti` path and verify Tor readiness through the existing bounded
`verify_tor_readiness` path.

On success the backend retains the running Arti child, `ReadyTorTransport`, explicit yt-dlp
executable path, typed `YtDlpMediaSourceUrl`, and output root.

Arti launch failures fail closed. Tor readiness failures explicitly stop and wait for the Arti child
before returning; cleanup failures are surfaced separately. Frontend IPC/output remains unchanged and
exposes none of the backend paths, SOCKS/proxy information, or media URL.

No yt-dlp or FFmpeg process is started and no media download occurs.

## Verification

PR #39 verified head:
`0821cfc0bb70b9f43388fecdaf50e253d72a22ad`

All 11 required workflows passed:

- continuity-guard;
- rust-check;
- desktop-shell-check;
- arti-sidecar-check;
- arti-materialization-check;
- arti-config-contract;
- arti-lifecycle-check;
- tor-readiness-check;
- ytdlp-sidecar-check;
- ytdlp-tor-metadata-check;
- ytdlp-tor-media-check.
