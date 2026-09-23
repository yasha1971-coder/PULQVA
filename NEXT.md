# NEXT

## Current verified state

T046 is complete.

Merged T046 main:
`7d6beb63071963c3442b53da15e1471d84dba6ff`

## Active atomic task

**T047 — Add a backend-only one-shot completed-file orchestration boundary**

T047 composes the existing verified boundaries into one backend-only download-to-file path:

- input is a validated `SearchCandidate` plus explicit backend runtime inputs;
- `CompletedFilePipelineInputs` carries the existing download runtime inputs, explicit FFmpeg
  executable path, and explicit Tor readiness timeout;
- an empty FFmpeg executable fails before any runtime phase starts;
- preflight, Arti preparation/readiness, Tor-gated yt-dlp request planning/launch, completed download,
  FFmpeg remux planning/launch/completion, and sanitized completed-file conversion all reuse the
  existing T037–T046 boundaries;
- Rust `?` propagation stops orchestration immediately on every failed phase;
- no direct-network fallback exists;
- FFmpeg still receives only the validated local artifact and remains local-file-only;
- final output is only the sanitized `CompletedFileView`;
- no Tauri command/frontend wiring is added;
- no backend filesystem path, executable path, argv, source URL, proxy/SOCKS data, process
  identifier, or raw completion internals cross the output boundary.

Branch:
`task/T047-backend-one-shot-completed-file-orchestration-boundary`

## Next task

T047 remains next until its exact head is verified green and closed.

## Do not do yet

- no live desktop command wiring;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

Desktop/privacy/media/remux checks are green for the exact T047 head and the backend-only orchestration
function composes the verified Tor-first download-to-sanitized-file pipeline without duplicating
network/process logic.
