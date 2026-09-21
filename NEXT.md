# NEXT

## Current verified state

T026 is complete.

PULQVA now has a stable `CompletedDownloadResult` for the future desktop/UI boundary:

- it is produced only after successful yt-dlp child completion and validated artifact receipt creation;
- the original typed `YtDlpMediaSourceUrl` is retained;
- canonical artifact path and byte size come only from `CompletedMediaArtifactReceipt`;
- process handles and Tor/proxy internals are not exposed;
- deterministic display-facing fields are available without adding network, process, or filesystem side effects.

Verified PR head:
`e25ecbee08e21cfb82b52514980423ed9904a1e1`

## Active atomic task

**T027 — Pin and prove the FFmpeg/ffprobe sidecar**

T027 pins the exact upstream FFmpeg source identity and one dated cross-platform static build
snapshot:

- upstream source commit: `a5923073bfd8f25b7300d93af3f8e690174ebd30`;
- version marker: `n9.0.2-3-ga5923073bf`;
- build snapshot: `BtbN/FFmpeg-Builds` tag `autobuild-2026-09-20-13-11`;
- Linux and Windows archive SHA-256 digests are frozen in-repo;
- archives are verified before extraction or execution;
- CI executes only local `ffmpeg -version` and `ffprobe -version`;
- no media URL, transform, remux, transcode, or runtime network route is introduced.

The source proof explicitly records that FFmpeg publishes the upstream source while BtbN provides the
cross-platform static binary build snapshot.

## Queued next task

**T028 — Add a typed local FFmpeg remux plan**

Create a pure-data FFmpeg remux plan that can take input only from `CompletedDownloadResult`,
chooses an explicit local output path/container, and produces deterministic local-only argv without
spawning FFmpeg.

## Do not do yet

- no FFmpeg process execution or media transformation;
- no desktop UI implementation;
- no AI provider;
- no arbitrary user URL execution;
- no direct-network fallback.

## Success

Both Windows and Linux prove the exact verified FFmpeg/ffprobe sidecar CLI before PULQVA is allowed
to introduce any FFmpeg media-processing side effect.
