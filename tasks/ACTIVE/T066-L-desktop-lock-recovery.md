# T066-L — restore desktop dependency reproducibility

Status: ACTIVE. Native CI pending; no local Cargo available.
Failure67fd587 desktop36257308177 Linux108446429179 / Windows108446429309
compiled tauri2.11.6 with incompatible transitive releases. Runtime tests not reached.
Successful b6c7130 Linux108443175567 resolved runtime2.11.3, runtime-wry2.11.4,
macros/codegen2.6.3, utils2.9.3. Failed run resolved2.12.0/2.12.0/2.7.0/2.7.0/2.10.0.
No desktop Cargo.lock was committed; CI resolved fresh dependencies each run.

Recovery: exact constraints on the five known-good versions, added defaults
disabled to preserve feature selection by existing Tauri edges. Cargo generates
the lock on CI, validates seven Tauri pins, uploads it BEFORE compilation and
emits compressed text plus SHA256 for connector retrieval. All check/test commands
now use --locked within that run. Both OSes generate independently for comparison.

This is transitional, NOT full cross-run reproducibility yet. Next retrieve both
real Cargo-generated locks, compare them, commit the reviewed complete lock,
remove generation fallback/text exporter, require committed lock plus --locked.
Never invent checksums or regenerate blindly on each future run. Tests remain
strict; no Tauri-wide upgrade or Deno activation. Parent T066 stays ACTIVE.
