# T020 — Add a typed yt-dlp launch plan gated by ReadyTorTransport

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added pure-data `YtDlpLaunchPlan`.

Construction requires `ReadyTorTransport`; executable path is explicit; arguments are
deterministic and begin with `--ignore-config --proxy <verified socks5h route>`.

There is no direct/clearnet route variant, shell command string, process spawn, or media URL.

## Verification

PR #19 verified head:
`72410559872be701a282e9eb385743c3fc323154`

All existing checks passed.
