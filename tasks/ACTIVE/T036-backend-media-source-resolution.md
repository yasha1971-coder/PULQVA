# T036 — Add the backend-only typed media-source resolution boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Bridge a revalidated data-only Download action to the existing typed media layer without exposing a
URL to the frontend and without starting any side effect.

## Acceptance criteria

- desktop Rust backend links the existing `pulqva-privacy` crate;
- only an exact revalidated local proof candidate locator can enter media-source resolution;
- backend-only deterministic mapping resolves the supported proof locator to the immutable T024 media object;
- resolved source is constructed as `YtDlpMediaSourceUrl`;
- unknown or unsupported locators fail closed;
- frontend never receives, constructs, parses, or stores the media URL;
- frontend response exposes only inert readiness/stage data;
- no Tor, yt-dlp, FFmpeg, shell, external network, or filesystem output is started;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- actual Tor bootstrap from desktop UI;
- actual download execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
