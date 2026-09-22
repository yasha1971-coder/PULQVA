# T035 — Add the typed desktop Download action boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added the explicit Download action after validated candidate selection while keeping the boundary
strictly data-only.

The frontend sends only validated intent text plus the opaque selected locator. Rust reconstructs
and revalidates the deterministic local candidate set, requires an exact locator match, and returns
only deterministic intent/title/locator/action/stage data. Unknown locators fail closed.

The opaque locator is not interpreted as a URL and T035 starts no yt-dlp, FFmpeg, shell, network, or
filesystem side effect.

## Verification

Merged PR #34 as:
`1f295ccee7631a699a3ecbd4922ee796b7a1f462`

Post-merge push CI on that exact commit passed all 11 workflows:

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
