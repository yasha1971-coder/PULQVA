# NEXT

## Current verified state

T019 is complete.

PULQVA now pins the official yt-dlp standalone release 2026.08.19:

- Linux yt-dlp_linux and Windows yt-dlp.exe are pinned by exact SHA-256;
- both binaries passed exact --version verification;
- --ignore-config --help passed on Windows and Linux;
- no media URL or product-runtime media request was used;
- all privacy and continuity checks stayed green.

Verified PR head:
dea983cc546a2239612b6bf592ca49db4f2cb5f7

## Active atomic task

**T020 — Add a typed yt-dlp launch plan gated by ReadyTorTransport**

The pure-data launch plan requires ReadyTorTransport at construction time.

It keeps the executable path explicit and deterministically emits only:

- --ignore-config;
- --proxy;
- the verified socks5h:// route from ReadyTorTransport::proxy_url().

There is no direct-route variant, shell command, process spawn, or media URL in T020.

## Queued next task

**T021 — Add a typed yt-dlp media request plan**

Add typed source/output request data on top of the Tor-gated launch plan while keeping the task
pure-data and non-executing.

## Do not do yet

- no yt-dlp process spawn;
- no media download;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

Future media execution cannot construct its base yt-dlp launch plan without a verified Tor
transport, and deterministic arguments cannot silently inherit ambient yt-dlp configuration.
