# NEXT

## Current verified state

T023 is complete.

PULQVA now proves a real metadata-only yt-dlp request through Tor:

- actual pinned Arti 2.6.0 and yt-dlp 2026.08.19 binaries are used;
- `ReadyTorTransport` is required before the yt-dlp plan exists;
- metadata-only mode adds `--skip-download --dump-single-json --no-playlist`;
- the request is bounded;
- successful execution writes no media payload;
- no direct retry/fallback exists;
- Linux completes metadata extraction successfully;
- Windows remains bounded and fail-closed when Tor readiness is unavailable.

Verified PR head:
`ebc5da77ccc203095c102c6dcdf80ea4f2170c9f`

## Active atomic task

**T024 — Prove one bounded real media download through Tor**

The proof uses one fixed public MP4 pinned to an immutable GitHub commit:

- source repository: `mediaelement/mediaelement-files`;
- source commit: `4d21a042353022326071acb0251ab75cd6bae114`;
- source object: `big_buck_bunny.mp4`;
- expected size: 5,510,872 bytes.

The real pinned yt-dlp process is constructed only from `YtDlpMediaRequestPlan`, inherits only the
verified `socks5h://` Tor route, writes into an isolated output root, and is bounded by an explicit
timeout. Linux must produce exactly one media artifact matching the pinned byte size. Windows may
take the existing bounded Tor-readiness fail-closed path.

## Queued next task

**T025 — Add a typed completed-media artifact receipt**

After a successful yt-dlp child exit, validate the isolated output root and return a typed receipt
for the completed regular media artifact without following symlinks or accepting path escape.

## Do not do yet

- no arbitrary user URL execution;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

One real media object crosses the full typed path and is written byte-complete through Tor with no
fallback and no FFmpeg dependency.
