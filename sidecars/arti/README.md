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

The production identity recipe is pinned by `.github/workflows/arti-sidecar-check.yml`: Arti 2.6.0,
Rust 1.91.0, Cargo `--locked`, `static-sqlite` on Windows, and MSVC linker option `/Brepro` on
Windows. CI rebuilds the executable using that recipe, computes the bytes' SHA-256, and fails closed
unless it matches the committed platform identity.

The final Windows identity was proven byte-identical across two unchanged `/Brepro` CI runs on
PR #59 head `ad17896b52535122ef5766dcd4ae0960f2b19df7`:

- first Windows job `107436310670`: SHA-256
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`,
  19,760,128 bytes;
- unchanged rerun job `107449565233`: the exact same SHA-256
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`,
  19,760,128 bytes.

That reproduced value is the committed Windows package-source identity in `SHA256SUMS`.

A version string alone is not executable authentication. Changing Arti, toolchain, features, build
recipe, or expected executable bytes requires a dedicated atomic identity update with Windows/Linux
CI evidence.

The sidecar check may execute `arti --version` and `arti help proxy` only as local CLI proof. It
does not bootstrap Tor or make a Tor network connection.
