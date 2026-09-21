# T028 — Add a typed local FFmpeg remux plan

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Create a pure-data local FFmpeg remux plan whose input can come only from
`CompletedDownloadResult` and whose output target is explicit.

## Acceptance criteria

- typed FFmpeg remux plan exists;
- plan requires an explicit FFmpeg executable path;
- input media path comes only from `CompletedDownloadResult`;
- no HTTP(S), proxy, or other network input form exists;
- output root/path is explicit and non-empty;
- output path cannot equal the validated input artifact path;
- output container is a typed supported value;
- argv is deterministic and includes non-interactive/local-only process flags;
- no shell command or FFmpeg process is created;
- tests cover invalid output and deterministic argv;
- all existing checks remain green.

## Out of scope

- launching FFmpeg;
- transcoding/re-encoding;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
