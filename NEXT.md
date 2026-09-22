# NEXT

## Current verified state

T045 is complete.

Merged T045 main:
`e6018e8697885025be6201c841b66e41c892c0f9`

## Active atomic task

**T046 — Add a sanitized desktop completed-file view boundary**

T046 adds a data-only desktop success value derived from a verified completed remux:

- input is a verified `CompletedFfmpegRemuxResult`;
- output contains only a display filename, byte size, and stable `completed-file-ready` stage;
- filename comes only from the validated output file-name component;
- invalid/missing filename fails closed;
- display text replaces control characters and path separators with `_`;
- no absolute or parent path is serialized;
- no executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw backend result
  internals are serialized;
- frontend command wiring remains unchanged and out of scope;
- no process or external network access is introduced.

Branch:
`task/T046-sanitized-desktop-completed-file-view-boundary`

## Next task

T046 remains next until its exact head is verified green and closed.

## Do not do yet

- no live desktop command wiring for completed results;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

Desktop/privacy/remux checks are green for the exact T046 head and no backend filesystem/runtime
internals cross the desktop view boundary.
