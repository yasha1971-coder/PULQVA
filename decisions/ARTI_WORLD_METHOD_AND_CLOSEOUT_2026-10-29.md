# Arti recovery: external method review and fixture closeout
Status: SHA root cause PENDING; fixture correction locally tested, hosted pending.

Primary-source review:
- https://doc.rust-lang.org/cargo/commands/cargo-install.html : packaged lockfile is used with --locked; cargo install configuration discovery is system/user-level rather than normal project discovery. A lock alone does not identify the native SDK.
- https://dev-doc.rust-lang.org/stable/rustc/remap-source-paths.html : path normalization is best-effort; linker/external-tool paths require separate treatment.
- https://reproducible-builds.org/docs/build-path/ : normalize build paths; do not confuse metadata-only changes with changed code/data.
- https://lists.reproducible-builds.org/pipermail/rb-general/2024-December/003592.html : first-hand Threema Windows experience uses fixed Rust plus linker timestamp/PDB-path controls and artifact comparison. Those controls are necessary in that case, not a universal guarantee.
- https://diffoscope.org/ : use structural binary comparison to localize differences rather than repeatedly changing an allowlisted hash. No private/local artifact uploaded to an external comparison service.
- https://arti.torproject.org/guides/compiling-arti/ : official guidance describes compiling Arti; no official prebuilt binary is offered as a shortcut here.

Our evidence: actual .text/.rdata changes, same current output across independent builds, current MSVC-root variation not sufficient, historical SDK equality unknown. Do not claim a broken Arti source or compromised binary from a hash mismatch alone; do not claim security from matching hashes either. Old pin remains unchanged until source/input provenance, reproducibility and relevant validation qualify a successor or restore the former output.

Concrete recovery in this commit: job123663849485 failed removing its own TemporaryDirectory with Windows sharing/access errors after job membership emptied. Add a bounded4s directory-release wait, catching PermissionError only, re-raising persistent failures; preserve native ownership/port checks. Native resources are not proven released merely because a PID/job count is empty. Two deterministic tests exercise eventual release and mandatory failure on persistent lock; all6 existing actual native fixture probes pass locally. Total8 tests PASS/0.408s. No Arti/PULQVA launch, pin/lock/product/kernel/USB change, local installer, merge or D cleanup.

Next: verify hosted fixture recovery at exact new SHA, then localize historical/current SDK and native-object differences with structural binary tools in a trusted isolated builder. Normalize/freeze all actual inputs before any justified pin promotion. Do not repeat the completed MSVC-root experiment unchanged.
