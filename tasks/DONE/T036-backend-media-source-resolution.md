# T036 — Add the backend-only typed media-source resolution boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only typed media-source resolution boundary to the desktop Rust backend.

The desktop now links `pulqva-privacy`. Raw locators are first revalidated against the deterministic
`SearchCandidate` set. Only the exact proof locator
`local:test:candidate:official-live` is supported for media resolution and maps internally to the
immutable T024 media object. The mapping is constructed as `YtDlpMediaSourceUrl`.

Validated candidates without an approved mapping fail closed. Unknown locators fail before media-source
resolution. The frontend receives only inert readiness/stage data and never receives, constructs,
parses, or stores the media URL.

No Tor bootstrap, yt-dlp, FFmpeg, shell, external network, or filesystem output is started.

## Verification

PR #36 verified head:
`510c737f4845bd4c40c1b2a1571b116bf59bcd66`

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
