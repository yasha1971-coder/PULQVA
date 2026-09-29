# T068-B0 — HTTP/TLS candidate and real lock acquisition

Date: 2026-09-27. Status: candidate native CI pending, not shipping adoption.
Purpose: unblock real Commons HTTPS discovery without repeating the T066 unlocked
transitive-dependency failure or inventing a Cargo-generated lock.

## Observed starting point
PR73 b223f3a8825256c002b91bffd8508ecb9ba275be has 11 successful associated workflows.
No production HTTPS executor exists yet. Local Cargo/rustc are absent and a bounded
container clone failed DNS. Existing production/workspace manifests and locks stay
unchanged while an isolated Windows/Linux CI copy resolves the candidate additions.

## Entry review: exact source, not version-name assumptions
The actual reqwest v0.13.5 Cargo.toml (blob 8123a6ebec4f16de6820ea00a6575ed00108b3a1)
requires Rust 1.85.0, below the project's pinned 1.91.0, but transitive compatibility
still needs native Cargo resolution. Its features differ from older 0.12 examples:
Rustls defaults to an aws-lc provider and a platform verifier. Select explicit
rustls-no-provider + socks, with separately configured Rustls/ring and embedded
Mozilla roots. This is ordinary maintained TLS, not a custom cryptographic stack.
The explicit configuration avoids silently inheriting OS trust/environment at this
boundary. It does not prove OS egress confinement, current revocation checking or
hostile-machine security. The platform-verifier dependency may still be present in
the resolved graph, but the candidate client is given a preconfigured TLS backend.

Exact candidates, all subject to native validation:
- reqwest 0.13.5: default features off, rustls-no-provider + socks.
- rustls 0.23.43: default features off, ring + std + tls12.
- webpki-roots 1.0.9: compiled-in root bundle.
- tokio 1.53.1: rt + net + time only; existing dependency edges may add features.

No HTTP/3, cookies, system proxy, auto decompression or generic URL/client injection
is needed for this one endpoint. Explicitly disable proxy exclusions, redirects,
protocol retry policy, referrer propagation and TLS key logging. Use async chunk
reading under one outer deadline when implementing B rather than resetting a new
time budget for each read. The candidate only compiles that body path for now.

## Root-store tradeoff
The root-bundle documentation recommends platform verification for software which
cannot update readily; bundled roots need deliberate package/security updates.
Before release, establish an update process for those roots and review revocation
requirements. No runtime fetch of certificates/support code is introduced here.
Do not call a static bundle universally superior or claim native trust is insecure.

## Native test scope and lock adoption
The probe builds the intended client APIs. A bounded local SOCKS server asserts
ATYP=DOMAIN with commons.wikimedia.org:443, returns a ruleset refusal and records
any second CONNECT during its observation window. A fresh child repeats this with
NO_PROXY=* and poisoned proxy variables; no global process environment mutation.
The fake SOCKS server opens no external connections. This is not an OS-wide egress
measurement. No TLS handshake or successful HTTP response is exercised in this phase.
Full TLS, redirect, truncation, chunked-limit, timeout/cancellation, dead-Tor and
live positive/negative tests remain mandatory for B/C, not implied by a green probe.

Cargo runs only on a fresh copy of tracked regular repository files. Use
cargo update --workspace, then reject any disappearance of an existing registry
name/version/source/checksum identity. New dependencies are pinned, compiled with
--locked and exported with the full lock SHA-256, byte count, source checkout SHA,
manifest and feature graph. Retrieve BOTH native artifacts and compare before
adoption. Generation is one-off preparation, not a per-release unlocked fallback.
The accepted lock must later be committed verbatim and the generator retired.
A candidate receipt is written before tests; it is not a test-success receipt.

## Exit review
Cargo's official --workspace documentation confirms that packages already present
in the lock are retained while missing dependencies may be resolved. --locked then
fails rather than silently re-resolve. Proxy::all plus disabled system proxy and
explicitly absent exclusions is the intended route, but the actual pinned native
probe is the acceptance authority for API compatibility. Mutable docs showed mixed
0.13.4/0.13.5 cached versions; use the inspected v0.13.5 source and observed native
results rather than representing every latest-doc page as exact-version evidence.
No dependency has been adopted, no live result or E2E percentage is claimed.

## Primary sources actually consulted
- https://github.com/seanmonstar/reqwest/blob/v0.13.5/Cargo.toml
- https://docs.rs/reqwest/latest/reqwest/
- https://docs.rs/reqwest/latest/reqwest/blocking/struct.ClientBuilder.html
- https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html
- https://docs.rs/rustls/latest/rustls/
- https://docs.rs/webpki-roots/latest/webpki_roots/
- https://docs.rs/tokio/latest/tokio/
- https://doc.rust-lang.org/cargo/commands/cargo-update.html

Targeted review only; not an exhaustive industry survey or independent audit.
