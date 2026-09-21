# NEXT

## Current verified state

T035 is complete.

PULQVA now has the explicit desktop Download action boundary required by the UX contract while the
boundary remains data-only:

- Download appears only after a candidate has been validated and selected;
- frontend sends only validated intent text + the opaque selected locator through Tauri IPC;
- Rust reconstructs and revalidates the deterministic `SearchCandidate` set;
- locator matching is exact and fail-closed;
- unknown locators are rejected;
- the result contains deterministic intent/title/locator/action/stage data only;
- the opaque locator is not interpreted or converted to a URL;
- no yt-dlp, FFmpeg, shell, network, or filesystem side effect starts.

Verified main commit:
`1f295ccee7631a699a3ecbd4922ee796b7a1f462`

All 11 post-merge push workflows completed successfully for that exact commit.

## Next atomic task

**T036 — Add the backend-only typed media-source resolution boundary**

Resolve one revalidated local proof candidate to a typed `YtDlpMediaSourceUrl` entirely inside Rust,
using an exact backend-only mapping to the immutable T024 media object.

Required boundary:

- only an exact revalidated local proof locator may enter resolution;
- unknown or unsupported locators fail closed;
- the frontend must never receive, construct, parse, or store the media URL;
- the frontend may receive only inert readiness/stage data;
- no Tor bootstrap, yt-dlp, FFmpeg, shell, external network, or filesystem output starts.

## Do not do yet

- no external search provider;
- no AI provider;
- no actual download execution from the desktop UI;
- no yt-dlp/FFmpeg process from the desktop UI;
- no filesystem output from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

The repository checkpoint truthfully records T035 as complete and leaves T036 as the single READY
next task without starting it.
