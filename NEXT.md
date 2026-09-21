# NEXT

## Current verified state

T018 is complete.

PULQVA now has a bounded, fail-closed Tor readiness gate:

- only the privacy layer can mint `ReadyTorTransport`;
- bootstrap is activated explicitly from the deferred state;
- readiness probes only the local Tor SOCKS listener;
- the external target is passed as a SOCKS5 domain name, so DNS stays on the Tor side;
- no direct/clearnet fallback exists;
- Linux proved a ready Tor route with the pinned Arti 2.6.0 sidecar;
- Windows proved bounded fail-closed behavior on the hosted runner when Tor readiness timed out;
- all existing checks are green.

Verified PR head:
`490e6467ac2f56d69dcfdcb436775e216ebf3859`

## Active atomic task

**T019 — Pin and prove the yt-dlp standalone sidecar**

The official yt-dlp release observed on 2026-09-21 is pinned to `2026.08.19`.

T019 verifies the official Linux and Windows standalone release assets by exact SHA-256, then runs
only `--version` and `--help`. No media URL is supplied and PULQVA performs no media download.

## Queued next task

**T020 — Add a typed yt-dlp launch plan gated by ReadyTorTransport**

Model the future yt-dlp invocation as typed data. The plan must require a verified
`ReadyTorTransport` and must render a Tor-only proxy argument with no direct-network alternative.
No yt-dlp process or media request will be started in T020.

## Do not do yet

- no media URL request;
- no yt-dlp process from product runtime;
- no FFmpeg integration;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

The exact official yt-dlp standalone binaries are pinned and reproducibly verified on Windows and
Linux before any media-network behavior is introduced.
