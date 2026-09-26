# NEXT

## CURRENT: T065-B source snapshot and owned stage pending CI

PR70 branch feat/T065-deno-materialization. Last verified head:
a99eb1b023f04aa5fe011651b36f6866746ad526; all 11 workflows green. Desktop run
36221425648: Linux job 108347294393 and Windows 108347294486 each ran all five
deno_zip tests successfully. T064 is DONE; T065 remains ACTIVE.

B adds a bounded regular-file snapshot authenticated against fixed archive size/SHA,
then extraction into an exclusively created owned directory/file. Stage receipt keeps
ownership alive, syncs and rereads disk contents, sets Unix mode 0700, and removes
only matching owned paths on drop. Foreign replacements or unexpected files are
preserved; cleanup is nonrecursive. Exact B head/run IDs are in PR70 checkpoint.
Local continuity/diff checks passed; Rust unavailable locally, native CI pending.

ONE next action: inspect exact B head CI and diagnose red. After B passes, implement
T065-C: real pinned archives through Rust source/ZIP/stage adapters on both OSes,
checking output receipt, permissions, cleanup and independent extracted hash.
No docs-only closeout or merge before the real artifact proof.

Stages are ephemeral, not global runtime publication. No default Deno activation,
network download, environment/cache/egress confinement or YouTube proof. Identity
checks are not an atomic defense against hostile concurrent path replacement;
prelaunch revalidation remains necessary. Windows ACL isolation is not claimed.
Historical Linux failures, SDK content pinning and positive Windows live media
limitations remain unchanged.
