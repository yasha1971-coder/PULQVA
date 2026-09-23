# T053 — Resolve packaged sidecar source root from the Tauri resource directory

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Bound the T052 logical packaged-resource identifiers to the desktop application's backend-owned Tauri
resource directory without materializing executable bytes.

The real desktop command resolves the package resource root only through injected `AppHandle` using
`app.path().resource_dir()`. Frontend input cannot provide a resource or filesystem path.

Logical source identifiers remain backend-owned relative paths. Empty/traversing package resource
roots and invalid resource identifiers fail closed. Every resolved source path must remain beneath the
resource root and must differ from its app-owned runtime destination.

T052 sidecar kind/version/SHA-256 identity metadata is preserved unchanged, and runtime destinations
remain direct children of `runtime/bin`.

No source file is copied, written, chmodded, launched, downloaded, or accessed through external
networking. Tauri bundle-resource activation remains out of scope.

## Verification

PR #55 verified head:
`fc0f1664e72b37f4d67de24d495d0f2a9c26c0bb`

All 11 required workflows passed.
