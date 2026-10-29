# PR #98: bounded Tor readiness diagnostics

Status: WIP, no live acceptance; no merge authorization.
Base: proto/v0.1-source-admission at 0262e9ad0539f11fbdf03d12b9c168f73ba33ead.
Initial diagnostic SHA: 6e50cea4e17c506f5645044ee99612be9efc2ac9.

## Scope and invariant preservation

Opt-in PULQVA_TOR_DIAGNOSTICS=1 records bounded classified readiness metadata to
stderr after owned Arti stop/wait. Raw error strings, requests and paths are never
formatted. Default-off behavior, SOCKS remote DNS, readiness capability boundary,
90-second gate, sidecar pins, lockfiles and frozen kernel remain unchanged.
Sink failure cannot bypass cleanup. This is diagnostic evidence, not a Tor fix.
No prototype or Tor launched while preparing this PR; USB files unchanged.
Retain only explicitly opted-in owned test logs, with owner-controlled removal.

## Preserved checkpoint moved out of protected State/NEXT

The initial commit added candidate metadata and an appended continuation note to
PROJECT_STATE.json and NEXT.md. Continuity run 40866453529 rejected the protected
file changes. Both files are restored byte-for-byte to the PR base in this
correction. The diagnostic checkpoint now lives here and in the PR conversation;
active tasks, continuation routing, policy and verified anchors are untouched.

Local evidence: four diagnostic formatter tests passed against the real privacy
crate. Windows/Linux CI diagnostic unit jobs passed in run 40866453506. Full
Windows desktop checks passed; Linux desktop checks failed two old assertions
expecting 2.6.0 despite the existing Linux Arti 2.7.0 platform pin. Only these two
test expectations are corrected; production pins and code are not altered.
The intentionally panicking deno failure fixture is a passing negative test,
not a cause of the two failing desktop tests.

## Remaining gates

- Observe continuity and desktop integration for the corrective SHA.
- Separate Windows Arti build 40866453638 produced SHA256
  ab592becfcd446742e7af59aaed20720b391f26b770fa34a0065259448c870ad;
  pinned expected 9245c7b5f71391238539bf2d78a492453cae66f978cf8e479039a26f1667f3df.
  This PR does not adopt an unauthenticated replacement or relax its identity gate.
- Local cargo fetch --locked on exFAT failed setting adler2 mtime (OS error 87).
  Restore exact dependencies into isolated C: NTFS cache without changing locks.
- Every later runtime test needs explicit launch authorization and CLEANUP:
  close test app, reap owned children, verify PID identities/ports, verify package
  hashes, and save a new checkpoint even on error or timeout.

## Package evidence (not live readiness)

Windows source 98e4428: five expected files; app SHA256
83003b6321302586cb17101ee6753c92cedf9936292ef52ebfabe61d934691c1;
Arti 2.6.0 and other sidecars matched run 38353268449/job115150267055.
Linux source 0262e9a: five expected files; app SHA256
ebf8aa84f711a9a372b486c657be8269532712f2b1352c0937dc9ce77142f711;
Arti 2.7.0 SHA256 5f579c2525ae911d2ce402595b83512c03f8f24f224d49b05dcdee927b28ec01
matched run38408975752/job115335963887, where its measured identity was embedded
by the Linux packaging workflow. GTK3/WebKit2GTK4.1 host readiness is NOT TESTED.
Neither package has live GUI/Tor/search acceptance from this implementation work.
