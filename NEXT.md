# NEXT

## Current verified state

T033 is complete.

PULQVA now has a provider-neutral desktop candidate-list boundary:

- Rust validates the request as `SearchIntent`;
- local proof candidates are created only through `SearchCandidate`;
- the desktop receives deterministic title + opaque locator fields;
- the frontend renders locator values only as inert text;
- no provider, AI, URL, shell, media process, or frontend Internet access is involved.

Verified PR head:
`9b09501d6e8332437ec5583c5899961f41e08be5`

## Active atomic task

**T034 — Add the typed desktop candidate-selection boundary**

T034 adds explicit candidate selection without execution:

- each rendered candidate has one Select action;
- selection sends only validated intent text + opaque locator through Tauri IPC;
- Rust reconstructs the same deterministic validated candidate set;
- locator matching is exact and fail-closed;
- unknown locators are rejected;
- typed selection output contains only intent/title/locator/stage data;
- selection is rendered as inert text and triggers no network/media side effect.

## Queued next task

**T035 — Add the typed desktop Download action boundary**

Add the UX contract's explicit Download action after a validated candidate selection. The action must
remain data-only in T035: Rust revalidates the selected candidate and returns a typed download-action
request without starting yt-dlp, FFmpeg, networking, or filesystem output yet.

## Do not do yet

- no external search provider;
- no AI provider;
- no locator execution;
- no yt-dlp/FFmpeg process from the desktop UI;
- no filesystem output from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

The user can explicitly select one displayed candidate, Rust verifies that the locator belongs to the
validated local candidate set, and the opaque locator remains inert.
