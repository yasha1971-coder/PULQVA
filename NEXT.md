# NEXT

## Current verified state

T052 is complete.

Verified T052 closeout head:
`e693f3e0d095fdca39c0cff55b396043e91f590d`

All 11 required workflows passed for that exact head. T052 is merged on main at
`7205d3a6d07645fd05b670c393686609384d0288`.

## Active atomic task

**T053 — Resolve packaged sidecar source root from the Tauri resource directory**

T053 binds the T052 logical packaged-resource identifiers to one backend-owned package resource root:

- the real command resolves the resource root only through injected `AppHandle` using
  `app.path().resource_dir()`;
- frontend supplies no resource/filesystem path;
- logical T052 resource identifiers remain backend-owned relative paths;
- a pure typed resolver joins each logical identifier beneath the resource root;
- empty/traversing resource roots and non-normal logical resource paths fail closed;
- every resolved source must remain beneath the verified resource root;
- resolved source and runtime destination must differ;
- T052 kind/version/SHA-256 identity data is preserved unchanged;
- runtime destinations remain direct children of `runtime/bin`;
- this boundary does not require source files to exist and performs no copy/write/chmod/process/network operation;
- no bundle resources are activated by this task.

The Tauri 2 resource-directory API used here is the documented backend `PathResolver::resource_dir`
surface; no frontend path API is used.

Branch:
`task/T053-resolve-packaged-sidecar-source-root`

## Next task

T053 remains next until its exact head is verified green and closed.

## Do not do yet

- no sidecar binary copy/materialization;
- no Tauri bundle-resource activation;
- no installer/release packaging;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Windows/Linux desktop boundary tests and all existing privacy/media checks are green on the exact
T053 head, with all three sidecar source filesystem paths derived solely from the app-owned Tauri
resource directory and all runtime destinations still confined to `runtime/bin`.
