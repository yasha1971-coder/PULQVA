# PULQVA Desktop Shell

Minimal Tauri 2 shell for Windows and Linux.

The frontend is static local content. It has no frontend-owned external network API and calls only
the Rust `app_status` command through Tauri IPC.

Runtime external networking remains owned by the Rust privacy boundary.
