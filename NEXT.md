# NEXT

## Current verified state

T020 is complete.

PULQVA now has a pure-data yt-dlp launch plan that cannot be constructed without
`ReadyTorTransport`:

- executable path is explicit;
- argument order is deterministic;
- `--ignore-config` prevents ambient yt-dlp configuration from changing routing;
- `--proxy` comes only from the verified `socks5h://...` transport capability;
- no direct-route variant, shell command, process spawn, or media URL exists in the base plan.

Verified PR head:
`72410559872be701a282e9eb385743c3fc323154`

## Active atomic task

**T021 — Add a typed yt-dlp media request plan**

The pure-data request layer adds:

- validated HTTP(S)-only media source input;
- explicit output root;
- construction on top of `YtDlpLaunchPlan`;
- deterministic request arguments appended after the unchanged Tor-only base arguments.

Local files, stdin-like untyped input, unsupported URL schemes, empty authorities, and empty output
roots are rejected before any process exists.

## Queued next task

**T022 — Add a controlled yt-dlp child-process launcher**

Introduce the process boundary behind `YtDlpMediaRequestPlan` and prove exact argv/process
lifecycle with a local network-free fixture before any real media request is allowed.

## Do not do yet

- no real media download;
- no FFmpeg;
- no AI provider;
- no UI;
- no direct-network fallback.

## Success

A media request is typed and deterministic before execution, while its inherited base route remains
the verified Tor-only yt-dlp launch plan.
