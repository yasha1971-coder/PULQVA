# T065 — Verified Deno materialization boundary

Status: DONE in owned ephemeral materialization scope; PR70 closeout/merge pending.
Verified implementation: 75f06b16e7f4568ecf026427197f0515893ad306.
All 11 applicable workflows succeeded. No kernel or dependency change.

Fixed Deno 2.9.7 archive and executable size/SHA checks, strict single-member ZIP,
bounded regular-file source snapshot and exclusively owned staging are implemented.
Output is synced and reread, Unix mode 0700 applied, and ownership retained in a
typed receipt. Identity-checked nonrecursive cleanup preserves foreign replacements.

Desktop run 36226635072: Linux job 108361831992 (96 desktop + 9 companion tests),
Windows job 108361832119 (88 desktop + 7 companion tests). Both separately executed
the initially ignored real Deno integration test and FFmpeg compatibility test.

Independent Python PULQVA_DENO_REFERENCE and Rust PULQVA_DENO_COMPAT receipts agree
on both OSes, cleanup=ok. Real proof covers source/ZIP/stage, disk hash/size, native
name, Unix permissions, cleanup, sentinel preservation and missing/directory/wrong-
target rejection. See sidecars/deno/MATERIALIZATION_VERIFICATION.json.
Earlier A/B synthetic failure coverage and native evidence are in PR70 checkpoints.

No binary execution in this proof. No permanent publication, default activation,
live EJS/YouTube, Windows ACL isolation or descendant egress confinement is claimed.
Concurrent path replacement is not atomically prevented; prelaunch revalidation is
required. Environment/cache controls continue in T066. Historical Linux retrieval,
SDK content pinning and positive Windows media proof remain unresolved.
