# NEXT

## CURRENT: T066-B1 actual child environment probe; CI pending

PR71 branch feat/T066-deno-environment. Verified head
488d996a5c4affe62422ffffb2157f87c0cc2588: all 11 triggered workflows green.
Desktop run 36241456647, Linux 108402634096 / Windows 108402634265:
both T066-A policy tests passed. Main merge 10ece61 previously passed 12/12.

B1 adds a bounded native subprocess probe with poisoned intermediate parent and
actual clean child environment assertions. No Deno or network execution, no files
written/deleted by the fixture. Windows SystemRoot is a CI fixture input only.
New exact head and run IDs are saved in the PR launch checkpoint.

ONE next action: inspect exact-head CI and native probe evidence; diagnose red.
After green, T066-B2 owned cache/home/temp lifecycle and trusted native directory
discovery remain. No default activation, release sandbox or completed-product
claim. Historical network causes and positive Windows retrieval remain unresolved.
