# T050 — Resolve app-owned runtime root from desktop application data

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Resolve the backend runtime root from the operating system's application data location and feed that
root into the verified T049 `AppRuntimeLayout`.

## Acceptance criteria

- runtime root is resolved through the Tauri desktop application path API;
- frontend provides no filesystem path or runtime location;
- resolved root is a PULQVA-owned child beneath the application data directory;
- the resolved root is passed through `AppRuntimeLayout::new`;
- path-resolution failure maps to a typed fail-closed desktop error;
- no executable path, filesystem root, proxy/SOCKS value, media URL, argv, or process identifier is accepted from frontend input;
- command output remains only the sanitized completed-file view;
- no external network access is introduced by resolution;
- no runtime binary materialization is added;
- no packaging/release installer work is added;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- runtime binary materialization;
- Tauri bundle resources;
- packaging/release installers;
- external candidate search;
- AI provider integration.
