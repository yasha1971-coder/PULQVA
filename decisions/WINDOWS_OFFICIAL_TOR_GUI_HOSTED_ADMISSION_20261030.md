# Windows official Tor GUI integration: hosted admission
The Windows GUI now uses the owned official Tor adapter below the existing CommonsHttpsTransport. Linux retains Arti. All old packaged-resource checks remain; Tor is not renamed to Arti.

Local desktop testing was blocked before executing serde's build script by Windows Application Control error4551. This is a build-host restriction, not a test failure or Tor diagnosis. Do not disable protection or repeat the denied executable. Hosted CI is the authorized build route.

The new Windows workflow tests the complete privacy crate and desktop boundaries without skips or symlink policy changes, then builds the exact PR head. Its artifact binds app SHA256 to source SHA, toolchain, run and unchanged locks/kernel digests. It is an app artifact, not a self-contained package or GUI E2E acceptance.

Next: after observed green CI, verify artifact digest/receipt and stage a fresh C candidate with the already authenticated legacy sidecars plus qualified official Tor16.0 resources. Then real GUI query, explicit choice, small permitted WebM, independent SHA readback and owned Job/PID/port CLEANUP; repeat separately. No prototype execution occurred during this phase.

Qualified Tor archive SHA256: 2ee123e9167318a3742778ff704e75f01d4780f05a14522d451fe011011077c4.
Signature primary: EF6E286DDA85EA2A4BA7DE684E2C6E8793298290.
Member identities remain hard-coded and fail closed in official_tor_host.rs.
