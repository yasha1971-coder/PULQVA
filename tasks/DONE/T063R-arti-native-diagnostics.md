# T063R — Windows native compiler pin verification

DONE within compiler-selection scope at 2bf215fc06bcf6038fdfe94b71f665a932943fc2.
All 10 triggered checks green. Two independent clean builds produced byte-identical
Windows Arti executables matching the existing full SHA256 allowlist.
Compiler invocation for liblzma-sys 0.4.7 lzma2_decoder.c was inspected in the
preserved Cargo build-script output and selects authenticated MSVC 14.51.36231.
Evidence: sidecars/arti/WINDOWS_COMPILER_VERIFICATION.json.

Missing/changed pinned compiler files stop the build; no silent fallback.
This does not prove hermetic SDK inputs or guarantee availability on future hosted
images. Old failed runner compiler hashes are unavailable, so precise historical
causality remains unproven. Full binary SHA256 remains the final gate.
PR closeout must pass exact-head CI before merging. T064 follows after merge.
