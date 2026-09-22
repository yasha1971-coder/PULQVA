# NEXT

## Current verified state

T043 is complete.

PULQVA now has a backend-only FFmpeg remux planning boundary:

- input is a verified `CompletedDownloadResult`;
- FFmpeg executable path is explicit backend input;
- remux output is derived from the validated artifact as a sibling `pulqva-remux-*.mp4` path without UTF-8 filename assumptions;
- the typed plan reuses `FfmpegRemuxPlan` with explicit `FfmpegRemuxContainer::Mp4`;
- existing typed validation remains fail-closed for missing executable/output, traversal, and output-equals-input;
- plan arguments retain the existing local-only `file` protocol whitelist;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path, argv, source URL, or process identifier;
- no FFmpeg process is spawned;
- no external network access occurs.

Verified PR head:
`7eb8834b4dc0bb07ac8e10dde4dea7fa5ffbb0ab`

All 11 required workflows passed for that exact head.

## Next atomic task

**T044 — Add the backend-only controlled FFmpeg remux execution boundary**

Advance one verified T043 `FfmpegRemuxPlan` into a controlled running FFmpeg child through the
existing typed local launcher.

Required boundary:

- input is a verified T043 typed remux plan;
- FFmpeg launch reuses the existing `launch_ffmpeg_remux` boundary;
- no shell command string is constructed;
- no URL/proxy/network input exists;
- launch failures fail closed;
- backend owns the running FFmpeg child until explicit completion or cleanup;
- frontend receives no executable path, input/output filesystem path, argv, source URL, or process identifier;
- no external network access occurs;
- completion/result surfacing remains out of scope.

## Do not do yet

- no completed remux result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

A verified local remux plan can launch FFmpeg through the existing controlled typed process boundary
without shell execution or any network-capable path.
