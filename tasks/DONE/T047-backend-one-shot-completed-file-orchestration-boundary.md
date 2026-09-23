# T047 — Add a backend-only one-shot completed-file orchestration boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added one backend-only download-to-file orchestration boundary.

The new path composes the existing T037–T046 typed boundaries instead of duplicating process or
network logic. Starting from a validated candidate plus explicit backend runtime inputs, it performs
download preflight, Arti preparation and readiness verification, Tor-gated yt-dlp planning and launch,
validated completed download handling with deterministic Arti cleanup, local-only FFmpeg remux
planning and execution, validated FFmpeg completion, and final sanitized completed-file conversion.

Every failed phase returns before any later phase starts. Tor remains mandatory, no direct-network
fallback exists, FFmpeg receives only the validated local artifact, and the final return value is only
the sanitized `CompletedFileView`.

No Tauri command/frontend wiring was added. No backend filesystem path, executable path, argv, source
URL, proxy/SOCKS data, process identifier, or raw completion internals cross the output boundary.

## Verification

PR #47 verified head:
`7c56f08b62fc6110c2167398fbf9f012e46856c7`

All 11 required workflows passed.
