# NEXT

## Current verified state

T025 is complete.

PULQVA now has a completion-gated `CompletedMediaArtifactReceipt`:

- only a successful yt-dlp child can enter artifact validation;
- the exact output root is retained by the running typed request;
- exactly one regular non-empty file is required;
- symlinks, parent traversal, path escape, empty output, multiple files, and zero-byte artifacts are rejected;
- the receipt exposes only the canonical artifact path and byte size;
- Windows/Linux filesystem checks are green.

Verified PR head:
`3e03bd96302f7458947c661f38a65a762e5129b3`

## Active atomic task

**T026 — Add a completed download result for UI handoff**

A successful validated completion can now be consumed into `CompletedDownloadResult`.

The result:

- retains the original typed `YtDlpMediaSourceUrl`;
- derives artifact path and byte size only from `CompletedMediaArtifactReceipt`;
- exposes no child-process handle or proxy/Tor internals;
- has deterministic display-facing fields for source URL, canonical artifact path, and byte size;
- adds no network, process, or filesystem side effect during result construction.

## Queued next task

**T027 — Pin and prove the FFmpeg/ffprobe sidecar**

Pin one exact official FFmpeg build source for Windows and Linux and prove local
`ffmpeg -version` / `ffprobe -version` CLI availability before any post-processing is allowed.

## Do not do yet

- no desktop UI implementation;
- no FFmpeg media transformation;
- no AI provider;
- no arbitrary user URL execution;
- no direct-network fallback.

## Success

The future desktop UI can consume one stable typed completed-download value without learning about
process supervision or privacy-route internals.
