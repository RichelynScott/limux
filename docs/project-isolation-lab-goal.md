# Project Isolation Lab Goal Alignment

Status: Limux-local alignment note. The canonical Project Isolation Lab owner is
gumo/SUPPLY_CHAIN_SECURITY under `/home/riche/Proj/SUPPLY_CHAIN_SECURITY`.

## Official Active Goal

Build a reusable Project Isolation Lab for safe work across projects: establish
a persistent full isolated Linux VM baseline, then a disposable full Linux VM
factory for risky package/build/install/source-execution trials, then a
Firecracker microVM layer for fast repeatable quarantine runs after the VM
foundations exist.

Treat disposable WSL only as an ergonomics/compatibility companion lane, not
hostile-code containment. The trusted Windows/WSL environment remains the
control plane and should not run first-touch untrusted installers, package
scripts, source builds, MCP startup, or global runtime mutation. All movement
from lab to trusted host requires evidence export, hashes/manifests, rollback,
artifact-intake review, mutation review where applicable, and explicit operator
approval.

## Limux Boundary

Limux is not the focus of the goal. This repo's role is to stay usable as a
tool and, later, to provide a realistic GUI Rust/Cargo acceptance case if the
SCS-owned lab gates need one.

Do not treat older Limux Phase 5D3 review/consensus work as the default next
lane. That work is historical until the operator explicitly redirects this
session back to Limux product development.

## Current Safe Limux Posture

- Repo-local/user-local launcher is the usable baseline.
- `--launch-mode hcom` is available for `agent-team` and `review spawn`.
- Full user-local install is gated behind the lab acceptance path.
- Generated installers, `.deb`, AppImage, AUR/release workflows, sudo/system
  install, package refreshes, and first-touch source/package execution remain
  out of scope without reviewed gates and explicit approval.

## Source Pointers

- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACTIVE_GOAL.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/README.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/PRD_PLAN.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ROADMAP.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACCEPTANCE_GATES.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/PROJECT_ISOLATION_LAB_STRATEGY_2026-06-10.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/LIMUX_INSTALL_VM_LAB_STRATEGY_2026-06-10.md`

## Current Restart Checkpoint

As of 2026-06-11 08:50 EDT, SCS Wave A V2 successor and marker-proof status is:

- SCS V2 freeze is complete and pushed at
  `0c1882b23bdb0dac9617734d23024752e35af4c6`
  (`docs(lab): add wave a v2 hardening packet`); `origin/main` matches local
  HEAD. SCS status is clean except unrelated untracked
  `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`. Halo did not edit SCS.
- Final SCS hashes from gumo hcom `#31842`, locally verified by Halo:
  - Wave A V2 successor packet:
    `36cad9340fdbb38d22cd91642a1cb702766ece09075dae10fac1206dc1b3a1bb`
  - Wave A V2 hardening review record:
    `529e15b0d6272cd2e79b0e7ed1b80c95053b224d013f560da364746731ad18e7`
  - HTML decision packet:
    `8791c4abd2415534e3b46098d28c8611ae2704f7edc4731bffe90fbc9414e06f`
  - Hyper-V mutation-wave packet:
    `03dbf27488045a95fc99b654f6ab246dc64e93133d5689c3c9171a53310586f5`
  - SCS `HANDOFF.md`:
    `dcbd6808523e12d2c2abfcc331b90a23ff222a930a333351e5b33fa59bb4d4b3`
- Prior formal-review hashes verified locally:
  - formal Wave A review record:
    `b370b92a94eb7d2a76ac941626b1048d51d3e1a9fb76f3f886e32f1407f73955`
  - readiness record:
    `7d16ecbd2e2f87c666cb4735f2f084e351cb7c20b9fb837fa698d4851429484a`
  - Wave A command packet:
    `f98d5ea00752fb23f3128b678753b0e3946dd5de55fa63ba67198418e70fe2a3`
  - mutation-wave packet:
    `5ae62ae9b8633067c93e7aa367bec10a1da7b1268ba14b0c06b6dd4519925077`
  - active goal:
    `4ab4c02b02a426fffa7a1ea16b7eb63cf3e39744ec238697157a678af29784fa`
  - HTML decision packet:
    `223f6e44f021d7ffc8edbd88dee0fd628a7dab31b518078d4a0539f09867d94c`
- Gumo hcom `#30052` requested a bounded read-only Limux acceptance/protocol
  handoff lens for the formal Wave A mutation-script wave input. Halo replied
  `GO` for using the hash-pinned Wave A packet as Wave A review input only:
  no CRITICAL/HIGH/MEDIUM blockers, with one LOW wording hardening suggestion
  to add exact `Limux run` / `cargo run` / `lab-to-host import` strings to the
  generated summary `not_authorized` list.
- Gumo hcom `#30550` reported verification for `7145ac8`: `git diff --check`;
  HTML parser; embedded JS `node --check`; extracted Wave A Bash block `bash
  -n`; no-delete literal scan; Python `py_compile`; unit tests 18/18 OK; and
  official `SHA256SUMS` / ISO `HEAD` read-only recheck with no ISO/key download.
- Halo hcom `#30665` recommended the safe default: create a docs-only V2
  successor packet to patch LOW residuals before any execution-packet freeze.
  Gumo acked in `#30689` and started that path without execution, ISO/key
  download/import, or host/runtime mutation.
- Gumo hcom `#30729` and `#30934` requested bounded read-only affected-LOW
  reviews of V2, but both named hashes drifted while gumo continued patching.
  Gumo reissued hcom `#30973` with frozen hash `00ef5e18...`. Halo verified the
  exact hash and replied `GO` for using it as the next V2 review artifact only.
  Gumo later acknowledged that `00ef5e18...` was superseded by final LOW/INFO
  patching and reissued hcom `#31264` for `36cad934...`. Halo verified that
  exact hash and replied `GO` for using it as the next V2 review artifact only.
  SCS later committed/pushed the freeze at `0c1882b23...`.
- The current V2 draft says it applies LOW hardening from the formal review:
  base-10 numeric validation, keyserver removal in favor of reviewed local
  public-key input, observed `VALIDSIG` evidence capture, signing-key-or-primary
  fingerprint matching, tighter no-authorization summary wording, and continued
  runtime proof planning.
- Gumo hcom `#31842` reports SCS verification passed: `git diff --check`;
  staged `git diff --cached --check` before commit; HTMLParser parse;
  `node --check` on extracted HTML JS; `bash -n` on extracted V2 command;
  isolated `static_check_no_delete_api.py` scan on extracted V2 shell with
  0 REMOVE/0 REVIEW; `py_compile` for watcher/tests; and
  `unittest tests.security_posture.test_supply_chain_watch -v` with 18 tests OK.
- SCS marker-proof packet freeze is complete and pushed at
  `bed7d37ec001c251971ba29f327a0ad25778ee5c`
  (`docs(lab): add marker proof review packet`); `origin/main` matches local
  HEAD. Gumo hcom `#32650` reported post-push SCS status clean except unrelated
  untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`. Halo did not edit
  SCS.
- Final marker-proof hashes from gumo hcom `#32650`; Halo rechecked the
  tracked-file hashes locally, and the extracted shell hash was verified during
  the read-only review:
  - Marker proof packet:
    `0284cf528d6abc53f5f96b8e87a56d0c2a51218afe217e0e0a7813d9467210c0`
  - Extracted shell block:
    `dc6da4e436cd9ddbc02fff8597f71b4589843eb54d9617ed21ec4c7958fe7cb4`
  - Marker proof review record:
    `28726df4453fab66cc5d1f09d1ecf2a622086d656fd42771be6cebc6a0df57c9`
  - HTML decision packet:
    `de4cf538e59f4fb5692d9da98f9bff62b7eee3e6b2b130ace2cb6e8fbf4d1139`
  - Hyper-V mutation-wave packet:
    `633148b184d8d713091b4b328371557acf21f8bb00df1260c95531906fc7ca73`
  - SCS `HANDOFF.md`:
    `18e62aea969587afedba28cea5ebef8b5cb9ef1689b37d692ef73e8e164bee22`
- Current marker-only WSL/DrvFs proof packet:
  `project_isolation_lab/docs/WAVE_A_WSL_DRVFS_MARKER_PROOF_PACKET_DRAFT_2026-06-11.md`
  at SHA256 `0284cf528d6abc53f5f96b8e87a56d0c2a51218afe217e0e0a7813d9467210c0`.
- Gumo hcom `#32019` requested a narrow read-only Halo review of that exact
  marker-proof hash. Halo replied in `#32076`: `WAIT` for the draft as a future
  execution-review candidate. No CRITICAL/HIGH blockers were found, but one
  MEDIUM blocker remains: the packet claims/proposes proving WSL ext4 to DrvFs
  behavior, but the command block captures path convention/path resolution
  (`/home/...` and `/mnt/c/...`) plus `df`/`mv` behavior without positive
  filesystem-type evidence for the WSL proof root or DrvFs target parent.
  Recommended fix: add explicit filesystem-type evidence, for example
  `stat -f`, `df -T`, `findmnt`, or equivalent, record it in evidence and
  summary, and either fail closed against reviewed expected values or require
  explicit operator/reviewer acceptance of the observed types.
  That `917c1753...` hash was superseded by gumo hcom `#32203`.
- Gumo hcom `#32203` reissued exact hash `9d497029...` after adding explicit
  `stat -f` filesystem-type evidence, expected WSL/DrvFs filesystem-type
  values, standalone executed-script SHA256 gate, no-direct-stdin/paste guard,
  `mv -nT` exit-code capture, marker-scale disclaimer, minimum proof
  free-space gate, exact basename containment, fuller `not_authorized`, and
  updated blockers/evidence outputs. Halo replied: `WAIT`. The prior
  filesystem-evidence MEDIUM is closed in substance, but a new MEDIUM blocker
  remains: the Failure Behavior section says wrong expected filesystem-type
  values stop before creating evidence, WSL marker, or DrvFs target paths, while
  the script creates `EVIDENCE_ROOT`, `WSL_PROOF_ROOT`, and `TARGET_PARENT`
  before comparing `WSL_FS_TYPE_BEFORE` / `TARGET_FS_TYPE_BEFORE` to expected
  values. If `TARGET_PARENT` was absent, a wrong filesystem expectation can
  still leave a newly created DrvFs target directory despite the documented
  fail-before-target-path claim.
- Recommended fix for `9d497029...`: either move filesystem-type checks earlier
  using existing parents such as `/mnt/c` and a reviewed WSL parent before
  creating child/evidence directories, or revise Failure Behavior / approval
  text to accurately state which parent/evidence directories may be created
  before filesystem mismatch stops. If target directory creation remains before
  filesystem-type validation, make that residual state explicit and require
  operator acceptance.
- Gumo hcom `#32386` then superseded `9d497029...` with exact hash
  `0284cf528d6abc53f5f96b8e87a56d0c2a51218afe217e0e0a7813d9467210c0`. Halo
  replied: `GO` for using this exact marker-proof draft as the next formal
  review/freeze candidate only. No unresolved CRITICAL/HIGH/MEDIUM blockers
  were found in the requested fix-verification scope. The prior `#32203`
  failure-order MEDIUM is closed: expected filesystem-type values are now
  checked against existing anchors `/home/riche` and `/mnt/c` before `mkdir` or
  path creation, child `WSL_PROOF_ROOT` and `TARGET_PARENT` filesystem types are
  checked again after creation, `findmnt` source/type/target/options evidence is
  recorded, `/dev/fd` and readable regular-file execution guards are present,
  standalone executed-script SHA256 gate remains present, collision `mv -nT`
  no-overwrite no longer fails solely on nonzero rc and records rc as evidence,
  and the marker-scale disclaimer, minimum proof free-space gate, exact basename
  containment, fuller `not_authorized`, blockers, and evidence outputs are
  present.
- LOW residuals for the future formal mutation-script review, not blockers to
  this exact artifact: if `WSL_PROOF_PARENT` itself is an unexpected existing
  symlink or mount point, the post-creation filesystem-type check would catch it
  before marker movement but after evidence/proof directories may be created.
  Also, the standalone-file guard blocks direct paste and `/dev/fd` paths but
  does not prohibit sourcing a regular reviewed script file; decide whether
  subprocess-only execution matters before the formal execution packet.
- Decision remains `WAIT` for execution: no ISO download/use approval, no
  operator execution approval, no selected execution operator/window, no
  approved `APPROVAL_REF`, `EXECUTION_OPERATOR`, `EXECUTION_WINDOW_UTC`, or
  `EXPECTED_PACKET_SHA256`, and no host/VM/WSL/Docker/HNS/WinNAT/network/
  package/SCRIM/global-config/Limux/runtime mutation approval.
- Gumo hcom `#32650` reports SCS verification passed: `git diff --check`;
  staged `git diff --cached --check`; HTMLParser parse; `node --check`
  extracted HTML JS; `bash -n` on the extracted marker shell; isolated
  `static_check_no_delete_api.py` scan on the extracted marker shell with
  0 REMOVE/0 REVIEW; `py_compile`; and
  `unittest tests.security_posture.test_supply_chain_watch -v` with 18 tests
  OK.
- SCS marker-proof formal mutation-wave review is now durable and pushed at
  `b8abc7d932a1e51c3e2cbd9f182aac6ca2beb913`
  (`docs(lab): record marker proof formal review`); `origin/main` matches local
  HEAD. SCS status is clean except unrelated untracked
  `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`.
- Formal marker-proof review artifacts from gumo hcom `#32994`, locally
  verified by Halo for tracked files:
  - Formal review record:
    `9b1b393904cc3b976584e18703f1d7e6d9002e064674f980e600b94e83b27250`
  - Hyper-V mutation-wave packet:
    `95f287fa4499162c31f25c44c54bd21df0dbed680385f465c6aa06a305e0904b`
  - HTML decision packet:
    `562f011e8f732b791bdb41019022ea9a07f8bb81ea4c0f00f64ae330ce7a8e5c`
  - SCS `HANDOFF.md`:
    `e76467628c7011cdbee0230e53c93163b554987912dda6a4e793658a79117e63`
- Formal review record:
  `project_isolation_lab/docs/WAVE_A_WSL_DRVFS_MARKER_PROOF_MUTATION_WAVE_REVIEW_2026-06-11.md`
  records `Decision: WAIT`, no execution approval, and no marker/ISO/
  key/checksum/network/Hyper-V/VM/WSL/Limux/Cargo/package/lab-to-host mutation
  authorization.
- Gumo hcom `#32994` reports SCS verification passed: `git diff --check`;
  staged diff check; HTML parse; extracted JS `node --check`; marker extracted
  shell hash / `bash -n` / static no-delete scan; `py_compile`; and 18 watcher
  tests.
- SCS marker-proof execution approval-input checklist is now durable and pushed
  at `f1272a0375e6ddc83343bba68d85c98cd6d635fc`
  (`docs(lab): add marker proof approval inputs`); `origin/main` matches local
  HEAD. SCS status is clean except unrelated untracked
  `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`. Halo did not edit SCS.
- Approval-input checkpoint artifacts from gumo hcom `#33327`, locally verified
  by Halo for tracked files:
  - Approval-input checklist:
    `dfb8bbf7b3b265bee3eec3ec65bcc99a4ab894f817391384b13cfebbbb5dcb45`
  - Hyper-V mutation-wave packet:
    `f20756e843a126fc41705e7177a9fbd5140af767c9355d51edbe30d5bf0cdcd9`
  - HTML decision packet:
    `ce8600e94f840968b0c90efd0ddb60b2c86da608e498735dcd6523e77a3ab209`
  - SCS `HANDOFF.md`:
    `dd1d234ba0b362e703206b50d27c41feb84d71f95954a223981ced347727cafc`
- Approval-input record:
  `project_isolation_lab/docs/WAVE_A_WSL_DRVFS_MARKER_PROOF_EXECUTION_APPROVAL_INPUTS_2026-06-11.md`
  records `Decision: WAIT`, says it is docs-only, not an execution packet,
  contains no command block, and does not authorize marker creation or runtime
  mutation.
- Gumo hcom `#33327` reports Claude adversarial review found no mutation/bypass
  risk and hash pins were reverified. Reported verification passed: diff
  checks; HTML parse; extracted JS `node --check`; marker shell hash /
  `bash -n` / static no-delete scan; `py_compile`; and 18 watcher tests.
- Hash note: gumo hcom `#33327` omitted the `3ec` segment in the approval-input
  hash text. Halo's local `sha256sum` against the tracked file in SCS commit
  `f1272a0` produced the value recorded above.
- Subsequent live SCS WIP: gumo hcom `#33581` requested read-only review of
  untracked
  `project_isolation_lab/docs/WAVE_A_WSL_DRVFS_MARKER_PROOF_PACKET_DRAFT_V2_2026-06-11.md`
  at packet SHA256
  `c52377cefa8be15d768cbbaabe5a05ddedb2e2bed1cdcfc566708a94f2f37e39`
  and extracted shell SHA256
  `3008e42671967c63221b1722187574c60e3796137c4f1d481ab58e46e53567f2`.
  Halo replied in hcom `#33749`: `GO` for using that exact draft as the next
  review/freeze candidate only, with no CRITICAL/HIGH/MEDIUM blockers in the
  requested LOW-fix spot-check. LOW residual, not blocker: mount-point ancestors
  below the anchors but above exact proof parents are not rejected before
  `mkdir`; post-creation filesystem-type checks should catch this before marker
  movement, but after evidence/proof/target directories may exist. Gumo hcom
  `#33763` acknowledged the stricter residual wording and kept execution
  `WAIT`. That interim WIP state is superseded by the durable `e455617...`
  checkpoint below, and execution remains `WAIT/NO-GO`.
- SCS final committed/pushed V2 marker-proof review-candidate state is now
  durable at commit `e455617ee84d3b86bb5739833199220076a9e8d7`
  (`docs(lab): add marker proof v2 review candidate`). SCS `main...origin/main`
  is clean except unrelated untracked
  `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`. Halo did not edit SCS.
- Final SCS hashes from gumo hcom `#34336`, locally verified by Halo for
  file-backed artifacts:
  - V2 packet:
    `c52377cefa8be15d768cbbaabe5a05ddedb2e2bed1cdcfc566708a94f2f37e39`
  - Extracted V2 shell:
    `3008e42671967c63221b1722187574c60e3796137c4f1d481ab58e46e53567f2`
  - V2 hardening review:
    `a0ded5dc093c6e98ae669190bf76706fbb6dbb81248a2e29aa63667250a87e2d`
  - Approval inputs:
    `da2a63cc670e07eeaed9749b04544a6ed7ad3d74728043973ca4900c0c0bdc5f`
  - Hyper-V review packet:
    `016652ae546200e636170381a79cf3af8cafa4c1797e479a688fbf7062af9be1`
  - HTML decision packet:
    `8d65541e5111db042616bfe5b68098640aec2a0bb3a7806a02fc7c9c2169079b`
  - SCS `HANDOFF.md`:
    `414315f33144c54dc624249d760ebb559591554ee0644d2f5bdfab03049e3c49`
  - `ACTIVE_GOAL.md`:
    `94cb1849aa30c2522de444f31ef65c08b02f0d531f3cb0413c152f123fbd4c76`
- Gumo hcom `#34336` reports verification passed: `git diff --check`, staged
  diff check, V2 shell hash / `bash -n` / static no-delete scan, HTML parse,
  HTML JS `node --check`, `py_compile`, and 18 watcher tests.
- Halo local verification after hcom `#34336`: SCS `git status --short
  --branch`, `rev-parse HEAD`, `git diff --check`, tracked-file SHA256 checks
  for the final hash set, fenced-shell extraction to `/tmp`, extracted V2 shell
  SHA256 match, `bash -n`, and Codex static no-delete scan over a dedicated
  `/tmp` copy with 0 REMOVE and 0 REVIEW.
- Subsequent SCS WIP after `e455617...` is already present and non-durable as
  of the 2026-06-11 08:50 EDT read-only spot-check. Current SCS status shows
  root/project docs and `project_isolation_lab/tasks/prd-003-evidence-export-intake.md` modified, an
  untracked `project_isolation_lab/docs/DATA_ONLY_EVIDENCE_INTAKE_GATE_DRAFT_2026-06-11.md`,
  and unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`.
  Gumo hcom `#34691`, `#34696`, `#34887`, and `#35006` confirm this Gate
  D/evidence-intake lane is still WIP, Claude adversarial review found MEDIUM
  issues including a polyglot/remote-content check issue and a later PRD
  screenshot contradiction finding, gumo is patching/finalizing, and there is
  no final SCS hash set yet.
  Last-observed volatile WIP hashes for the new evidence-intake lane:
  - `DATA_ONLY_EVIDENCE_INTAKE_GATE_DRAFT_2026-06-11.md`:
    `24b8bebd9ee8244c154cc67e3c1893faaba4f187966953da560235ee9be91fae`
  - `prd-003-evidence-export-intake.md`:
    `8f90287f415c606afc34a2c10b5f8e146cd3f05c48ae4c9f3b1b216637b4f564`
  - `ACCEPTANCE_GATES.md`:
    `607db367d7af031ce62c6615272d9ae8fe0296fd4b8ee31af4a63c8cebc528f5`
  - `PRD_ACCEPTANCE_REVIEW_2026-06-10.md`:
    `12f5cac630edd0e4ba28254c08709ea2032652e4b74492bb38d9f85fa6b6a118`
  These hashes are a restart breadcrumb only, not a review target; do not treat
  this evidence-intake WIP as durable until SCS commits/pushes or gumo issues
  an exact-hash review request after patching.
- Limux-side operator packet for this final checkpoint:
  `docs/PROJECT_ISOLATION_LAB_LIMUX_STATUS_DECISION_PACKET_2026-06-11_HALO.html`.
  It is a human-readable copy-back/status packet, not an execution approval.

## Numbered Options Moving Forward

1. **Docs/handoff first - active maintenance**: keep Limux restart docs and
   this status packet aligned with the durable SCS `e455617...` checkpoint and
   the latest explicitly non-durable Gate D WIP caveats.
2. **SCS durable V2 marker-proof checkpoint**: complete at SCS commit
   `e455617...` with V2 packet `c52377ce...`, extracted shell `3008e426...`,
   hardening review `a0ded5dc...`, approval inputs `da2a63cc...`, HTML packet
   `8d65541e...`, and gumo `#34336` verification. This is a review/freeze
   candidate only, not execution approval.
3. **If gumo sends a new exact-hash review request**: review it read-only only.
   Do not edit SCS, run packets, create markers, or mutate
   ISO/key/checksum/network/Hyper-V/VM/WSL/Limux/Cargo/package/runtime/global-
   config/SCRIM/lab-to-host state.
4. **Evidence-intake WIP**: current SCS Gate D/evidence-intake draft
   `DATA_ONLY_EVIDENCE_INTAKE_GATE_DRAFT_2026-06-11.md` is non-durable. Gumo is
   patching Claude MEDIUM findings; wait for commit/push or a new exact-hash
   review request.
5. **Marker execution gate**: only after frozen execution packet, final mutation
   review, explicit operator approval, execution window/operator, packet/script
   hashes, filesystem-type values, marker disposition, and residual disposition.
6. **Prior dry-run proof packet**: exact draft `0284cf52...` is frozen in SCS commit
   `bed7d37`, and formal marker-proof review is pushed at `b8abc7d`. It is
   still `WAIT/NO-GO` for execution, marker creation, ISO/key import/download,
   package execution, and host mutation until the operator approves a concrete
   execution packet/input checklist.
7. **Marker-proof approval inputs**: complete at SCS commit `f1272a0...` with
   approval-input checklist `dfb8bbf7...`, Hyper-V mutation-wave packet
   `f20756e8...`, HTML packet `ce8600e9...`, and gumo `#33327` verification.
   It is docs-only and still `WAIT/NO-GO`; it names the missing values for any
   future marker-only execution packet.
8. **Wave A ISO intake approval packet**: only after dry-run proof and mutation
   review converge should SCS request explicit operator approval to run ISO
   intake.
9. **Later lab layers**: persistent full Linux VM baseline, disposable full-VM
   factory, and Firecracker microVM layer remain downstream gated work.

As of 2026-06-10 22:10 EDT, SCS commit
`8ff345f docs(lab): add nat and iso intake blockers` is pushed. The
SCS-owned Hyper-V host mutation packet remains a draft with decision `WAIT`. It
is not approved to run.

Recorded hashes:

- `ACTIVE_GOAL.md`:
  `5cbdea1958ed5e442911da67411d9265c7816f2550f951e6ff070401a32d0a25`
- `HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`:
  `2c5a602ced61196807b58d72f82affec286d9fbe7bb3e9b8bc21b23a1f6c8597`
- `HYPERV_DOCKER_WSL2_NAT_RECONCILIATION_PLAN_2026-06-10.md`:
  `a4c8b6bf5a4ccc05874aa595820b1bc12d6db26ebaa67b949a77983292b96f0c`
- `UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`:
  `de4ec9be126a2c5438aff6098ca06eb3d31de5fb18b996b6fd59eae69686c5dc`
- `HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md`:
  `d7ba3c50cedd30c61b3c891f467cc79b3bf32650aebd5f30a5159a1fa8c426f0`
- `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
  `da7d795f12ccd59999bf3c5a8f9e969620d01f08123b9cd308fa8ed592f99b51`

Halo's latest Limux-side read found the packet materially safer but still
`WAIT`: it now has staged PowerShell blocks with a hard Hyper-V reboot
boundary, fail-closed preflight checks, unique evidence root, validated WSL/hcom
placeholders, deterministic offline first boot, later deliberate network
attachment, a fail-closed `GUEST_INTERFACE` netplan substitution, NAT/ISO
blocker plans, split offline-baseline vs optional switch/WinNAT stages, and
stage-scoped acceptance checks.

SCS has a review record at
`/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md`
that records Halo review inputs, the failed Claude multi-review attempt as no
usable formal wave result, applied fixes, and remaining blockers.

Read-only placeholder discovery from this session on 2026-06-10 21:24 EDT:

- `wsl.exe --list --verbose` shows default `Ubuntu` running on WSL2.
- `wsl.exe --status` reports default distribution `Ubuntu` and default version
  `2`.
- `docker-desktop` is also running on WSL2. This is not a mutation by itself,
  but it means the Docker/HNS/WSL2 NAT reconciliation gate is live and should
  remain a stop condition before any WinNAT creation.
- `hcom --version --name halo` reports `hcom 0.7.18`.
- `hcom list --name halo` succeeds and displays this hcom identity as
  `worker-limux-halo`.
- Candidate values for this session: `CONTROL_PLANE_WSL_DISTRO=Ubuntu` and
  `HCOM_CHECK_NAME=halo`. The packet should still resolve and validate both
  immediately before execution rather than hardcoding them permanently.

Read-only Ubuntu ISO artifact-intake discovery from this session on 2026-06-10
21:28 EDT:

- Official release directory:
  `https://releases.ubuntu.com/24.04/`
- Current title: Ubuntu 24.04.4 Noble Numbat.
- Candidate desktop ISO: `ubuntu-24.04.4-desktop-amd64.iso`.
- Directory listing describes it as the 64-bit PC AMD64 desktop image, size
  6.2G, modified 2026-02-10 01:41.
- Official `SHA256SUMS` entry:
  `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e *ubuntu-24.04.4-desktop-amd64.iso`
- `SHA256SUMS.gpg` verified successfully in an isolated `/tmp` GnuPG home after
  importing Ubuntu CD Image Automatic Signing Key (2012), key ID
  `D94AA3F0EFE21092`, fingerprint
  `8439 38DF 228D 22F7 B374 2BC0 D94A A3F0 EFE2 1092`.
- The ISO itself was not downloaded. This is artifact-intake evidence only, not
  approval to download, mount, boot, install, or execute anything.
- Sources used:
  `https://releases.ubuntu.com/24.04/`,
  `https://releases.ubuntu.com/24.04/SHA256SUMS`,
  `https://releases.ubuntu.com/24.04/SHA256SUMS.gpg`, and
  `https://ubuntu.com/tutorials/how-to-verify-ubuntu`.

Before any execution can be considered, the packet still needs formal
`$mutation-script-wave`, Docker/HNS/WSL2 NAT coexistence resolution before any
WinNAT creation, a frozen reviewed ISO download/use packet before downloading
or attaching the ISO, exact execution-packet freeze with SHA256, and explicit
operator approval for the execution window.

## Verified SCS Closeout

Halo verified SCS `main` aligned with `origin/main` at `8ff345f`, with only
unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.
Halo verified the recorded hashes above locally. No host/VM/WSL/Docker/HNS/
WinNAT/network/package/ISO/SCRIM/runtime mutation was run from Halo.

## Superseded Live In-Progress Caveat

This caveat was superseded by the verified `e8ef33a` closeout below. It remains
as the review trail for the blockers gumo fixed before committing.

As of 2026-06-10 22:24 EDT, SCS is no longer a clean frozen checkpoint from
Halo's view. Gumo acknowledged hcom `#28096` that he owns the current dirty
PRD/docs edits and that the previous frozen hashes are stale until he commits or
updates them.

Observed dirty SCS paths:

- `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`
- `project_isolation_lab/README.md`
- `project_isolation_lab/docs/ACCEPTANCE_GATES.md`
- `project_isolation_lab/docs/PRD_PLAN.md`
- `project_isolation_lab/docs/ROADMAP.md`
- `project_isolation_lab/tasks/prd-001-hyperv-linux-vm-baseline.md`
- untracked `project_isolation_lab/docs/PRD_ACCEPTANCE_REVIEW_2026-06-10.md`
- unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`

Halo's current review decision is still `WAIT`, not formal
`$mutation-script-wave` approval. Open findings routed to gumo:

1. Enforce local ISO existence, expected SHA256 via `Get-FileHash`, and
   artifact-intake approval/evidence reference before `Add-VMDvdDrive`.
2. Require exact network-stage preflight before GUI `New-NetIPAddress` /
   `New-NetNat`, or make the reviewed PowerShell B4 block the only accepted NAT
   command shape.
3. Scope in-VM gateway/DNS checks to the `NETWORK` stage only.
4. Decide whether `project_isolation_lab/evidence/` is tracked or ignored before
   future ISO intake.

At that point, the recorded `8ff345f` hashes were no longer current because
SCS had advanced. Halo waited for gumo's next pushed commit and verified the
new hashes locally before updating this file again.

## Verified SCS PRD/Packet Closeout

As of 2026-06-10 22:33 EDT, gumo pushed SCS commit
`e8ef33a docs(lab): record prd acceptance gates`
(`e8ef33ab190f76f605d369412b957b6e17e74636`). Halo verified SCS `main`
aligned with `origin/main`, with only unrelated untracked
`SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.

Final verified hashes:

- `ACTIVE_GOAL.md`:
  `159d4ea1c8397a2640768017ea25dc46afe01a4b816de2462a618b5cb76dc8ad`
- `HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`:
  `bd3b053c99c555684fa198c229842f179ecd577c5985df534895811b767ca2bb`
- `HYPERV_DOCKER_WSL2_NAT_RECONCILIATION_PLAN_2026-06-10.md`:
  `a4c8b6bf5a4ccc05874aa595820b1bc12d6db26ebaa67b949a77983292b96f0c`
- `UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`:
  `d472b0b5b555d6aa8026690b06bbb7b016d04eb6751cadd30602f3c4b55cbc32`
- `HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md`:
  `9cdd0923d5f2618ec9544a617f68b564bf3adecfafd75d6ff4969cf623a2d0f2`
- `PRD_ACCEPTANCE_REVIEW_2026-06-10.md`:
  `c1be23fb69d96e9567865742a2a53ee1cd976e70a0ddbe3e8df65c3768814581`
- `prd-001-hyperv-linux-vm-baseline.md`:
  `7580a396bf2f76b507f74893dce3c40d885a3524c4836a376a640326a61ece86`
- `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
  `f05a03f2f7e3a890a149a1bcbe04c4349416b97166fe8a96e86c37afea88b82e`

Status remains `WAIT`. The packet is stronger planning evidence, not approval
to run Hyper-V, create a VM, create WinNAT, download/attach the ISO, run
packages, grant secrets, or mutate the trusted host. Next work is formal
`$mutation-script-wave` on the exact SCS packet set and a separate frozen ISO
artifact-intake/download packet.

## Superseded Mutation-Wave Draft Caveat

As of 2026-06-10 22:46 EDT, SCS was dirty again after `e8ef33a`. Gumo appeared
to be drafting the next review input packet:
`project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`.
This dirty state was superseded by SCS commit `7427285` below. It remains here
only as the review trail for hcom `#28584`, `#28712`, and `#28723`.

Observed SCS dirty paths:

- `FYI.md`
- `README.md`
- `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`
- `project_isolation_lab/FYI.md`
- `project_isolation_lab/README.md`
- `project_isolation_lab/docs/ACTIVE_GOAL.md`
- `project_isolation_lab/docs/PRD_PLAN.md`
- `project_isolation_lab/docs/ROADMAP.md`
- untracked `project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`
- unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`

Halo's read-only review found the new packet directionally aligned: it keeps
Decision `WAIT`, says it is not a completed wave or execution approval, splits
future review into Wave A ISO intake, Wave B offline Hyper-V baseline, and Wave
C optional network stage, and defers Wave C behind Docker/HNS/WSL2 NAT
reconciliation. Open docs-drift findings were sent to gumo in corrected hcom
`#28584`:

1. HTML copy-back should name Wave A ISO intake packet review/freeze as the next
   formal scope, or explain why generic Hyper-V packet review remains the next
   step.
2. Mutation-wave packet wording should clarify whether Wave B is the first VM
   mutation scope while Wave A is the first formal review/artifact-intake scope.

Gumo acked in hcom `#28606` and is patching. He also reported Claude-side
findings: Wave B is too broad across the B0-B3/reboot boundary, convergence
criteria are underspecified, gate docs are missing from the hash set, and Wave A
wording is too executable-looking while the ISO packet is still a stub.

Halo closeout `#28712` found those requested findings addressed in the current
draft: Wave A is first formal review/artifact-intake scope; Wave B is split
across B0-B3 and not a single pasteable run; convergence criteria are tighter;
gate-authority hashes include `ACCEPTANCE_GATES.md` and `ACTIVE_GOAL.md`; and
Wave A says it is not yet a frozen command packet. Gumo acked in `#28723` and
is rerunning checks before commit/push.

Ignore hcom `#28569` except as an error trail: shell backtick substitution
stripped inline phrases and corrupted the NAT hash. Corrected review is hcom
`#28584`. Decision remains `WAIT`; formal `$mutation-script-wave` has not run
or converged, and no host/VM/network/package/runtime mutation is approved.

## Verified SCS Mutation-Wave Packet Closeout

As of 2026-06-10 22:53 EDT, gumo pushed SCS commit
`7427285 docs(lab): add hyperv mutation wave packet`
(`7427285b267bba3d69483c3354edf504299e2956`). Halo verified SCS `main`
aligned with `origin/main`, with only unrelated untracked
`SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.

Final verified hashes:

- `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
  `12622e2416addad87d4ad4fad222f3df94aa4372ed52dea2f67096a52d2c9fbb`
- `HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`:
  `bd3b053c99c555684fa198c229842f179ecd577c5985df534895811b767ca2bb`
- `HYPERV_DOCKER_WSL2_NAT_RECONCILIATION_PLAN_2026-06-10.md`:
  `a4c8b6bf5a4ccc05874aa595820b1bc12d6db26ebaa67b949a77983292b96f0c`
- `UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`:
  `d472b0b5b555d6aa8026690b06bbb7b016d04eb6751cadd30602f3c4b55cbc32`
- `HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md`:
  `9cdd0923d5f2618ec9544a617f68b564bf3adecfafd75d6ff4969cf623a2d0f2`
- `PRD_ACCEPTANCE_REVIEW_2026-06-10.md`:
  `c1be23fb69d96e9567865742a2a53ee1cd976e70a0ddbe3e8df65c3768814581`
- `ACTIVE_GOAL.md`:
  `9b4cd6fa81326d5f7749bb2b1e35adc35b7b3c2d588a43a53673b4323fcc947f`
- `ACCEPTANCE_GATES.md`:
  `a3279d7b87ca40fd6bcff74dc483dc1f92fb4372bad2730c92152345ee281404`
- `prd-001-hyperv-linux-vm-baseline.md`:
  `7580a396bf2f76b507f74893dce3c40d885a3524c4836a376a640326a61ece86`
- `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
  `4d4cb6824b38e7592ae863025323d9c856bac810f12734a6386a3d6a42740af1`
- SCS `HANDOFF.md`:
  `0f2fa08be67cd5323ec0fa33f19c4742cb4b66b8499d520a331d69b7f76497f3`

Gumo's hcom closeout `#28853` reports `git diff --check`, HTML parser,
embedded JS `node --check`, `py_compile`, and unittest 18 OK. Halo verified
`git diff --check`, HTML parser, embedded JS `node --check`, no-write Python
syntax compile, `python3 -B -m unittest tests.security_posture.test_supply_chain_watch -v`
with 18 tests OK, and HEAD/upstream SHA alignment.

Status remains `WAIT`: this is not formal `$mutation-script-wave` GO, not ISO
download/use approval, and not host/VM/network/package/runtime approval. Next
work is for SCS/gumo to durably freeze/commit exact marker-proof draft
`0284cf52...` and route it into formal mutation-script review after Halo's
`#32386` review.

Verify SCS state before relying on the recorded pointers:

```bash
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY status --short --branch
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY log -5 --oneline --decorate
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/WAVE_A_UBUNTU_2404_ISO_INTAKE_COMMAND_PACKET_DRAFT_V2_2026-06-11.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/WAVE_A_UBUNTU_2404_ISO_V2_HARDENING_REVIEW_2026-06-11.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/WAVE_A_WSL_DRVFS_MARKER_PROOF_PACKET_DRAFT_2026-06-11.md
hcom --version --name halo
hcom list --name halo
hcom events --last 80 --thread project-isolation-lab-goal --name halo
```
