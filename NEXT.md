# NEXT

## Current verified state

T021 is complete.

PULQVA now has a typed, pure-data yt-dlp media request layer:

- media source is explicitly HTTP(S)-only;
- local/file and unsupported schemes are rejected;
- authority and output root are explicit;
- request construction requires the Tor-gated `YtDlpLaunchPlan`;
- Tor base arguments are preserved unchanged;
- request argv is deterministic;
- no process, shell command, or direct-network fallback exists in the planning layer.

Verified PR head:
`369bb883554a1bc33e15e34b2afbbfa7956c758f`

## Active atomic task

**T022 — Add a controlled yt-dlp child-process launcher**

The first yt-dlp process side effect is now behind `YtDlpMediaRequestPlan`.

The proof uses local fixture executables only:

- launcher accepts only the typed request plan;
- executable and argv come only from that plan;
- child is spawned directly with no shell;
- `RunningYtDlp` owns the child handle;
- deterministic stop/wait cleanup exists;
- a local fake Arti/SOCKS fixture mints the ready transport without external network;
- the yt-dlp fixture records the exact argv and performs no network.

## Queued next task

**T023 — Prove a real pinned yt-dlp metadata-only request through Tor**

Use the actual pinned yt-dlp binary only after `ReadyTorTransport` exists, perform a bounded
metadata-only request through Tor, and prove there is no direct-network fallback or media download.

## Do not do yet

- no full media download;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

A fully typed request can cross the process boundary with exact Tor-only argv and deterministic
process ownership while the test remains network-free.
