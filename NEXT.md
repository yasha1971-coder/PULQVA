# NEXT

## Current verified state

T022 is complete.

PULQVA now has a controlled yt-dlp process boundary:

- launcher accepts only `YtDlpMediaRequestPlan`;
- executable and argv come only from the typed plan;
- child is spawned directly with no shell;
- `RunningYtDlp` owns the process handle;
- deterministic stop/wait cleanup exists;
- Windows/Linux tests use local network-free fixtures;
- the exact Tor proxy/base arguments and request arguments are preserved across the process boundary.

Verified PR head:
`be07ecbaba2dd3027b3592ba0a4c191739e9c0e1`

## Active atomic task

**T023 — Prove a real pinned yt-dlp metadata-only request through Tor**

T023 adds an explicit metadata-only mode to `YtDlpMediaRequestPlan`:

- `--skip-download`;
- `--dump-single-json`;
- `--no-playlist`.

The proof uses the actual pinned Arti 2.6.0 and yt-dlp 2026.08.19 binaries, requires
`ReadyTorTransport` before the yt-dlp plan can exist, runs a bounded request through the verified
`socks5h://` route, and rejects any created download output.

Linux must complete the metadata-only request successfully. On GitHub-hosted Windows, Tor readiness
may instead terminate in the already-defined bounded fail-closed path; no yt-dlp process is started
in that case.

## Queued next task

**T024 — Prove one bounded real media download through Tor**

Use the typed process path to download one small fixed sample media object into an isolated output
root through Tor, with an explicit timeout, no FFmpeg, and no direct-network fallback.

## Do not do yet

- no arbitrary user URL execution;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

The pinned real yt-dlp binary can perform metadata extraction only after Tor readiness, the request
is bounded, no media payload is written, and failure never triggers a direct retry.
