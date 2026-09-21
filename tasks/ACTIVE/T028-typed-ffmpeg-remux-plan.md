# T028 — Add a typed local FFmpeg remux plan

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Create a pure-data local FFmpeg remux plan whose input can come only from
`CompletedDownloadResult` and whose output target is explicit.

## Acceptance criteria

- typed `FfmpegRemuxPlan` exists;
- plan requires an explicit non-empty FFmpeg executable path;
- input media path is copied only from `CompletedDownloadResult`;
- no HTTP(S), proxy, or other network input form exists;
- output path is explicit and non-empty;
- output path cannot equal the validated input artifact path;
- parent-directory traversal in output is rejected;
- output container is a typed supported value: MP4 or Matroska;
- argv is deterministic;
- argv includes `-nostdin`, `-y`, `-protocol_whitelist file`, `-map 0`, and `-c copy`;
- no shell command or FFmpeg process is created;
- tests cover invalid output and deterministic argv;
- all existing checks remain green.

## Out of scope

- launching FFmpeg;
- transcoding/re-encoding;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
