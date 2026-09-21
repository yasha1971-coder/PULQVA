# T031 — Bootstrap the Tauri 2 desktop shell

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Create the first Windows/Linux Tauri 2 desktop shell around the existing Rust core without giving
the frontend ownership of external network access.

## Acceptance criteria

- Tauri 2 desktop app scaffold exists in-repo;
- Windows and Linux compile checks exist;
- Rust backend exposes one minimal typed status/version command;
- frontend calls only the typed Tauri command boundary;
- no frontend fetch/XHR/WebSocket/external URL call is introduced;
- no remote web content is loaded;
- application CSP/network configuration does not grant arbitrary external connectivity;
- existing privacy/media checks remain green.

## Out of scope

- real search/download UI flow;
- AI provider integration;
- arbitrary user URL execution;
- packaging/release installers.
