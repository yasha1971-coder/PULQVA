# PULQVA Arti sidecar

Pinned upstream binary package: `arti 2.6.0`.

This directory records both the pinned Arti package version and the exact content identity of the
platform executables intended for PULQVA packaging.

## Executable identity contract

`SHA256SUMS` contains exactly one supported identity per packaged platform path:

- `linux-x86_64/arti`
- `windows-x86_64/arti.exe`

The initial identities were captured from PR #59 head
`ddd9be7815dc31bc88f2c405229da5e36b51e2ff`, workflow run `35921760775`:

- Linux job `107387185462`: SHA-256
  `d6bef8db24c6edbeb7c7ee7c91df53e8b37959eb0f838ac9b9d08ab4d716a543`,
  23,619,232 bytes.
- The initial Windows candidate was not byte-reproducible because the MSVC-linked PE/debug
  metadata carried build-time values. It is retained only as historical capture evidence, not as
  the final Windows identity.

The current Windows identity recipe is pinned by `.github/workflows/arti-sidecar-check.yml`:
Arti 2.6.0, Rust 1.91.0 and its bundled rust-lld, Cargo `--locked`, `static-sqlite`,
`/Brepro`, and `/DEBUG:NONE`. CI verifies the actual Windows PE has no CodeView/PDB
identity and retains its reproducible-build marker. Linux's recipe and identity are unchanged.

The older MSVC `/Brepro` identity reproduced on its original runner image but drifted after
the floating MSVC toolset changed. Pinning rust-lld alone left 20 differing debug identity bytes.
Disabling PDB identity resolved that observed drift in two independent builds of PR #61 head
`923b7a48d0cff324347f4cedf98e7c62e7e6ab2a`, workflow run `36030525791`:

- first Windows job `107737813985`, artifact `10823040260`;
- repeated Windows job `107754281056`, artifact `10824592634`;
- both executables: 19,624,960 bytes;
- both SHA-256: `9245c7b5f71391238539bf2d78a492453cae66f978cf8e479039a26f1667f3df`;
- downloaded executables independently hashed and compared byte-for-byte: identical.

The linker SHA-256 in both receipts is
`808749afb43b8a4f708bfc3db6867a74c93d5dc20b18904ce1aa8c6e8fc41dcf`.
Both builds passed compilation, CLI and actual PE metadata checks; they failed only against
the previous allowlist value. This update promotes the independently reproduced value to
`SHA256SUMS`; the promotion commit must pass its own CI before T059 can close.
This is evidence for these builds, not a guarantee across arbitrary future runner images.
CI continues to hash the complete executable and fails closed on any mismatch.

A version string alone is not executable authentication. Changing Arti, toolchain, features, build
recipe, or expected executable bytes requires a dedicated atomic identity update with Windows/Linux
CI evidence.

The sidecar check may execute `arti --version` and `arti help proxy` only as local CLI proof. It
does not bootstrap Tor or make a Tor network connection.
