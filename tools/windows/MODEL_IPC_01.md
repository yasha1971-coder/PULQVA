# MODEL-IPC-01 owner witness (test only)

Implementation is WIP until exact-head hosted compilation and deterministic checks pass.
Real Qwen and egress are NOT TESTED. Do not change ACLs, policy, firewall or loopback exemptions.
Use the diagnostic executable from the existing windows-appcontainer-ipc-probe-artifact
workflow at the implementation commit; verify its receipt SHA256 and source_sha first.
The unchanged production GUI/server and working package are not part of this test.

In a normal non-admin PowerShell, set three absolute fixed-local-drive paths:

```powershell
$env:PULQVA_MODEL_CLI = 'C:\path\llama-cli.exe'
$env:PULQVA_MODEL_GGUF = 'C:\path\Qwen_Qwen3.5-4B-Q4_K_M.gguf'
$env:PULQVA_MODEL_RECEIPT_DIR = 'C:\path\existing-owned-receipts'
.\pulqva_appcontainer_ipc_probe.exe --ignored --exact windows_isolation::model_ipc_witness::real_qwen_appcontainer_witness --nocapture
```

Replace paths with existing owner-controlled assets, never UNC, junctions or symlinks.
CLI/model bytes are streamed through SHA256 before launch and compared to the owner pins.
Adjacent DLL identities are inventoried but this is not DLL dependency closure.
The compiled production system policy/schema and fixed public countdown request are frozen
in manifest.json before spawn. No private user input is consumed. Every generation is new.

The process uses the existing zero-capability AppContainer Job launcher, separate stderr pipe,
180-second child deadline, combined 1 MiB output cap and existing 5-second cleanup.
A strict whole-stdout production decoder rejects logs, wrappers, duplicate/unknown fields,
invalid UTF8, trailing text and authority payloads. No JSON substring recovery.
An access denial is evidence: retain exact error, stop and do not repair permissions.
Generated text is never executed.

Retain the generation directory (manifest, bounded stdout/stderr and matrix_receipt.json).
Functional witness success is separate from overall acceptance, which remains false:
direct-network egress requires independent host observation and remains SKIP/NOT_TESTED.
A setup failure still emits a receipt once the output generation exists.
Hashing can exceed the child deadline; the deadline starts immediately before spawn.
Do not call PING/PONG or compilation a model PASS.

Known implementation gaps for the next bounded verification phase: hosted compile/tests,
receipt-validator deterministic tests and complete per-probe failure attribution/provenance.
Do not run a costly evidence matrix until those gates pass.
