# T031 — Bootstrap the Tauri 2 desktop shell

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Create the first Windows/Linux Tauri 2 desktop shell around the existing Rust core without giving
the frontend ownership of external network access.

## Acceptance criteria

- Tauri 2 desktop app scaffold exists in-repo;
- Tauri is pinned to 2.11.6 and tauri-build to 2.6.3;
- Windows and Linux compile checks exist;
- Rust backend links `pulqva-core`;
- Rust backend exposes one typed `app_status` status/version command;
- frontend calls only the typed Tauri IPC command boundary;
- no frontend fetch/XHR/WebSocket/EventSource/sendBeacon is introduced;
- no remote web content is loaded;
- CSP grants only local content plus required Tauri IPC endpoints;
- desktop frontend is static local HTML/CSS/JS with no remote dependency;
- existing privacy/media checks remain green.

## Out of scope

- real search/download UI flow;
- AI provider integration;
- arbitrary user URL execution;
- packaging/release installers.
