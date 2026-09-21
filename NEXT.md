# NEXT

## Current verified state

T027 is complete.

PULQVA now pins and proves the FFmpeg/ffprobe sidecar before any media transformation is allowed:

- upstream FFmpeg source commit is pinned to
  `a5923073bfd8f25b7300d93af3f8e690174ebd30`;
- version marker is `n9.0.2-3-ga5923073bf`;
- Linux and Windows archives come from one dated BtbN build snapshot;
- both archives are SHA-256 verified before extraction or execution;
- `ffmpeg -version` and `ffprobe -version` are green on Windows and Linux;
- no runtime network route or media transformation was introduced.

Verified PR head:
`84cd0dca0e6610cabec9f42a1f9905065856fe25`

## Active atomic task

**T028 — Add a typed local FFmpeg remux plan**

T028 adds a pure-data `FfmpegRemuxPlan`:

- explicit FFmpeg executable path;
- input path copied only from `CompletedDownloadResult`;
- typed output container: MP4 or Matroska;
- explicit non-empty output path;
- output cannot equal the validated input artifact;
- parent-directory traversal in output is rejected;
- deterministic argv uses `-nostdin`, `-y`, `-protocol_whitelist file`, `-map 0`, and `-c copy`;
- no shell, process spawn, URL input, proxy input, or network route exists in the plan.

## Queued next task

**T029 — Add a controlled FFmpeg child-process launcher**

Put the first FFmpeg process side effect behind `FfmpegRemuxPlan` and prove exact argv plus
deterministic process cleanup with a local fixture before any real remux is executed.

## Do not do yet

- no real FFmpeg media transformation;
- no desktop UI implementation;
- no AI provider;
- no arbitrary user URL execution;
- no direct-network fallback.

## Success

A validated completed download can become a deterministic local-only remux plan without exposing any
network input surface or creating an FFmpeg process.
