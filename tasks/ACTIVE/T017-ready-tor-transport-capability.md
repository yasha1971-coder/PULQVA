# T017 — Add a readiness-gated Tor transport capability

Parent: PRIVACY FOUNDATION  
Status: ACTIVE

## Goal

Make verified Tor readiness a type-level prerequisite for future external-network adapters.

## Acceptance criteria

- public `ReadyTorTransport` type exists;
- it has no public constructor;
- raw `TorSocksEndpoint` no longer publicly exposes a proxy URL;
- only privacy-crate internal code can mint `ReadyTorTransport`;
- ready capability exposes the verified endpoint and `socks5h` proxy URL;
- no direct/clearnet route type is introduced;
- no sockets, process spawn, Tor bootstrap, or external network activity is added;
- `cargo test --workspace --locked` passes;
- all existing privacy checks remain green.

## Out of scope

- actual readiness verification;
- Tor bootstrap trigger;
- SOCKS/DNS probes;
- external requests;
- dynamic port discovery;
- yt-dlp/FFmpeg;
- AI;
- UI.
