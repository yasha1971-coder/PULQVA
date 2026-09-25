# T063R — Windows Arti build identity recovery

Status: ACTIVE, diagnostic phase only.

Diagnostic map acceptance passed at 97e84ca5, run 36161256355. Changed function
is liblzma-sys 0.4.7 lzma2_decode. Current phase: explicit authenticated MSVC
compiler trial; SDK directories selected but their content not pinned.
Windows execution and independent clean-build agreement remain pending.

Outcome: preserve linker map and available MSVC/SDK input hashes from one Windows
build while retaining the full executable identity gate. Acceptance: usable map,
nonempty inventory; resolve differing function RVA 0xdb0c40 to its library/object.

Not yet tested on Windows. No local PowerShell runtime is available.
Follow-up repair must constrain demonstrated changing inputs, prove independent
clean-build agreement and pass exact-head CI. Never refresh the trusted hash just
because another runner produced a different binary. T064 stays blocked.
