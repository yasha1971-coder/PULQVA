# NEXT

## Current verified state

T034 is complete.

PULQVA now has an explicit typed candidate-selection boundary:

- every displayed local candidate has one Select action;
- selection sends only validated intent text + opaque locator through Tauri IPC;
- Rust reconstructs the deterministic validated candidate set;
- locator matching is exact and fail-closed;
- unknown locators are rejected;
- the selection result remains inert title/locator/stage data;
- no provider, network, shell, filesystem, or media side effect is involved.

Verified PR head:
`adad7423dab41387ca70d483bbedc466b50f1bf5`

## Active atomic task

**T035 — Add the typed desktop Download action boundary**

T035 adds the UX contract's explicit Download action while keeping it data-only:

- Download appears only after a candidate has been validated and selected;
- frontend sends only validated intent text + the opaque selected locator;
- Rust reconstructs and revalidates the candidate set again;
- the locator must exactly match a validated `SearchCandidate`;
- the response contains only deterministic intent/title/locator/action/stage data;
- the opaque locator is not interpreted or converted to a URL;
- no yt-dlp, FFmpeg, shell, network, or filesystem action starts.

## Queued next task

**T036 — Add the backend-only typed media-source resolution boundary**

Resolve one revalidated local proof candidate to a typed `YtDlpMediaSourceUrl` entirely inside Rust,
using an exact backend-only mapping to the immutable T024 media object. The frontend must never
receive or construct the media URL, and no process/network/filesystem side effect may start yet.

## Do not do yet

- no external search provider;
- no AI provider;
- no actual download execution from the desktop UI;
- no yt-dlp/FFmpeg process from the desktop UI;
- no filesystem output from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

After a validated selection, the user can press Download and receive a typed, fail-closed,
data-only Download action plan while the selected opaque locator remains inert.
