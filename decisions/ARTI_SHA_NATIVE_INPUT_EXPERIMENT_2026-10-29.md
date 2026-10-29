# Arti SHA failure: controlled native-input experiment
Status: EXPERIMENT_PREPARED; no pin promotion, no root cause claimed.
Base PR10004310a1b21d370bf05e588dd126732cef71c0988. Failing job123635868309.
Verified old9245c7 and newab592bec binaries differ in2,232,733bytes, including .text/.rdata; same file size19,624,960. Rust/linker/flags and448 crate name/version sets match, but historical lock checksum was not captured. Old artifact listings empty; old verified package binary retained on C. New artifact ZIP digests matched GitHub metadata. Runner image changed; compiler64 EXE/DLL files authenticate, headers/libs do not.

One experiment varies MSVC include/CRT library ROOT (restored vs runner tree, both14.51.36231); compiler/archiver/DLL selection, Rust1.91, rust-lld, SDK10.0.26100 and build flags stay fixed. Record BOTH tree manifests in each arm plus exact registry Arti source archive/Cargo.lock hashes, selected env and existing object fingerprints/link maps. Root path and content are one declared factor, not proof of a particular header/library or historical SDK equivalence.

Both arms retain production CLI/PE metadata/full-file pin checks. Different artifact names avoid collision. Runner-input arm is diagnostic, not a repin or accepted binary. At completion compare source/lock/tool identities and both tree manifests BEFORE interpreting output hashes. If arm hashes differ, localize native input selection; if equal, reject this factor as sufficient and investigate another measured boundary. No unchanged manual rerun or silent hash update.

Next: observe exact experiment-head run/jobs and compare arm receipts and artifacts. Priority2 D cleanup is not started; nothing may be deleted before owner confirms a concrete preservation-backed list. No USB changes, local installer/executable launch, merge or OVH.
