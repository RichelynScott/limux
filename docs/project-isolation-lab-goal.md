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

As of 2026-06-11 06:04 EDT, SCS Wave A docs/readiness status is:

- Last pushed SCS commit:
  `96acd684ae77dfcc521d8298444c77a8be434237`
  (`docs(lab): add wave a iso intake draft`)
- Halo verified SCS `main` aligned with `origin/main`, but local status after
  gumo's `#29613` closeout showed two untracked files:
  `SECURITY_VM_SETUP_AND_LIMUX.code-workspace` and
  `project_isolation_lab/docs/WAVE_A_UBUNTU_2404_ISO_INTAKE_REVIEW_READINESS_2026-06-11.md`.
  Halo sent hcom `#29710` asking gumo to either commit/push that readiness
  record if it is intended as durable SCS docs, or confirm it is intentionally
  local/untracked and should be ignored. Gumo acked in hcom `#29732`: the
  readiness record is intentional and is being wired into SCS docs now, to be
  committed/pushed with pointer updates.
- Current SCS caveat after `#29732`: SCS is dirty again while gumo wires the
  readiness record into docs. Halo observed modified `FYI.md`, `HANDOFF.md`,
  `README.md`, `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`,
  `project_isolation_lab/FYI.md`, `project_isolation_lab/README.md`,
  `project_isolation_lab/docs/ACTIVE_GOAL.md`,
  `project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`,
  `project_isolation_lab/docs/PRD_PLAN.md`,
  `project_isolation_lab/docs/ROADMAP.md`, plus untracked
  `project_isolation_lab/docs/WAVE_A_UBUNTU_2404_ISO_INTAKE_REVIEW_READINESS_2026-06-11.md`
  and unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`.
  Do not treat SCS as clean/final for the readiness record until gumo reports
  the next commit SHA, hashes, verification, and post-push status.
- Current SCS dirty follow-up hashes observed by Halo:
  - readiness record:
    `5d51b8f33548a232897e67af4e2c415c2766b6f0bd40293c2c549549a25ae6b1`
  - SCS `HANDOFF.md`:
    `e07b7351bc5762640f0f34e0bb6bdfc717d5dcb71a99da125f1fb2e297cd4a05`
  - SCS `FYI.md`:
    `ab611a17d661f0df59558911a5e44033f52dfe0b4de4bc89af6281e0ed1b289a`
  - project isolation `FYI.md`:
    `5f7e3c8ef584f50557bba238e2d32ee0e95c5d5656d1f3f25fadf04769c00625`
  - `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
    `d0360c46c28c5debbf4e7d8278a72a761cc3d9b9134c201d9cded3043dce0f91`
  - `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
    `e26468b6f0104f35c13ed05039aabdb0d7777524c76a4231a5adf13459574261`
  - `ACTIVE_GOAL.md`:
    `4651a5ae6fa5a92c07925fbc3820fb10cf4c90064330a080544641189385b752`
- Final hashes from SCS commit `96acd684` before the dirty readiness follow-up:
  - `WAVE_A_UBUNTU_2404_ISO_INTAKE_COMMAND_PACKET_DRAFT_2026-06-10.md`:
    `f98d5ea00752fb23f3128b678753b0e3946dd5de55fa63ba67198418e70fe2a3`
  - `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
    `996e60895f039b65ae248e75fc5b0dad4bb27cceba6da8e20147b7e6d30d9e12`
  - `UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`:
    `9b518851690752ab399800613c6bafc00e94d74e5e3659ed09d7055315e54265`
  - `ACTIVE_GOAL.md`:
    `9e415b6a0a441c45258f52c21e89628bccb28ab8e75530db5c7c716c8866f22d`
  - `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
    `0e8ac9af7dc56493428cdccd2756aedda55279054487ae11b96bc7998147164f`
  - SCS `HANDOFF.md`:
    `690dae94139db9675545a8dea6d2b6e89519c2542eba61e2b80d6e28a3553f98`
- Review trail: Halo hcom `#29055`, `#29255`, `#29437`, and `#29567`; gumo
  hcom `#29106`, `#29279`, `#29509`, `#29577`, and final closeout `#29613`.
  Claude narrow re-review reported no remaining HIGH/MEDIUM blockers, and Halo
  `#29567` closed out the current Wave A draft from read-only review.
- Gumo-reported verification in `#29613`: `git diff --check`; Wave A embedded
  Bash block `bash -n`; no-delete literal scan; HTML parser; embedded JS
  `node --check`; Python `py_compile`; unit tests 18 OK; official Ubuntu
  metadata recheck without ISO download (`HEAD 200`, `Content-Length
  6655619072`, expected `SHA256SUMS` line present).
- Halo local verification after `#29613`: SCS status, `rev-parse HEAD`, recent
  log, and SHA256 checks for the final hash set above.
- Decision remains `WAIT`: no formal `$mutation-script-wave` GO, no ISO
  download/use approval, and no host/VM/WSL/Docker/HNS/WinNAT/network/package/
  SCRIM/global-config/runtime execution approval. Halo did not edit SCS and did
  not run host or runtime mutation. The current open SCS closeout caveat is
  hcom `#29732`: readiness-record pointer updates are in progress under gumo.
  Halo read the current readiness record at SHA256
  `5d51b8f33548a232897e67af4e2c415c2766b6f0bd40293c2c549549a25ae6b1`
  and found no new material blocker in its WAIT/readiness framing.

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
work is Wave A ISO intake packet review/freeze only.

Verify SCS state before relying on the recorded pointers:

```bash
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY status --short --branch
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY log -5 --oneline --decorate
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/PRD_ACCEPTANCE_REVIEW_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACCEPTANCE_GATES.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACTIVE_GOAL.md
hcom --version --name halo
hcom list --name halo
wsl.exe --list --verbose
wsl.exe --status
hcom events --last 80 --agent gumo --name halo
```
