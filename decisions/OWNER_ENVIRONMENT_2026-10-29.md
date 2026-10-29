# Owner environment and manual launch report — 2026-10-29

## Evidence and scope

The owner supplied this information in chat on 2026-10-29. Installed application
versions below are transcribed from the owner's Windows Installed apps list;
they were not independently inventoried or executed during this documentation task.
The owner reports manually launching PULQVA both on native Windows and in Linux
through WSL (written as "VSL" in the message). Exact executable paths, source
commits, launch dates, commands, and completed user journeys were not supplied.
This is evidence of an owner-reported manual launch, not an independently
verified clean-machine launch, Tor/privacy guarantee, or end-to-end acceptance.

## Relevant reported local environment

| Component | Owner-supplied information |
| --- | --- |
| Windows Subsystem for Linux | 2.6.3.0; Linux environment previously used for PULQVA |
| PowerShell / terminal | Owner reports command-line tooling used on Windows and Linux; PowerShell edition/version not established |
| 7-Zip x64 | 25.01 |
| Git | 2.52.0 |
| GitHub CLI | 2.102.0 |
| GitHub Desktop | 3.6.2 |
| Visual Studio Build Tools 2022 | 17.14.16 |
| Windows SDK | 10.0.26100.4654 (listed package version 10.1.26100.4654) |
| CMake | 4.3.3 |
| Node.js | 22.19.0 |
| Python x64 | 3.11.9 |
| Visual Studio Code | 1.144.0 |
| Microsoft Edge | 156.0.4314.50; this alone does not establish WebView2 Runtime availability |
| Visual C++ runtime x64 | v14, 14.51.36247.0 |
| Visual C++ runtime x86 | 2015–2022, 14.44.35211.0 |
| Ollama | 0.40.4; owner reports a downloaded/installed local LLM, currently not used |
| Local LLM identity | UNKNOWN: model name, tag, quantization, storage path, and running state not established |

The general installed-app list also contains several .NET runtimes, development
utilities and unrelated applications. These are not recorded individually because
no PULQVA requirement for them was established. No application was removed,
updated, installed, started, or configured as part of saving this report.

## Previously observed local PULQVA context

- C: source repository: C:\PULQVA-NATIVE\src\PULQVA-0ed2af1,
  observed detached HEAD fedd9f91ef259e23e78e3dc7902a61af478a897f.
- Image source: /work/src/PULQVA-0ed2af1 inside
  D:\PULQVA-WSL\linux-workspace.ext4,
  observed HEAD 0ed2af16d8a83194d8a4e967a3989fa2c2649452.
- Previously inspected Windows executable:
  C:\PULQVA-NATIVE\pulqva-w\PULQVA.exe, 12,222,464 bytes, x64 PE.
  SHA-256: 83003B6321302586CB17101EE6753C92CEDF9936292EF52EBFABE61D934691C1.
- Its README-FIRST.txt declares source commit
  98e44285d77250b2a063a8e2d4992df5f7e7c275 and calls it an experimental
  portable candidate, not a verified release. Binary-to-source correspondence
  has not been independently proven. The owner did not identify this specific
  executable as the one they launched.

## Continuity limits

Do not assume the laptop lacks development tools merely because an older agent
or sandbox could not access them. The historical NEXT.md statement in the inspected C: copy that no local
cargo/rustc existed describes that earlier session, not a fresh inventory of this
Windows laptop. Rust toolchain availability and versions still require a scoped
check. Do not reinstall dependencies solely on the basis of that historical text.

A local Ollama installation does not establish that PULQVA uses it or that a model
is running. Do not start it, switch providers, or weaken privacy contracts based
on this note. The existing product task and last verified commit remain unchanged.

One possible later diagnostic action, requiring its own authorized phase: identify
the existing local model and actual launch package through scoped read-only
inspection, without executing PULQVA, starting WSL or starting a model service.
