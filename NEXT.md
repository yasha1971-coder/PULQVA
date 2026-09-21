# NEXT

## Current verified state

T024 is complete.

PULQVA now proves one real media object across the full typed Tor-only yt-dlp path:

- actual pinned Arti 2.6.0 and yt-dlp 2026.08.19 are used;
- `ReadyTorTransport` is required before the media plan exists;
- source is pinned to one immutable public MP4 object;
- execution is bounded;
- Linux writes exactly one 5,510,872-byte regular artifact;
- Windows remains bounded and fail-closed when Tor readiness is unavailable;
- no FFmpeg or direct fallback is involved.

Verified PR head:
`ea371388699f4fc9347815681c065299c31efe23`

## Active atomic task

**T025 — Add a typed completed-media artifact receipt**

A successful yt-dlp child can now be consumed into `CompletedMediaArtifactReceipt`.

The receipt boundary requires:

- successful child completion before validation;
- the exact output root retained by the launched typed request;
- exactly one regular file;
- no symlinks;
- no lexical parent-directory traversal in the root;
- canonical artifact path contained by the canonical output root;
- non-zero byte size.

The receipt exposes only the canonical artifact path and byte size.

## Queued next task

**T026 — Add a completed download result for UI handoff**

Combine source metadata and `CompletedMediaArtifactReceipt` into a stable typed result that the
future desktop UI can display/download without exposing process internals.

## Do not do yet

- no arbitrary user URL execution;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

Only a successfully completed yt-dlp child can produce a typed receipt for one validated,
non-empty artifact contained inside its explicit output root.
