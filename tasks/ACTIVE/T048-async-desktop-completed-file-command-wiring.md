# T048 — Add async desktop completed-file command wiring

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Expose the verified T047 backend pipeline to the Tauri desktop shell while keeping runtime inputs
backend-owned and keeping blocking process work off the UI thread.

## Acceptance criteria

- frontend input is only query text plus validated candidate locator;
- query is revalidated through `SearchIntent`;
- locator is revalidated against the local candidate set;
- unknown/unsupported candidates fail closed before runtime work;
- backend constructs Arti, yt-dlp, FFmpeg, Tor state/cache/config, output-root, and timeout inputs;
- no executable path, filesystem root, proxy/SOCKS value, media URL, argv, or process identifier is accepted from frontend input;
- command execution moves the blocking T047 pipeline off the UI thread;
- task-join failure maps to a typed fail-closed desktop error;
- successful output is only the sanitized `CompletedFileView`;
- no duplicate direct network/process implementation is introduced;
- no direct-network fallback exists;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- external candidate search;
- AI provider integration;
- packaging/release installers.
