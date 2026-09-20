# NEXT

## Current verified state

T005 is complete.

`pulqva-core` now contains provider-neutral boundaries for both sides of search:

- `SearchIntent`: validated natural-language request;
- `SearchCandidate`: validated human-visible title + opaque backend locator.

The candidate locator deliberately has no URL or command semantics in core.

GitHub Actions verified continuity and `cargo test --workspace --locked` on commit
`e14ee06114e98a14cfe6d60b48c0c040847faa5b`.

## Next atomic task

**T006 — Add fail-closed Tor SOCKS capability boundary**

Create a dependency-free `pulqva-privacy` crate with a `TorSocksEndpoint` value representing
only a local Tor SOCKS endpoint.

The value must:

- use fixed loopback host `127.0.0.1`;
- require a non-zero port;
- render a `socks5h://` proxy URL so name resolution stays inside the proxy path;
- expose no Direct/Clearnet alternative.

This task defines capability data only. It does not bootstrap Tor or open sockets.

## Do not do yet

- no Arti dependency/bootstrap;
- no sockets or HTTP;
- no yt-dlp;
- no AI provider;
- no UI;
- no search/download execution;
- no direct-network mode.

## Success

The privacy crate is dependency-free, invalid port 0 is rejected, a valid endpoint renders the
expected `socks5h://127.0.0.1:<port>` value, no direct route exists, and all CI remains green.
