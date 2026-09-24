# T057 — Establish packaged Arti executable content identity

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-24

## Result

Arti now has an explicit backend-verifiable content identity for the exact packaged executable on
both supported targets.

Committed identities:

- Linux x86_64 / `arti`:
  `d6bef8db24c6edbeb7c7ee7c91df53e8b37959eb0f838ac9b9d08ab4d716a543`,
  23,619,232 bytes.
- Windows x86_64 / `arti.exe`:
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`,
  19,760,128 bytes.

The Windows recipe required MSVC linker option `/Brepro`. Before `/Brepro`, PE/debug timestamps
made nominally identical builds produce different hashes. Two unchanged `/Brepro` Windows runs
produced byte-identical output, and a fresh committed-identity CI run reproduced the same digest.

Backend behavior now:

- carries a platform-specific pinned Arti SHA-256 in `BundledSidecarIdentity`;
- rejects missing, duplicate, malformed, and wrong-platform Arti identity metadata;
- validates packaged Arti source bytes against the pinned digest;
- no longer treats a version string alone as executable authentication.

No Arti runtime materialization, chmod, Tor launch change, frontend filesystem authority, bundle
activation, installer, external provider, AI provider, or direct-network fallback was introduced.

## Verification

Verified implementation head:
`233b4cc291066404c3b0119b7e9664981cfc6b10`

All 11 required workflows passed for that exact head.

The decisive fresh Windows job `107480733952` emitted:

`actual_sha256=14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`

`expected_sha256=14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`

`byte_size=19760128`.
