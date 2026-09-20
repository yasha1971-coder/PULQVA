# T016 — Prove real Arti deferred-bootstrap lifecycle

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Prove that PULQVA's prepared-runtime launcher can control the actual pinned Arti 2.6.0 sidecar on
Windows and Linux before any request is allowed to trigger Tor bootstrap.

## Acceptance criteria

- test uses the real pinned Arti 2.6.0 binary;
- real launch goes through `PreparedArtiRuntime`;
- verified config keeps `application.defer_bootstrap = true`;
- direct child spawn, no shell;
- no SOCKS request;
- no DNS request;
- no readiness probe;
- no user workload is sent to Arti;
- child can be stopped and reaped deterministically;
- Windows and Linux CI coverage;
- all existing checks remain green.

## Out of scope

- triggering Tor bootstrap;
- Tor reachability verification;
- SOCKS readiness probing;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
