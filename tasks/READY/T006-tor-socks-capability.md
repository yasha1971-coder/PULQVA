# T006 — Add fail-closed Tor SOCKS capability boundary

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Introduce a minimal dependency-free privacy capability that future networked adapters must
receive instead of inventing their own direct route.

## Contract

Create `crates/pulqva-privacy` with `TorSocksEndpoint`:

- host is fixed to `127.0.0.1`;
- port must be non-zero;
- proxy representation is `socks5h://127.0.0.1:<port>`;
- `socks5h` is intentional so DNS resolution remains on the proxy side;
- there is no Direct/Clearnet enum variant or fallback representation.

## Acceptance criteria

- port 0 is rejected;
- valid port is readable;
- proxy URL is deterministic and uses `socks5h`;
- crate introduces no external dependency;
- task opens no socket and performs no network request;
- no direct-network route exists;
- `cargo test --workspace --locked` passes;
- continuity guard remains green.

## Out of scope

- Arti or Tor bootstrap;
- reachability/health checks;
- HTTP clients;
- yt-dlp;
- search/download execution;
- UI;
- AI.
