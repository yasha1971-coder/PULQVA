# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE.

## Atomic outcome
Through the existing T067 coordinator, prove one bounded real external path:
natural-language request -> at least two real candidates -> explicit selection ->
real downloaded file -> typed receipt, with all external traffic privacy-gated.

## Fixture constraints
- Stable public source, small payload, no account/API key.
- Avoid YouTube/JS challenge for this generic first-live proof.
- No frontend network and no new generic HTTP escape hatch.
- Remote DNS remains in the Tor path.
- Failure to establish ReadyTorTransport produces no external request.
- Download is bounded by time and output size and isolated to a test-owned directory.
- Receipt/file content must be objectively verifiable.
- Evidence must distinguish Linux/Windows positive retrieval from fail-closed-only results.

## Windows transport prerequisite — verified 2026-09-29

Real Windows execution removed the transport blocker without weakening platform security:

- Rust/Cargo 1.91.0 executed normally.
- Smart App Control was in ENFORCE mode.
- Local `cargo install arti 2.6.0` was blocked by Windows Code Integrity event 3077
  when Cargo attempted to execute a generated build-script executable.
- Smart App Control was not disabled.
- Official Tor Project Expert Bundle `15.0.23` was downloaded and its source archive
  matched SHA-256
  `231dad6b9cb401a54c260db7046965ef04e4f72ff071b140d423fb5da281ab1e`.
- Extracted `tor.exe` matched SHA-256
  `60c45b01938c799862e511a9a5bab12f959a819c6264a24502edc342165f570c`
  and reported Tor `0.4.9.12`.
- The executable ran under the existing Windows security policy.
- Tor reached `Bootstrapped 100% (done): Done`.
- SOCKS5 listened on `127.0.0.1:19050`.
- A request through that SOCKS endpoint to the Tor Project check API returned
  `IsTor=true`.
- The bounded test process was stopped afterwards.

The exit IP is intentionally omitted. See ADR-0007 and
`sidecars/tor/WINDOWS_NATIVE_VERIFICATION.json`.

This proves the Windows Tor transport prerequisite only. It does **not** satisfy
the T068 request -> choices -> selected file acceptance by itself.

## Current implementation decision

For Windows, integrate the pinned official Tor Expert Bundle beneath the existing
privacy boundary. Do not compile Arti on the end-user machine and do not silently
download Tor at application runtime.

Keep the existing `ReadyTorTransport` capability as the network authorization
boundary. The Windows little-t lifecycle/materialization work must converge on that
same capability rather than creating a second networking path.

The current Linux Arti path remains unchanged until separately migrated.

## Acceptance
- Same coordinator/interfaces verified by T067; no parallel demo-only orchestration.
- >=2 real candidates are obtained externally through the privacy boundary.
- Explicit candidate selection is validated.
- Selected content is downloaded through the existing Tor-gated retrieval adapter.
- File exists, is nonempty and matches fixture-specific validation.
- Tor-unavailable negative case fails closed.
- Exact-head CI records platform-specific evidence.

## Next atomic implementation step
1. Add Windows official-Tor runtime materialization/launch beneath the existing privacy layer.
2. Certify it into the existing `ReadyTorTransport` capability.
3. Use that transport with the unchanged T067 coordinator and a stable non-YouTube fixture.
4. Complete one real request -> >=2 choices -> explicit selection -> file -> receipt path.
5. Keep the negative Tor-unavailable case fail-closed.

## Out of scope
YouTube EJS/POT/SABR reliability, UI, Autopilot, provider expansion,
absolute anonymity/sandbox claims, and a claim that the final clean-machine release
package is already verified.
