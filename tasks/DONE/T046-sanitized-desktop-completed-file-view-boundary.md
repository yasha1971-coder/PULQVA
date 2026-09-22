# T046 — Add a sanitized desktop completed-file view boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a sanitized data-only desktop completed-file view boundary.

A verified `CompletedFfmpegRemuxResult` can now be converted into a value containing only a display
filename, byte size, and stable `completed-file-ready` stage. The filename is derived only from the
validated output file-name component. Missing/empty names fail closed, and control characters or path
separators are replaced in display text.

No absolute or parent filesystem path, executable path, argv, source URL, proxy/SOCKS data, process
identifier, or raw backend result internals are serialized. Frontend command wiring is unchanged. No
process or external network access is introduced.

## Verification

PR #46 verified head:
`2812dec01ed08b94a35f0c7b42561f8dccf689ff`

All 11 required workflows passed.
