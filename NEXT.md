# NEXT

## Current verified state

T035 is complete.

Verified functional checkpoint:
`1f295ccee7631a699a3ecbd4922ee796b7a1f462`

The bookkeeping closeout is merged on main at:
`629ef51ba3efdd334899f8bfbf679ac17fd5b2ef`

## Active atomic task

**T036 — Add the backend-only typed media-source resolution boundary**

Implementation boundary:

- desktop Rust links the existing `pulqva-privacy` crate;
- raw frontend locators are first revalidated against the deterministic `SearchCandidate` set;
- only the exact proof locator `local:test:candidate:official-live` is supported;
- that typed candidate maps inside Rust to the immutable T024 media object;
- the mapping is constructed as `YtDlpMediaSourceUrl`;
- validated candidates without an approved mapping fail closed;
- arbitrary/unknown locators still fail before media-source resolution;
- the frontend receives only inert media-source readiness/stage data, never the URL;
- no Tor bootstrap, yt-dlp, FFmpeg, shell, external network, or filesystem output is started.

Branch:
`task/T036-backend-media-source-resolution`

## Next task

Not selected yet. T036 must be verified and closed before defining another implementation task.

## Do not do yet

- no external search provider;
- no AI provider;
- no actual download execution from the desktop UI;
- no yt-dlp/FFmpeg process from the desktop UI;
- no filesystem output from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T036 head,
with no media URL exposed to frontend code.
