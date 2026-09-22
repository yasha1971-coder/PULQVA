# NEXT

## Current verified state

T036 is complete.

PULQVA now has a backend-only typed media-source resolution boundary:

- desktop Rust links the existing `pulqva-privacy` crate;
- candidate locators are revalidated against the deterministic `SearchCandidate` set;
- only `local:test:candidate:official-live` has an approved proof mapping;
- the approved mapping resolves inside Rust to the immutable T024 media object;
- the resolved value is constructed as `YtDlpMediaSourceUrl`;
- validated but unsupported candidates fail closed;
- unknown locators fail closed before resolution;
- the frontend receives only inert readiness/stage data and never receives the media URL;
- no Tor bootstrap, yt-dlp, FFmpeg, shell, network, or filesystem side effect starts.

Verified PR head:
`510c737f4845bd4c40c1b2a1571b116bf59bcd66`

All 11 required workflows passed for that exact head.

## Next atomic task

**T037 — Add the typed desktop download preflight specification**

Create a pure-data backend preflight specification for the later real Download path. It must combine
only already-validated local inputs needed by later runtime orchestration while keeping process and
network execution out of scope.

Required boundary:

- input is a revalidated supported candidate whose media source resolves through T036;
- sidecar executable identities/paths are explicit backend inputs;
- Tor runtime directories and download output root are explicit backend inputs;
- no frontend media URL or executable path surface is introduced;
- no process spawn, Tor bootstrap, yt-dlp/FFmpeg execution, external network, or filesystem output;
- invalid or missing required inputs fail closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no real Download execution from the desktop UI;
- no Tor bootstrap from the desktop UI;
- no yt-dlp/FFmpeg process from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

The next backend boundary is a deterministic typed preflight specification that can later feed the
existing Tor/yt-dlp runtime types without exposing privacy-sensitive or executable data to frontend code.
