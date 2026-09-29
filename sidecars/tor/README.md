# PULQVA official Tor sidecar

Windows production baseline introduced by ADR-0007.

## Pinned source

- Tor Project Expert Bundle: `15.0.23`
- Reported Tor core: `0.4.9.12`
- Windows archive: `tor-expert-bundle-windows-x86_64-15.0.23.tar.gz`
- Source: Tor Project distribution archive for release 15.0.23
- Product policy: package verified runtime files; do not compile Arti or download Tor on the end-user machine.

`SHA256SUMS` pins both the full source archive and the extracted `tor/tor.exe`.
The archive digest is authoritative for the complete runtime payload, including adjacent
runtime libraries and data files.

## Windows evidence

`WINDOWS_NATIVE_VERIFICATION.json` records the real-machine proof performed on
2026-09-29 with Smart App Control enabled:

1. official archive hash verified;
2. `tor.exe --version` executed;
3. Tor reached bootstrap 100%;
4. SOCKS5 listened on loopback;
5. Tor Project check returned `IsTor=true`;
6. Tor was stopped after the bounded test.

The exit IP is deliberately not retained.

## Boundary

This sidecar does not weaken the privacy architecture. Network consumers still require
the existing `ReadyTorTransport` capability and `socks5h` semantics. Failure to
materialize, launch, bootstrap, or certify the Tor endpoint must fail closed.

The current Linux Arti path is retained until a separate migration proof exists.
