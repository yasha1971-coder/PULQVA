# T031 — Bootstrap the Tauri 2 desktop shell

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added the first Windows/Linux Tauri 2 desktop shell.

The Rust backend links `pulqva-core` and exposes a typed `app_status` command. The frontend is
static local content, uses only Tauri IPC, and is protected by a CSP that grants no arbitrary
external connectivity. Windows/Linux desktop compile checks are green.

## Verification

PR #30 verified head:
`6faff7d194d87e714ae3b50d34834223a34309b1`

All existing checks passed, including `desktop-shell-check`.
