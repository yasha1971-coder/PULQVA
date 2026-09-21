# T035 — Add the typed desktop Download action boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Add the explicit Download action required by the UX contract while keeping this step data-only.

## Acceptance criteria

- frontend exposes Download only after a candidate has been validated and selected;
- frontend sends only validated intent text + opaque selected locator through Tauri IPC;
- Rust reconstructs and revalidates the deterministic candidate set;
- selected locator must exactly match one validated `SearchCandidate`;
- typed download-action response contains deterministic intent/title/locator/action/stage fields;
- frontend renders the planned action as inert text;
- no locator interpretation or conversion to URL exists;
- no yt-dlp, FFmpeg, shell, network, or filesystem output is started;
- frontend retains no external Internet API;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- actual download execution;
- external candidate search;
- AI provider integration;
- Autopilot;
- packaging/release installers.
