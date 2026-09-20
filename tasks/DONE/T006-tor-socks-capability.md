# T006 — Add fail-closed Tor SOCKS capability boundary

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Created dependency-free `crates/pulqva-privacy` with `TorSocksEndpoint`.

Properties:

- loopback host fixed to `127.0.0.1`;
- port 0 rejected;
- valid port readable;
- deterministic `socks5h://127.0.0.1:<port>` representation;
- no Direct/Clearnet route representation;
- no socket or network activity.

## Verification

PR #5 verified head commit:
`61fe501443c4883ea6b87a0760f2d72cf4836aa3`

GitHub Actions:

- continuity-guard: success;
- rust-check / `cargo test --workspace --locked`: success.
