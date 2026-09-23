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
- Windows job `107387185755`: SHA-256
  `437c9391d7c7d70b8d298c8e6edd5310cb6e0511204c872c3a6c84fbc79e5bba`,
  19,759,616 bytes.

The production identity recipe is pinned by `.github/workflows/arti-sidecar-check.yml`: Arti 2.6.0,
Rust 1.91.0, Cargo `--locked`, and `static-sqlite` on Windows. CI rebuilds the executable using
that recipe, computes the bytes' SHA-256, and fails closed unless it matches the committed platform
identity.

A version string alone is not executable authentication. Changing Arti, toolchain, features, build
recipe, or expected executable bytes requires a dedicated atomic identity update with Windows/Linux
CI evidence.

The sidecar check may execute `arti --version` and `arti help proxy` only as local CLI proof. It
does not bootstrap Tor or make a Tor network connection.
