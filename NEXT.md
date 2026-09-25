# NEXT

## CURRENT: T063R independent clean build and invocation evidence

First pinned trial head 96419d7649867a0ea872112d37e2de9aa08395f4 passed all
10 triggered workflows. Windows job 108164756494, run 36163260806, passed
compiler authentication and matched approved Arti SHA256 9245c7b5...667f3df.
Artifacts: executable 10877001743, diagnostics 10877275847.

cc debug commands were not exposed in the normal Cargo success log. Preserve
liblzma-sys build-script output in the diagnostics artifact on this fresh build.
Also correct the receipt's cosmetic file-count formatting (all hashes were
checked by the loop). No compiler, build flags, or approved identity changes.

ONE next action: observe this exact head's CI, inspect preserved liblzma-sys
output for the actual cl.exe invocation compiling lzma2_decoder.c, and compare
its executable byte-for-byte with artifact 10877001743. No merge until evidence
is sufficient. SDK content remains unpinned; T064 remains blocked.

Earlier sections below are historical context.

## LATEST: T063R explicit native compiler trial

Diagnostic head 97e84ca5 passed all 10 triggered checks; map artifact 10875553956
resolved changed code to liblzma-sys 0.4.7 lzma2_decode. PR #67 checkpoint
5836033697 records evidence. Trial now selects MSVC 14.51.36231 explicitly for
target C/C++/archiver, authenticates 64 compiler-directory EXE/DLL files from
that artifact, and selects SDK 10.0.26100.0 include/library directories.
SDK headers/libraries are NOT content-pinned: no hermetic-build claim.

ONE next action: inspect exact-head Windows CI, verify the selected compiler
receipt and actual cc invocation plus unchanged executable hash. If successful,
require an independent clean build before claiming reproducibility recovery.
Unavailable or changed compiler inputs must fail without fallback. PR remains
draft; T064 blocked. No new executable identity is approved.

Earlier diagnostic-launch instructions below are superseded by this section.

## OVERRIDE: T063R Windows Arti identity recovery — diagnostic launch

PR #66 merged as main 900cb164acb5734ef9d0a3b2bb04450f95c3f827, now 10/11 green.
Failure: run 36155156130 Windows job 108137774946. PR #66 comment 5835789784
preserves comparison: 480 bytes differ, 471 in .text; function RVA 0xdb0c40.
Good artifact 10872482256; failed artifact 10873048740. VS versions differ;
precise native input cause is unproven. No trusted SHA256 changes.

Branch recovery/T063R-arti-native-diagnostics records a linker map and available
native input hashes. Inventory alone does not prove selection. ONE next action:
observe exact-head CI and inspect arti-windows-build-diagnostics to resolve the
function/library before preparing input-pinning repair. Head/PR/run IDs go in
the PR launch checkpoint. Do not merge diagnostic work as a completed fix.
T064 blocked until recovery; a green rerun alone cannot close reproducibility.

The following is historical pre-merge context, superseded by this recovery:

## CURRENT: T063 local execution verification complete; closeout CI/merge pending

PR #66 branch task/T063-deno-restricted-execution.
Verified head c8900e98199def05a5dc2d029b0c8c025a6a0a7e: all 11 workflows green.
Run 36142395574: Linux job 108095142055, Windows job 108095142491.
Full receipts preserved in RESTRICTED_EXECUTION_VERIFICATION.json.

T063 DONE within local fixture scope. Computation and all explicit denials passed.
Linux has a loopback-only namespace and unprivileged runtime. Windows effective
exact-program outbound Block was verified and its removal confirmed.
Policy evidence is not packet capture, DNS-service isolation or an app sandbox.

ONE next action: observe PR #66 closeout exact-head checks; diagnose red or mark
ready and merge. Do not create another documentation-only closeout.
T064 READY after merge: typed bundled-Deno selection in yt-dlp Rust launch plans.
Do not start T064 in this closeout phase.

No actual EJS challenge, integrated yt-dlp/Deno, YouTube, minimum-OS or full
package proof. Existing Windows positive media gap and unknown Linux exit-1 remain.
