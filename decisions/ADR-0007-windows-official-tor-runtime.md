# ADR-0007 — Use the official Tor Expert Bundle for the Windows Tor sidecar

Status: Accepted  
Date: 2026-09-29

## Context

PULQVA requires Tor-by-default, fail-closed external networking, remote DNS through the
privacy path, and a zero-configuration user package.

ADR-0004 selected Arti 2.6.0 as the Tor sidecar and established a reproducible Windows
build in GitHub Actions. On a real Windows host with Smart App Control in ENFORCE mode,
Rust 1.91.0 and Cargo 1.91.0 executed correctly, but a local `cargo install arti` could
not complete because Windows Code Integrity blocked Cargo-generated build-script
executables. Event 3077 recorded the blocked `syn` build script. Disabling Windows
security is not an acceptable product requirement.

The same host then downloaded the official Tor Project Windows x86_64 Expert Bundle,
verified its published archive SHA-256, extracted it, executed `tor.exe` without
weakening Smart App Control, bootstrapped Tor to 100%, exposed a loopback SOCKS5
listener, and received `IsTor=true` from the Tor Project check endpoint.

## Decision

For Windows, use the official Tor Project Expert Bundle as the production Tor sidecar
baseline instead of building Arti on the end-user machine.

Pinned Windows baseline:

- Expert Bundle release: `15.0.23`;
- Tor core reported by `tor.exe --version`: `0.4.9.12`;
- archive: `tor-expert-bundle-windows-x86_64-15.0.23.tar.gz`;
- archive SHA-256:
  `231dad6b9cb401a54c260db7046965ef04e4f72ff071b140d423fb5da281ab1e`;
- extracted `tor/tor.exe` SHA-256:
  `60c45b01938c799862e511a9a5bab12f959a819c6264a24502edc342165f570c`.

The release package must contain already verified Tor runtime files. PULQVA must not
compile Arti or silently download Tor at application runtime.

The existing `ReadyTorTransport` capability remains the privacy boundary. Windows
lifecycle/materialization code may change beneath that boundary, but network adapters
must still receive only a readiness-certified SOCKS endpoint and must retain
`socks5h` remote DNS behavior.

ADR-0004 remains historical and remains the currently verified Linux Arti path. This
ADR supersedes ADR-0004 for the Windows production sidecar only. A Linux migration to
little-t Tor requires separate evidence and is not implied here.

## Real-machine evidence

Observed on 2026-09-29:

- Smart App Control: `ENFORCE`;
- local Arti build: blocked by Windows Code Integrity event 3077;
- official Expert Bundle archive bytes: `22,432,027`;
- archive SHA-256 matched the pinned digest;
- extracted `tor.exe` bytes: `10,222,592`;
- `tor.exe --version`: `Tor version 0.4.9.12`;
- Tor bootstrap: `Bootstrapped 100% (done): Done`;
- SOCKS listener: `127.0.0.1:19050`;
- Tor Project check response: `IsTor=true`;
- test Tor process stopped cleanly after verification.

The observed exit IP is intentionally not retained because it is not needed for the
contract.

This is evidence for the tested Windows machine and pinned bytes, not a claim that
every Windows policy configuration will allow the executable.

## Security consequences

- Do not disable Smart App Control, WDAC, Defender, or other platform protections.
- Verify the pinned source archive before extracting or packaging it.
- Verify packaged runtime identity before launch.
- Keep the SOCKS listener on loopback.
- Preserve fail-closed behavior: no Tor readiness means no external request.
- Preserve remote hostname resolution through SOCKS; no host DNS fallback.
- No runtime download of remote executable components.
- No absolute-anonymity claim.

## Implementation consequence

T068 is **not complete** merely because the transport bootstrapped. The next atomic
implementation step is to materialize and launch the pinned Windows Tor runtime through
the existing privacy/backend boundary, obtain `ReadyTorTransport`, and then complete
the existing T068 acceptance path:

`natural-language request -> >=2 real choices -> explicit selection -> real file -> receipt`.

