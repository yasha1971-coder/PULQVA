# NEXT

## CURRENT: committed desktop lock / real Deno spawn diagnosis; CI pending

PR71 branch feat/T066-deno-environment. Overall verified anchor remainsb6c7130.
5dbdad0 desktop36258520003 Windows108449766013 / Linux108449766100 compiled
Tauri and passed native tests/archive materialization. Generated locks match
byte-for-byte112174 bytes; SHA2566f84196e590bf00f9a62d887beef61c46782064e277120e3da5435f1f8c687d2.
Committed actual lock, removed generation fallback/exporter. CI requires tracked
lock; all cargo check/test commands --locked. TOML/YAML/hash/diff locally checked.

Real Deno runtime test now fails trial-spawn-failed on both OSes. Added test-only
error kind/raw OS code, no path/environment dump. Writable extraction handles are
only a hypothesis until diagnostics confirm. New head/run IDs in PR checkpoint.

ONE next action: inspect exact-head native spawn code, repair only evidenced
cause while retaining executable identity/cleanup ownership. No blind rerun or
runtime activation. Lock recovery not overall-green until native tests complete.
