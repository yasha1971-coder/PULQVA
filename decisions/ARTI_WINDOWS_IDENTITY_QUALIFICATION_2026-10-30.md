# Windows Arti executable identity qualification — 2026-10-30

Owner-authorized identity migration for PR #100, parent 122f95df6ffd268910c05621cf4f4ab1c3d8989a.
Promote Windows SHA-256 ab592becfcd446742e7af59aaed20720b391f26b770fa34a0065259448c870ad (19,624,960 bytes); retain Linux identity.
This candidate has independently matched in jobs 123635868309, 123663848878 and
123663848481. Source is crates.io Arti 2.6.0, checksum
6c2d88fb27c29401cac3d384ef7c3998915192b9e08fda9823e02ca7d578be2e; published lock checksum
939d35880fcb517cce8de9588d9f4b1cc2eb1aa548b88f0a1569da67ba1a0567. Rust 1.91.0, rust-lld
808749afb43b8a4f708bfc3db6867a74c93d5dc20b18904ce1aa8c6e8fc41dcf,
static-sqlite, /Brepro and /DEBUG:NONE are unchanged. The 64-file compiler gate stays.

The old 9245c7b5f71391238539bf2d78a492453cae66f978cf8e479039a26f1667f3df came from source 923b7a48d0cff324347f4cedf98e7c62e7e6ab2a,
run 36030525791, runner image 20260922.246.2. Current image is 20261021.278.1.
Differences include executable code and read-only data, not only timestamps.
First native function-boundary drift was localized to SQLite; current SQLite object
78baee9501df02656726f9d37fb8af49ee6465c8589d46c94c502bbb20a7800e
was independently replayed in run 40951808158. The exact changed historical SDK
input is unproven; do not describe it as established or claim a fully hermetic build.

Known-advisory review is recorded in sidecars/arti/WINDOWS_IDENTITY_QUALIFICATION.json:
722 registry packages queried; 448 observed compiled packages; 19 advisory matches
classified by target, actual callers and features. ctutils masks are unused by
universal-hash byte-array CtEq. zstd optional legacy/experimental/seekable/threading
paths are disabled. RSA decryption oracle is absent. block-buffer 0.10.4 has a
separate cursor field, unlike the reported inline-cursor mechanism. paste maintenance
debt remains explicit. This is scoped applicability evidence, not universal safety.

CI now enforces the reviewed source/lock and actual compiled zstd feature exclusions,
in addition to the unchanged complete executable SHA, CLI, PE and lifecycle gates.
The completed runner/restored comparison arm is removed; the authenticated restored
production arm remains. No kernel, State/NEXT, application code or USB files change.

Checkpoint: pin promotion prepared; exact new-head CI PENDING. One checking wave
only. Observe that SHA and preserve its run/job/artifact identities. Do not merge,
repeat closed archaeology, start MVP/runtime or resume D cleanup.
