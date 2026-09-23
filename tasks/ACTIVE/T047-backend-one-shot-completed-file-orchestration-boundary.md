# T047 — Add a backend-only one-shot completed-file orchestration boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Compose the existing verified download/remux boundaries into one backend-only path that produces the
sanitized T046 completed-file view.

## Acceptance criteria

- input is a validated local candidate plus explicit backend runtime inputs;
- reuse existing T037–T046 typed boundaries;
- Tor readiness is required before yt-dlp launch;
- no direct-network fallback exists;
- yt-dlp completion reuses the existing validated completed-download boundary;
- Arti cleanup remains deterministic and fail-closed;
- FFmpeg plan input comes only from the validated completed download;
- FFmpeg execution remains local-file-only;
- FFmpeg completion reuses the typed validated completed-remux boundary;
- final output is only the sanitized T046 completed-file view;
- no later phase starts after an earlier failure;
- no backend filesystem path, executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw completion internals cross the output boundary;
- no Tauri command/frontend wiring is added;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- live desktop command wiring;
- external candidate search;
- AI provider integration;
- packaging/release installers.
