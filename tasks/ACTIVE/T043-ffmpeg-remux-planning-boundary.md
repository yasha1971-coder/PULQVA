# T043 — Add the backend-only FFmpeg remux planning boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE  
## Recovery

The implementation head remains previously verified, but T043 is active again because the closeout
head repeatedly failed the Ubuntu real-media proof. Recovery replaces only the external proof fixture
with a small stable Wikimedia Commons media file while preserving the same Tor-gated yt-dlp path.

## Result

Added a backend-only FFmpeg remux planning boundary.

A verified `CompletedDownloadResult` can now be converted into the existing typed
`FfmpegRemuxPlan` using an explicit backend FFmpeg executable path. The remux output is derived from
the validated artifact path as a sibling `pulqva-remux-*.mp4` path without requiring a UTF-8 file
name.

The plan uses explicit `FfmpegRemuxContainer::Mp4` and inherits the existing local-file-only
`file` protocol whitelist. Existing typed validation remains fail-closed for missing executable or
output, traversal, and output-equals-input cases.

Frontend IPC/output is unchanged and exposes no executable path, input/output filesystem path, argv,
source URL, or process identifier. No FFmpeg process is started and no external network access occurs.

## Verification

PR #43 verified head:
`7eb8834b4dc0bb07ac8e10dde4dea7fa5ffbb0ab`

All 11 required workflows passed:

- continuity-guard;
- rust-check;
- desktop-shell-check;
- arti-sidecar-check;
- arti-materialization-check;
- arti-config-contract;
- arti-lifecycle-check;
- tor-readiness-check;
- ytdlp-sidecar-check;
- ytdlp-tor-metadata-check;
- ytdlp-tor-media-check.
