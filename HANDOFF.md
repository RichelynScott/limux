# Limux Session Handoff

Last updated: 2026-06-10 23:20 EDT

## Active Thread Goal - Project Isolation Lab

The official active goal is **not Limux feature development**. Limux is a usable
tool and may later serve as one acceptance artifact, but the team focus is the
cross-project isolation lab:

```text
Build a reusable Project Isolation Lab for safe work across projects: establish
a persistent full isolated Linux VM baseline, then a disposable full Linux VM
factory for risky package/build/install/source-execution trials, then a
Firecracker microVM layer for fast repeatable quarantine runs after the VM
foundations exist. Treat disposable WSL only as an ergonomics/compatibility
companion lane, not hostile-code containment. The trusted Windows/WSL
environment remains the control plane and should not run first-touch untrusted
installers, package scripts, source builds, MCP startup, or global runtime
mutation. All movement from lab to trusted host requires evidence export,
hashes/manifests, rollback, artifact-intake review, mutation review where
applicable, and explicit operator approval.
```

Limux-side responsibility: keep the existing repo-local/user-local Limux setup
usable, preserve the hcom launch-mode work already shipped, and provide Limux
only as an eventual acceptance case if the isolation-lab gates need a real GUI
Rust/Cargo project. Do not resume Phase 5D3/internal Limux review feature work
unless the operator explicitly redirects back to Limux product development.

Canonical isolation-lab ownership remains in
`/home/riche/Proj/SUPPLY_CHAIN_SECURITY` with gumo/SCS. This repo now has a
local pointer at `docs/project-isolation-lab-goal.md`; treat it as a Limux
alignment note, not the source of truth.

Current SCS active dirty state as of 2026-06-10 23:20 EDT:

- SCS was clean/aligned at commit
  `7427285b267bba3d69483c3354edf504299e2956` after the mutation-wave packet
  closeout below, but gumo has since started the next Wave A ISO intake command
  packet draft. Do not treat the `7427285` hash set as current for Wave A until
  gumo commits/pushes or explicitly supersedes the draft.
- Observed dirty SCS paths:
  - modified: `FYI.md`
  - modified: `HANDOFF.md`
  - modified: `README.md`
  - modified: `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`
  - modified: `project_isolation_lab/FYI.md`
  - modified: `project_isolation_lab/README.md`
  - modified: `project_isolation_lab/docs/ACTIVE_GOAL.md`
  - modified: `project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`
  - modified: `project_isolation_lab/docs/PRD_PLAN.md`
  - modified: `project_isolation_lab/docs/ROADMAP.md`
  - modified: `project_isolation_lab/docs/UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`
  - untracked:
    `project_isolation_lab/docs/WAVE_A_UBUNTU_2404_ISO_INTAKE_COMMAND_PACKET_DRAFT_2026-06-10.md`
  - unrelated untracked: `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`
- Current draft hashes from Halo's read-only review:
  - `WAVE_A_UBUNTU_2404_ISO_INTAKE_COMMAND_PACKET_DRAFT_2026-06-10.md`:
    `adf63eb53406a18ebc4c95e9b726a83cf0841021475fb0676c83bc17f9a52024`
  - `UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`:
    `9b518851690752ab399800613c6bafc00e94d74e5e3659ed09d7055315e54265`
  - `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
    `4f8ade54f8f2f20e58bda382d1bb60cde340842aed04448772aad8c86efe0aab`
  - `ACTIVE_GOAL.md`:
    `9e415b6a0a441c45258f52c21e89628bccb28ab8e75530db5c7c716c8866f22d`
  - `PRD_PLAN.md`:
    `81c37d69456db7ec26eea383c377b17d4ac6c65d8025dc564fb0160132cdb038`
  - `ROADMAP.md`:
    `fd97bb23a56bde0ac466cfa896d711860ca023388bd398bbe9d41588b10f4ff1`
  - `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
    `0e8ac9af7dc56493428cdccd2756aedda55279054487ae11b96bc7998147164f`
  - SCS `HANDOFF.md`:
    `0a476769ec8fce353bedc1726b5c10c21b31bd5e9d7b241bc2d444ef737b0245`
- Halo sent file-backed hcom review `#29055` to gumo. Decision remains
  `WAIT`: not formal `$mutation-script-wave` GO, not ISO download/use
  approval, and not host/VM/network/package/runtime approval.
- Gumo acked in hcom `#29106`: he is patching the Wave A draft rather than
  accepting residual risk, including approval/window/packet-hash gating,
  HTTPS redirect/status metadata, `VALIDSIG` fingerprint binding,
  disk-space/max-size guard, stricter target path preflight, and evidence
  summary improvements. The Wave A draft hash is now changed, and those items
  are materially improved.
- Halo sent follow-up hcom review `#29255`. Decision remains `WAIT`: not formal
  `$mutation-script-wave` GO, not ISO download/use approval, and not
  host/VM/network/package/runtime approval.
- What looks fixed from hcom `#29055`: approval/window/operator/packet-hash
  values now fail closed before evidence-directory creation, target-directory
  creation, network calls, or artifact writes; `curl` now uses
  `--proto-redir '=https'` and records headers/effective URL/status/byte count/
  timing metadata; GPG verification parses `VALIDSIG`; target free-space and
  `curl --max-filesize` guards exist; target path preflight is stricter; the
  evidence summary and operator-bound no-use attestation are improved.
- Remaining findings sent to gumo in `#29255` before commit/freeze:
  1. HIGH: patched draft now downloads the 6.2G ISO partial into the SCS repo
     tree at `project_isolation_lab/evidence/.../download/` before moving it to
     `/mnt/c/VMs/SCS-Lab/ISOs/`. That conflicts with the packet's Non-Goal:
     no movement of the ISO into a Git repo or trusted project source tree.
     Recommended fix: keep repo-local evidence to metadata/checksums/logs only
     and place ISO partial bytes under a reviewed non-repo intake/quarantine
     path such as the approved target parent or sibling staging directory.
  2. MED: effective URL and HTTP metadata are captured but not enforced. Add
     fail-closed checks for expected `https://releases.ubuntu.com/24.04/...`
     effective URLs or explicitly reviewed allowed redirects, plus `http_code`
     checks.
  3. LOW/MED: required-tool list omits `date`, `uname`, and `cat`; integer
     validation accepts `0`. Harden those before freeze.
- Gumo acked in hcom `#29279`: he agrees with the repo-local raw ISO conflict
  and is patching raw ISO partial staging to a non-repo WSL state path, with
  only metadata/hashes under `project_isolation_lab/evidence/`. He also plans
  to enforce effective URLs/`http_code`, add `date`/`uname`/`cat` tooling, and
  add nonzero integer validation before final verification/commit.
- Halo sent follow-up hcom review `#29437` after gumo's `#29279` patch.
  Decision remains `WAIT`: not formal `$mutation-script-wave` GO, not ISO
  download/use approval, and not host/VM/network/package/runtime approval.
- What looks fixed from hcom `#29255`: raw ISO partial now stages outside the
  repo under `/home/riche/.local/state/scs-lab-intake`; effective URL plus
  `http_code` are enforced; `date`, `uname`, and `cat` are required; numeric
  knobs reject zero.
- Remaining findings sent to gumo in `#29437` before commit/freeze:
  1. HIGH/MED: WSL staging writes raw ISO bytes after only a string-prefix
     check on `WSL_INTAKE_ROOT`. The script records `realpath -e` for the
     intake root but does not fail closed that the resolved path is exactly
     under `/home/riche/.local/state/scs-lab-intake` and not under the SCS repo
     or another trusted tree. Add resolved-path containment checks for
     `WSL_INTAKE_PARENT`/`WSL_INTAKE_ROOT` before network writes.
  2. MED: free-space guard still checks only the final `/mnt/c` target, but the
     6.2G ISO is downloaded first to `WSL_INTAKE_ROOT`. Add `df -PB1` and a
     staging threshold before the ISO `curl`, plus before/after staging
     evidence.
  3. LOW optional: `assert_curl_writeout` uses `awk -F=` and `$2`, which is OK
     for current no-query URLs but fragile for any future URL containing `=`.
- Gumo acked in hcom `#29509`: Claude re-review also found the
  staging-filesystem free-space gap, and gumo is patching both material
  findings now: resolved-path containment for `WSL_INTAKE_PARENT`/root with
  repo-escape rejection, plus `MIN_STAGING_FREE_BYTES` and WSL staging
  before/after `df` evidence. The `awk` parser issue remains optional unless
  touched cheaply.
- Verification run by Halo for the patched dirty draft: targeted full read of
  the Wave A draft, targeted diff read of changed SCS pointer docs, targeted
  `rg` for approval/hash, redirect metadata, `VALIDSIG`, disk/max-size, path
  preflight, WSL intake path, and attestation changes; targeted `sed` read of
  the command block; `git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY diff
  --check`; `git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY status --short
  --branch`; and SHA256 checks listed above. Halo did not download the ISO and
  did not edit SCS.

Current SCS restart pointers as of 2026-06-10 22:10 EDT:

- SCS commit:
  `8ff345f docs(lab): add nat and iso intake blockers`
- Canonical active goal:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACTIVE_GOAL.md`
  SHA256:
  `5cbdea1958ed5e442911da67411d9265c7816f2550f951e6ff070401a32d0a25`
- Phase 1 host-mutation draft:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`
  SHA256:
  `2c5a602ced61196807b58d72f82affec286d9fbe7bb3e9b8bc21b23a1f6c8597`
- Packet status: `WAIT`; it is not executable. Gumo folded in Halo review
  fixes and the NAT/ISO blocker plans: staged PowerShell path with a hard
  Hyper-V reboot boundary, fail-closed preflight checks, unique/fail-if-exists
  evidence root, conservative WSL/hcom placeholder validation, deterministic
  offline first boot, later deliberate network attachment, `GUEST_INTERFACE`
  substitution for the guest netplan fallback, split offline-baseline vs
  optional switch/WinNAT stages, and stage-scoped acceptance checks.
- Docker/HNS/WSL2 NAT reconciliation plan:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_DOCKER_WSL2_NAT_RECONCILIATION_PLAN_2026-06-10.md`
  SHA256:
  `a4c8b6bf5a4ccc05874aa595820b1bc12d6db26ebaa67b949a77983292b96f0c`
- Ubuntu ISO artifact-intake plan:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/UBUNTU_2404_ISO_ARTIFACT_INTAKE_PLAN_2026-06-10.md`
  SHA256:
  `de4ec9be126a2c5438aff6098ca06eb3d31de5fb18b996b6fd59eae69686c5dc`
- SCS review record:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md`
  SHA256:
  `d7ba3c50cedd30c61b3c891f467cc79b3bf32650aebd5f30a5159a1fa8c426f0`
- SCS HTML decision packet:
  `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`
  SHA256:
  `da7d795f12ccd59999bf3c5a8f9e969620d01f08123b9cd308fa8ed592f99b51`
- Remaining gates: formal `$mutation-script-wave`, Docker/HNS/WSL2 NAT
  coexistence decision before WinNAT, frozen reviewed ISO download/use packet
  before downloading or attaching the ISO, exact execution-packet freeze with
  SHA256, and explicit operator approval/execution window.
- Read-only placeholder discovery from this trusted WSL session found:
  `CONTROL_PLANE_WSL_DISTRO` candidate `Ubuntu`; `HCOM_CHECK_NAME` candidate
  `halo` as the hcom sender-name argument; `hcom --version --name halo`
  reports `hcom 0.7.18`; `hcom list --name halo` succeeds and identifies this
  session as `worker-limux-halo`. These are candidates only; the packet should
  still resolve them immediately before execution.
- Important live caveat: `wsl.exe --list --verbose` also shows
  `docker-desktop` running on WSL2. Treat Docker/HNS/WSL2 NAT reconciliation as
  a live stop condition before any WinNAT creation.
- Read-only Ubuntu ISO provenance discovery from official Ubuntu sources found
  current candidate `ubuntu-24.04.4-desktop-amd64.iso` from
  `https://releases.ubuntu.com/24.04/`, listed as Ubuntu 24.04.4 Noble Numbat,
  AMD64 desktop image, size 6.2G, modified 2026-02-10 01:41. `SHA256SUMS`
  entry: `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e`.
  `SHA256SUMS.gpg` verified in an isolated `/tmp` GnuPG home with Ubuntu CD
  Image Automatic Signing Key (2012), key ID `D94AA3F0EFE21092`, fingerprint
  `8439 38DF 228D 22F7 B374 2BC0 D94A A3F0 EFE2 1092`. The ISO itself was not
  downloaded. Treat this as artifact-intake evidence only, not download or
  execution approval.

Verified SCS closeout as of 2026-06-10 22:10 EDT:

- Halo verified SCS `main` aligned with `origin/main` at `8ff345f`, with only
  unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.
- Halo verified the final SCS hashes listed above. No host/VM/WSL/Docker/HNS/
  WinNAT/network/package/ISO/SCRIM/runtime mutation was run from Halo.

Superseded SCS in-progress state as of 2026-06-10 22:24 EDT:

This was superseded by the verified `e8ef33a` closeout below. It remains here
only as the review trail for the blockers gumo fixed before committing.

- SCS owner gumo acknowledged hcom `#28096` that the current SCS dirty PRD/docs
  edits are his and that the prior frozen hashes are stale until he commits or
  updates them.
- Halo observed SCS dirty state after the `8ff345f` checkpoint:
  `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`,
  `project_isolation_lab/README.md`,
  `project_isolation_lab/docs/ACCEPTANCE_GATES.md`,
  `project_isolation_lab/docs/PRD_PLAN.md`,
  `project_isolation_lab/docs/ROADMAP.md`,
  `project_isolation_lab/tasks/prd-001-hyperv-linux-vm-baseline.md`, plus new
  untracked `project_isolation_lab/docs/PRD_ACCEPTANCE_REVIEW_2026-06-10.md`
  and unrelated untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`.
- Halo's latest read-only pre-exec review remains `WAIT`, not a formal
  `$mutation-script-wave` GO. Open findings sent to gumo:
  1. Stage B2 must fail closed on local ISO existence, expected SHA256 via
     `Get-FileHash`, and artifact-intake approval/evidence reference before
     `Add-VMDvdDrive`.
  2. Optional GUI NAT stage must require exact network-stage preflight
     immediately before `New-NetIPAddress` / `New-NetNat`, or defer to the
     reviewed PowerShell B4 command shape only.
  3. In-VM gateway/DNS acceptance checks such as `ping 172.29.240.1` and
     `getent hosts archive.ubuntu.com` must be scoped to `NETWORK`; `OFFLINE_VM`
     acceptance should not expect them to pass.
  4. SCS should decide whether `project_isolation_lab/evidence/` is intentionally
     tracked. If not, it needs a `.gitignore` entry before future ISO intake so
     checksum/evidence files or accidental artifacts do not become repo noise.
- At that point, Limux's recorded SCS hashes were intentionally not updated
  until gumo reported a pushed commit and Halo verified the new hashes locally.

Verified SCS PRD/packet closeout as of 2026-06-10 22:33 EDT:

- SCS commit:
  `e8ef33a docs(lab): record prd acceptance gates`
- Full SCS commit SHA:
  `e8ef33ab190f76f605d369412b957b6e17e74636`
- Halo verified SCS `main` aligned with `origin/main`, with only unrelated
  untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.
- Verified final hashes:
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
- Gumo's reported verification for SCS commit `e8ef33a`: `git diff --check`;
  HTML parser; embedded JS `node --check`; `python3 -m py_compile
  security_posture/supply_chain_watch.py
  tests/security_posture/test_supply_chain_watch.py`; `python3 -m unittest
  tests.security_posture.test_supply_chain_watch -v` with 18 tests OK; and
  `git diff --cached --check` before commit.
- Decision remains `WAIT`; this is not a formal `$mutation-script-wave` GO, not
  ISO download/use approval, and not execution approval.
- No host/VM/WSL/Docker/HNS/WinNAT/network/package/ISO/SCRIM/runtime mutation
  was run by Halo or reported by gumo for this docs-only commit.

Superseded SCS mutation-wave packet draft as of 2026-06-10 22:46 EDT:

This draft state was superseded by SCS commit `7427285` recorded below. It
remains here only as the review trail for hcom `#28584`, `#28712`, and `#28723`.

- SCS is dirty again after `e8ef33a`; do not treat the `e8ef33a` hash set as
  the current frozen packet set until gumo commits/pushes or explicitly
  supersedes the draft.
- Gumo patched after hcom `#28606`. Halo sent a clean read-only closeout in
  hcom `#28712`; gumo acked in hcom `#28723` and is rerunning local SCS
  verification before commit/push. Treat hashes as current draft evidence, not
  a frozen committed packet, until gumo reports a pushed commit.
- Observed SCS status:
  - modified: `FYI.md`
  - modified: `README.md`
  - modified: `docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`
  - modified: `project_isolation_lab/FYI.md`
  - modified: `project_isolation_lab/README.md`
  - modified: `project_isolation_lab/docs/ACTIVE_GOAL.md`
  - modified: `project_isolation_lab/docs/PRD_PLAN.md`
  - modified: `project_isolation_lab/docs/ROADMAP.md`
  - untracked: `project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`
  - unrelated untracked: `SECURITY_VM_SETUP_AND_LIMUX.code-workspace`
- Draft packet hash from Halo's initial 22:41 read-only check:
  - `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
    `5397bd6cea8bf18d15e265920692341f8430d41e9ad2f298a370847ed517e2ab`
- Gumo patched after that check. Later volatile hashes observed at 22:44:
  - `HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md`:
    `b2ce7b60b7ce429dd2c7e9db4786a01e548e6bd21b01ff730e515fb14c58e0bc`
  - `PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html`:
    `4d4cb6824b38e7592ae863025323d9c856bac810f12734a6386a3d6a42740af1`
  - `ACTIVE_GOAL.md`:
    `9b4cd6fa81326d5f7749bb2b1e35adc35b7b3c2d588a43a53673b4323fcc947f`
  - `ACCEPTANCE_GATES.md`:
    `a3279d7b87ca40fd6bcff74dc483dc1f92fb4372bad2730c92152345ee281404`
- Other checked draft hashes still matched the `e8ef33a` artifact set:
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
- Read-only Halo review: the new mutation-wave packet is directionally aligned
  because it keeps Decision `WAIT`, states it is not a completed wave or
  execution approval, splits future review into Wave A ISO intake, Wave B
  offline Hyper-V baseline, and Wave C optional network stage, and keeps Wave C
  deferred behind Docker/HNS/WSL2 NAT reconciliation.
- Open docs-drift findings sent to gumo in corrected hcom `#28584`:
  1. HTML copy-back still says the approved next step is generic Hyper-V packet
     review/freeze. It should say Wave A ISO intake packet review/freeze, or
     explain why generic Hyper-V review remains the approved next step.
  2. Mutation-wave packet wording says the recommended first executable scope is
     Wave B offline baseline, while the recommended next action says Wave A ISO
     intake first. Clarify this as first VM mutation scope vs first formal
     review/artifact-intake scope.
- Gumo acked in hcom `#28606` and is patching. He also reported Claude-side
  findings: Wave B is too broad across the B0-B3/reboot boundary, convergence
  criteria are underspecified, gate docs are missing from the hash set, and Wave
  A wording is too executable-looking while the ISO packet is still a stub.
- Halo closeout `#28712`: requested findings appear addressed. Wave A is first
  formal review/artifact-intake scope; Wave B is a review grouping split across
  B0-B3, not a single pasteable run; convergence criteria require five lenses,
  cross-family/non-converged blocker handling, timeout/failure as
  non-convergence, zero unresolved CRITICAL/HIGH findings, and no silent GO
  after three rounds; gate-authority hashes include `ACCEPTANCE_GATES.md` and
  `ACTIVE_GOAL.md`; Wave A says it is not yet a frozen command packet.
- Ignore/cross-check hcom `#28569`: shell backtick substitution stripped inline
  phrases and corrupted the NAT hash. Corrected message is hcom `#28584`.
- `git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY diff --check` was clean from
  Halo's read-only check.
- Decision remains `WAIT`; formal `$mutation-script-wave` has not run or
  converged, ISO download/use is not approved, host/VM/network/package/runtime
  mutation is not approved, and Halo ran no such mutation.

Verified SCS mutation-wave packet closeout as of 2026-06-10 22:53 EDT:

- SCS commit:
  `7427285 docs(lab): add hyperv mutation wave packet`
- Full SCS commit SHA:
  `7427285b267bba3d69483c3354edf504299e2956`
- Halo verified SCS `main` aligned with `origin/main`, with only unrelated
  untracked `SECURITY_VM_SETUP_AND_LIMUX.code-workspace` remaining.
- Verified final hashes:
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
- Gumo's hcom closeout `#28853` reported verification: `git diff --check`,
  HTML parser, embedded JS `node --check`, `py_compile`, and unittest 18 OK.
- Halo verification: `git diff --check`; HTML parser; embedded JS extracted to
  `/tmp/scs_project_isolation_packet.js` and checked with `node --check`;
  no-write Python syntax compile with `compile(...)`; `python3 -B -m unittest
  tests.security_posture.test_supply_chain_watch -v` with 18 tests OK; commit
  SHA and upstream SHA matched.
- Note: direct `python3 -m py_compile ...` from Halo failed because the SCS tree
  is read-only from this Limux sandbox and `py_compile` tried to write
  `__pycache__`; this is why Halo used no-write compile plus `python3 -B`
  unittest as local verification. Gumo reported normal `py_compile` passed from
  the SCS-owned session.
- Decision remains `WAIT`; this is not formal `$mutation-script-wave` GO, not
  ISO download/use approval, not host/VM/network execution approval, and not a
  package/runtime/global-config mutation approval.
- No host/VM/WSL/Docker/HNS/WinNAT/network/package/ISO/SCRIM/global-config/
  runtime mutation was run by Halo or reported by gumo for this docs-only
  commit.

## Immediate Next Action

Start from the Project Isolation Lab goal above. The next practical action for
this Limux session is to keep Limux stable as a tool and coordinate with gumo on
the SCS-owned lab docs, not to add more Limux features by default.

If resuming after a restart, first verify SCS is still aligned at `7427285` with
the final hashes above. Then continue toward Wave A ISO intake packet
review/freeze first. Wave A is review/freeze only until a future frozen command
packet is written and reviewed; do not download the ISO. Wave B offline Hyper-V
baseline cannot attach media without an accepted ISO evidence record, and Wave C
network stage remains deferred behind Docker/HNS/WSL2 NAT reconciliation. Do
not execute host/VM/network/artifact mutation from Limux.

```bash
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY status --short --branch
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY log -5 --oneline --decorate
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_PACKET_REVIEW_RECORD_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/PRD_ACCEPTANCE_REVIEW_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_MUTATION_SCRIPT_WAVE_REVIEW_PACKET_2026-06-10.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/PROJECT_ISOLATION_LAB_DECISION_PACKET_2026-06-10.html
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACCEPTANCE_GATES.md
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACTIVE_GOAL.md
hcom --version --name halo
hcom list --name halo
wsl.exe --list --verbose
wsl.exe --status
hcom events --last 80 --agent gumo --name halo
```

Current Limux setup is unblocked for local use from this checkout. The install
posture is still a **repo-local/user-local launcher**, not a polished system
package. The public entrypoint is available on `PATH` through user-local
symlinks:

```bash
limux --help
limux
limux-cli --help
```

Both `/home/riche/.local/bin/limux` and
`/home/riche/.local/bin/limux-cli` resolve to
`/home/riche/MCPs/limux/scripts/limux-dev`. The launcher executes
`target/release/limux-cli`, points it at `target/release/limux` through
`LIMUX_HOST_BIN`, and prepends `ghostty/zig-out/lib` to `LD_LIBRARY_PATH`.
`/home/riche/.local/bin` is already on `PATH`.

New user-facing behavior added on 2026-06-10:

```bash
limux agent-team --agents codex,claude --launch-mode hcom --cwd "$PWD"
limux review spawn --review-id <review-id> --launch-mode hcom
```

`--launch-mode hcom` keeps the Limux pane model but starts peers as
`hcom codex --run-here`, `hcom claude --run-here`, etc. Default behavior is
unchanged: without the flag, Limux launches bare `codex`, `claude`, `opencode`,
or `gemini`. The hcom mode is committed and pushed as
`a0f4e34 feat(cli): add hcom launch mode`.

Current verification on 2026-06-10 after the hcom-mode work:

```bash
./scripts/xvfb-smoke-test.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/check.sh
limux --help
limux agent-team --dry-run --agents codex,claude --launch-mode hcom --cwd /tmp/limux-hcom-dry-run-check --protocol-path /tmp/limux-hcom-dry-run-check/LIMUX_AGENTS.md --roster-path /tmp/limux-hcom-dry-run-check/LIMUX_TEAM_ROSTER.md --ledger-path /tmp/limux-hcom-dry-run-check/LIMUX_REVIEW_LEDGER.md --force-protocol-overwrite --force-roster-overwrite
rg -n "hcom codex --run-here|hcom claude --run-here" /tmp/limux-hcom-dry-run-check/LIMUX_AGENTS.md
```

All passed. The earlier `./scripts/check.sh` and Xvfb smoke failures in a live
Limux pane were caused by inherited `LIMUX_*` variables pointing tests at the
operator's real pane. That is fixed and pushed as
`678de2c fix(scripts): isolate verification from live limux env`; the scripts
now clear inherited live pane/socket env before running isolated checks.

Do not run `scripts/package.sh`, generated install scripts, or sudo/system
install lanes for immediate use unless the operator explicitly approves that
separate mutation/security gate. Zig is still not expected on `PATH`; the
current runtime uses the already-built `ghostty/zig-out/lib/libghostty.so`.

Gumo/SUPPLY_CHAIN_SECURITY owns the broader Project Isolation Lab lane. The safe
direction is no longer "make Limux more complete first"; it is to build the
reusable lab sequence: persistent full isolated Linux VM, evidence export/intake
gate, disposable full Linux VM factory, disposable WSL ergonomics checks only
after artifacts are no longer first-touch hostile inputs, then Firecracker
microVMs after full VM foundations. Any later Limux near-full/user-local install
must be handled as an acceptance case behind those gates, with manifest,
hashes, rollback, artifact-intake review, and explicit approval.

Known feature caveat from the user discussion: the colored per-terminal
"waiting for input" border/tab marker is not implemented. Existing Limux
support is workspace/sidebar unread attention via `limux notify` and agent hook
notification translation. Treat a true per-tab/per-pane waiting marker as a
future feature, not current product behavior.

Phase 5D2 reviewer spawn/evidence pointer wrapper is implemented and verified.
The current `agent-team` flow still writes protected generated protocol to
`LIMUX_AGENTS.md`, seeds `LIMUX_TEAM_ROSTER.md` and
`LIMUX_REVIEW_LEDGER.md` when missing, launches peer panes with bare commands
by default or hcom run-here commands when requested, waits for pane readiness,
sends each peer a sanitized one-line bootstrap prompt after all coordination
files exist, then submits it with explicit Enter. `--no-bootstrap`,
`--no-launch`, and `--dry-run` skip prompt sends. `agent-team --dry-run` still
materializes the generated protocol and seeds missing roster/ledger files; it
only skips host contact.

Review workflow:

```bash
limux review prepare \
  --artifact <path-or-ref> \
  --reviewer <codex|claude|gemini|opencode|manual> \
  --lens <security|correctness|maintainability|ux|release> \
  --summary <short-review-goal>

limux review spawn --review-id <review-id>
```

`review prepare` creates `reviews/<review-id>.md`, appends a pending entry to
`LIMUX_REVIEW_LEDGER.md`, and prints the exact reviewer prompt. It does not
contact the Limux host or launch reviewers. `--dry-run` previews paths,
Markdown, and prompt text without writing files.

`review spawn` reads an existing generated request, refuses `manual`
reviewers, creates one reviewer terminal pane through the live `pane.create`
path, sends the prepared prompt with `surface.send_text` plus explicit Enter,
writes `reviews/<review-id>.evidence.md`, and updates only the matching pending
ledger entry to `in-progress`. `--dry-run` validates request/ledger/evidence
paths without host contact. `--no-launch` creates the pane without typing the
reviewer command or prompt.

Recommended next scoped action for user utility: use the current Limux launcher
and hcom mode normally, while treating new engineering work as isolation-lab
support unless the operator explicitly asks for Limux product development.

Current implementation status: local launcher setup, env-isolated verification
scripts, and hcom launch mode are committed and pushed through
`a0f4e34 feat(cli): add hcom launch mode`. Verify with
`git status --short --branch` before continuing.

Restart closeout check on 2026-06-06 18:58 EDT: no new Limux project scope has
started after Phase 5D2 closeout. The tracked worktree was clean and `main` was
aligned with `origin/main` before this docs refresh. If resuming after reboot,
start with the Phase 5D3 recommendation below.

Historical next-step packet that selected Phase 5D1:

```text
docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.md
docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.html
```

Historical Limux feature sequence, now superseded by the Project Isolation Lab
goal unless explicitly reactivated:

1. **Done:** Phase 5D1 `limux review prepare` scaffold.
2. **Done:** Phase 5D2 reviewer spawn/evidence pointer wrapper.
3. **Next:** Phase 5D3 review collect/complete plus consensus and cross-team
   hcom pointer conventions.
4. **Later:** machine-readable roster/ledger adapters only if Markdown sidecars
   are not enough for automation.

Current verification baseline:

```bash
cargo fmt --check
git diff --check
cargo test -p limux-cli review_spawn
cargo test -p limux-cli review
cargo test -p limux-cli agent_team
cargo test -p limux-cli
cargo clippy -p limux-cli --all-targets -- -D warnings
LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/xvfb-smoke-test.sh
```

The operator requested an easier-to-read status/options artifact on
2026-05-29. Use this packet if a decision needs to be confirmed before coding:

```text
docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html
```

For the historical post-Phase-5C decision that led to Phase 5D1, see:

```text
docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.html
```

For the current install-prerequisite decision, use:

```text
docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html
```

The mutation review for the selected bounded host prerequisite lane is:

```text
docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md
```

The operator approved the exact command block in that file on 2026-05-29.
Artifact SHA256 at approval and pre-run verification:
`de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec`.

Initial Codex execution attempt status: `BLOCKED BEFORE MUTATION`. The pre-mutation evidence
and apt simulation ran, then execution stopped at `sudo apt-get update` because
sudo required a password. The run was cancelled instead of collecting or
handling a password in chat. No apt package install occurred, and `pkg-config`
was still absent at that point.

Second Codex attempt status: `STILL BLOCKED BEFORE MUTATION`. After the operator ran
`sudo -v` locally, Codex checked `sudo -n true` in its execution context. Sudo
still returned `sudo: a password is required`, which indicates the local sudo
cache did not carry into the Codex PTY/session. No package mutation occurred.

Manual operator execution status: `APT PREREQUISITES INSTALLED`. The operator
ran the approved apt lane manually in a trusted terminal. Post-install checks
show `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and
`libwebkitgtk-6.0-dev` installed. `pkg-config --modversion gtk4 libadwaita-1
webkitgtk-6.0` reports `4.14.5`, `1.5.0`, and `2.52.3`.

Previous blocker resolved: the host test now finds
`ghostty/zig-out/lib/libghostty.so`. The `ghostty/` submodule is initialized at
the pinned commit, and project-scoped Zig `0.15.2` was used from
`$HOME/.cache/limux-tools`. Zig is still not installed system-wide and is not
expected on `PATH`.

The draft-only Ghostty/Zig mutation review for that next gate is:

```text
docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md
```

The operator approved the exact v2 command block on 2026-05-29. Current v2
artifact SHA256:
`dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`.

Consensus gate result was `GO for explicit operator approval; WAIT for
execution`. Reviewers `niru`, `zori`, `kazu`, and the local Claude plugin
cleared v2 for approval consideration. The consensus report is:

```text
docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md
```

Execution result: `COMPLETE WITH WRAPPER DEVIATION DOCUMENTED`. The v2 lane used
project-scoped Zig `0.15.2` from official Zig metadata, SHA256
`02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`, the pinned
`am-will/ghostty` submodule commit
`81ab8ffa90185221782baf785e85387321e16f8d`, and evidence under:

```text
docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/
```

Focused host verification passed:

```bash
CARGO_NET_OFFLINE=true LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" cargo test --locked -p limux-host-linux surface_send_text_response
```

Result: `2 passed; 0 failed; 186 filtered out`. The later follow-up removed the
`unused_mut` warning at `rust/limux-host-linux/src/window.rs:4340`, and the full
workspace gate now passes.

Execution wrapper deviation: the command extraction wrapper captured all
`bash` fences in the review doc, so it first ran the README illustrative block:
`git submodule update --init --recursive` and
`(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)`. The build
failed immediately because `zig` was not on `PATH`; the approved v2 block then
ran successfully. Follow-up inspection found `ghostty` at the pinned commit,
`ghostty/.gitmodules` absent/non-empty check passed, and
`git -C ghostty submodule status --recursive` returned no nested submodules.

Start here:

```bash
git status --short --branch
sed -n '1,220p' HANDOFF.md
sed -n '70,130p' docs/cmux-parity-plan.md
sed -n '210,285p' docs/limux-hcom-workflow.md
rg -n "run_agent_team|build_agents_md|LIMUX_AGENTS|LIMUX_TEAM_ROSTER|LIMUX_REVIEW_LEDGER" rust/limux-cli/src/main.rs
```

Phase 5A completed in `rust/limux-cli/src/main.rs`:

1. Added a generated-file marker to `LIMUX_AGENTS.md`.
2. Added an `Instruction Sources` section that detects `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`.
3. The section references those files directly instead of copying or merging their contents.
4. Metadata includes path, modified time, and deterministic `fnv1a64` content hash for regular files.
5. Repo instruction files stay authoritative; `LIMUX_AGENTS.md` only adds runtime topology and messaging protocol.
6. Existing unmarked `LIMUX_AGENTS.md` files are refused by default; `--force-protocol-overwrite` is required to replace one.
7. `LIMUX_AGENTS.local.md` is documented as the durable local policy sidecar; Limux does not create or overwrite it.

Phase 5B completed in `rust/limux-cli/src/main.rs`,
`rust/limux-host-linux/src/window.rs`,
`rust/limux-host-linux/src/terminal.rs`, and `scripts/xvfb-smoke-test.sh`:

1. Added `--no-bootstrap` for live `agent-team` runs.
2. Kept generated pane-create commands as bare launchers such as `codex` or
   `claude`; arbitrary orientation text is sent only after the pane is created.
3. Wrote `LIMUX_AGENTS.md` before any bootstrap prompt send.
4. Sanitized generated bootstrap prompts more strictly than normal typed text:
   no CR, no tab, no LF, no bidi format controls, and no zero-width
   display-spoofing characters.
5. Sent the prompt through `surface.send_text`, then submitted it through
   `surface.send_key enter` so shells that treat paste/newline conservatively
   still receive the message.
6. Made live smoke use fake `codex`/`claude` binaries to prove the prompt was
   received after protocol write.
7. Fixed Ghostty Enter key submission for command-launch paths.

Phase 5C completed in `rust/limux-cli/src/main.rs` and
`scripts/xvfb-smoke-test.sh`:

1. Added default `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md`
   coordination files.
2. Added `--roster-path <path>`, `--ledger-path <path>`, and
   `--force-roster-overwrite`.
3. Seeded the roster and ledger when missing before any live bootstrap prompt.
4. Preserved existing roster and ledger files by default; the ledger remains
   create-if-missing only.
5. Rejected symlink, non-regular, and overlapping roster/ledger/protocol
   targets.
6. Pointed generated `LIMUX_AGENTS.md` and bootstrap prompts to the durable
   roster and ledger.
7. Expanded CLI tests and Xvfb smoke proof for creation, preservation, forced
   marked-roster replacement, unmarked force refusal, symlink refusal,
   overlapping-path refusal, and fake-agent file visibility.

Phase 5D1 completed in `rust/limux-cli/src/main.rs`:

1. Added `limux review prepare` as a host-independent scaffold.
2. Created review request files under `reviews/` atomically with `create_new`.
3. Seeded `LIMUX_REVIEW_LEDGER.md` if missing, then appended pending entries
   without rewriting existing ledger content.
4. Added `--dry-run`, `--review-id`, `--reviews-dir`, and `--ledger-path`.
5. Validated reviewer/lens choices and rejected control characters in prompt
   fields.
6. Rejected leaf symlink review directories, leaf symlink/non-regular ledgers,
   existing request files, and overlapping request/ledger paths. Parent path
   components are not recursively audited for symlinks; use trusted output
   directories.
7. Updated README, roadmap, workflow Markdown/HTML, decision packet, handoff,
   and FYI docs.

Recommended acceptance tests:

```bash
cargo test -p limux-cli review
cargo test -p limux-cli agent_team
cargo test -p limux-cli
cargo fmt --check
cargo clippy -p limux-cli --all-targets -- -D warnings
LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/xvfb-smoke-test.sh
git diff --check
```

`libghostty.so` and host GTK/pkg-config prerequisites are present locally. The
focused host warning is fixed, and the full workspace/Xvfb gates now pass with
the local Ghostty library. The smoke script exports `LD_LIBRARY_PATH`
automatically when `ghostty/zig-out/lib` exists; the full check still needs the
explicit `LD_LIBRARY_PATH` prefix.

## Completed This Session

| Time | Item | Result |
|---|---|---|
| 2026-05-29 early AM | Limux vs Multica decision packet | Created readable Markdown and dark-mode HTML decision guide with copy-back selections. |
| 2026-05-29 early AM | Multica adoption decision | User chose to keep Limux + hcom primary and defer Multica until after Limux fixes. |
| 2026-05-29 early AM | Global `$html-decision-packet` update request | Routed to `@kazu`; Kazu completed Sources/Evidence support in the global pattern/template/skill. |
| 2026-05-29 01:37 EDT | `agent-team` clobber fix | Commit `cec067f` changed default protocol output from `AGENTS.md` to `LIMUX_AGENTS.md` and added `--protocol-path`. |
| 2026-05-29 01:40 EDT | Verification | `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. |
| 2026-05-29 01:40 EDT | Full quality gate | `./scripts/check.sh` failed only because `libghostty` was missing; build prerequisite: `cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast`. |
| 2026-05-29 02:00 EDT | Subagent brainstorm | Five native subagents converged on reference-based instruction discovery, not silent inheritance/copying. |
| 2026-05-29 02:06 EDT | Stop-point docs | Created this handoff and FYI entry; refreshed decision/workflow docs for morning resumption. |
| 2026-05-29 17:00 EDT | Phase 5A implementation | Added generated marker, instruction-source metadata, no-overwrite guard, explicit force flag, local policy sidecar docs, and regression tests. |
| 2026-05-29 17:00 EDT | Verification | `cargo test -p limux-cli agent_team`, `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. |
| 2026-05-29 17:00 EDT | Cross-family review attempt | Claude plugin read-only review timed out after 120 seconds without findings; do not treat it as a passed review. |
| 2026-05-29 17:29 EDT | GTK send-text readiness fix | Updated the live GTK `surface.send_text` handler to convert `TerminalHandle::send_text == false` into a conflict error. Added focused unit tests for the response helper. |
| 2026-05-29 17:29 EDT | Verification | `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. `cargo test -p limux-host-linux surface_send_text_response` is blocked because `pkg-config` is missing. |
| 2026-05-29 17:44 EDT | Host prerequisite mutation review | Created draft-only mutation review for apt prerequisites. Decision is `WAIT` pending explicit approval. Zig/Ghostty remain separate gates. |
| 2026-05-29 19:07 EDT | Approved prerequisite block attempt | Verified artifact SHA, ran the pre-mutation evidence and apt simulation, then stopped at the first sudo command because a password was required. No packages were installed. |
| 2026-05-29 19:24 EDT | Sudo cache follow-up | Operator ran `sudo -v` locally, but `sudo -n true` inside Codex still required a password. No packages were installed. |
| 2026-05-29 19:51 EDT | Manual apt prerequisite completion | Operator manually completed the approved apt prerequisite lane. GTK/WebKit pkg-config checks pass. Host test now fails at the separate Ghostty/Zig gate. |
| 2026-05-29 20:10 EDT | Ghostty/Zig mutation review | Created draft-only review for project-scoped Zig 0.15.2 download, pinned Ghostty submodule initialization, `libghostty.so` build, and host test verification. Decision is `WAIT` pending explicit approval. |
| 2026-05-29 20:15 EDT | Ghostty/Zig consensus gate | `niru`, `zori`, `kazu`, and Claude plugin reviewed v1, returned `WAIT`, v2 was patched, then v2 re-review returned GO for operator approval. |
| 2026-05-29 20:32 EDT | Approved Ghostty/Zig execution | Verified v2 SHA, built `ghostty/zig-out/lib/libghostty.so`, captured evidence logs, and passed the locked offline host send-text test. Wrapper deviation documented: an earlier README bash fence initialized the top-level `ghostty` submodule before the approved v2 block; no nested submodules or system mutation were found. |
| 2026-05-29 20:47 EDT | Full gate and Xvfb smoke restored | Removed the host `unused_mut` warning, updated Xvfb smoke from `softpipe`/OpenGL 3.3 to `llvmpipe`/OpenGL 4.3, accepted current `new-pane --json` refs, and verified `cargo fmt --check`, `git diff --check`, `./scripts/check.sh`, and `./scripts/xvfb-smoke-test.sh`. |
| 2026-05-29 21:10 EDT | Shell-quoted launch snippet hardening | Added central generated-snippet shell quoting, quoted generated `LIMUX_AGENTS.md` scratch-pane commands, rejected unquoted extra `new-pane` positionals, removed nested prompt examples from docs, and verified focused CLI tests, full workspace check, and Xvfb smoke. Claude plugin review timed out; hcom reviewers converged on GO for the manual snippet path and deferred typed-PTY control-character policy before auto-bootstrap. |
| 2026-05-29 21:36 EDT | Typed-PTY control-character guard | Added shared typed-text validation in `limux-protocol`; enforced it in the CLI, standalone core dispatcher, live GTK bridge parser, and GTK host send sink; documented `send-key` as the control-key route; expanded Xvfb smoke stage 7 to reject ESC/BEL/C1 payloads across send/new-pane/respawn/paste/new-workspace. Claude plugin review timed out after 240 seconds, so it is not counted as passed; hcom reviewers `kazu`, `zori`, and `niru` had already converged on the policy shape. |
| 2026-05-29 22:31 EDT | Phase 5B automatic bootstrap | Added post-launch `agent-team` bootstrap prompts, `--no-bootstrap`, protocol-write-before-send behavior, stricter generated-prompt validation, explicit Enter submission, command-launch Enter fixes, fake-agent Xvfb proof, and refreshed workflow/decision/handoff docs. |
| 2026-05-29 23:23 EDT | Phase 5C durable roster and review ledger | Added `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` seeding, `--roster-path`, `--ledger-path`, `--force-roster-overwrite`, no-overwrite ledger preservation, marked-roster force replacement, symlink/nonregular/overlapping path refusal, bootstrap pointers, CLI tests, Xvfb fake-agent file-visibility proof, and refreshed workflow/decision/handoff docs. |
| 2026-05-30 00:20 EDT | Phase 5D1 reviewer workflow scaffold | Added `limux review prepare` with durable request-file creation, append-only pending ledger entries, dry-run planning, reviewer/lens validation, leaf symlink/nonregular/overlapping target refusal, control-character prompt-field rejection, README/roadmap/workflow updates, focused CLI tests, full workspace check, release Xvfb smoke proof, and Claude adversarial review follow-up fixes. |
| 2026-05-30 02:19 EDT | End-of-night closeout | Confirmed Phase 5D1 commit `e4ce6fd` is pushed to `main`, working tree is clean, and the next resume lane is Phase 5D2 reviewer spawn/capture wrapper. |
| 2026-06-05 14:08 EDT | Phase 5D2 reviewer spawn/evidence pointer | Added `limux review spawn` from existing prepared requests, live reviewer pane creation, prompt send/Enter submission, evidence pointer file creation, targeted pending-ledger entry update to `in-progress`, dry-run host avoidance, README/roadmap/workflow updates, focused RED/GREEN tests, full workspace check, and Xvfb smoke proof. |

## Current State

- Branch: `main`
- Phase 5B baseline commit: `0d2597b feat(cli): bootstrap agent-team peers after launch`
- Latest implementation commit: `20f5785 feat(cli): add review spawn wrapper`
- Latest implementation in this handoff: Phase 5D2 `limux review spawn` wrapper after Phase 5D1 `review prepare`.
- Working tree was clean and aligned with `origin/main` immediately after the
  Phase 5D2 implementation push and again before the 2026-06-06 restart docs
  refresh.

## Architectural Decisions Locked In

1. **No silent inheritance.** `LIMUX_AGENTS.md` should not copy, merge, or reinterpret `AGENTS.md` by default.
2. **Authority split.** Repo files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` remain authoritative project instructions.
3. **Runtime sidecar.** `LIMUX_AGENTS.md` is generated Limux runtime context: peers, surfaces, messaging, human notification, and routing.
4. **Durable coordination files.** `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` are create-if-missing durable operator files, not generated overwrite targets. Live surface/pane/workspace IDs remain in `LIMUX_AGENTS.md`.
5. **Zero-friction path.** Reduce friction through automated discovery, explicit pointers, environment variables, bootstrap, and later adapters, not hidden prompt composition.
6. **Launch automation waits.** Generated launch snippets should only start the agent binary. Automatic launch/bootstrap sends bounded prompt text only after protocol/roster/ledger write and pane readiness through guarded typed-text plus explicit `send-key enter`.

## Subagent Brainstorm Synthesis

The proposed future shape:

```text
AGENTS.md / CLAUDE.md / GEMINI.md = project instructions
LIMUX_AGENTS.md                  = generated runtime protocol
LIMUX_AGENTS.local.md            = optional durable local team policy
LIMUX_TEAM_ROSTER.md             = durable project/team routing roster
LIMUX_REVIEW_LEDGER.md           = durable review/consensus ledger
.limux/ adapters                 = later tool-specific discovery helpers
```

Phase ordering:

1. **Done:** Improve generated `LIMUX_AGENTS.md` with instruction-source detection, generated marker, no-overwrite guard, and local-policy extension point.
2. **Done:** Fix `surface.send_text` readiness/failure semantics in the GTK host bridge and verify it through the full workspace gate.
3. **Done:** Add caller-shell quoting tests and generated-snippet hardening before expanding automatic launch/bootstrap behavior.
4. **Done:** Define and test the typed-PTY control-character policy for `limux send`, respawn, paste-buffer, `pane.create --command`, `workspace.create --command`, direct socket callers, and the live GTK host sink.
5. **Done:** Implement two-phase automatic bootstrap: launch the agent binary, wait for pane readiness, then send prompt text through guarded `surface.send_text` plus explicit Enter.
6. **Done:** Seed a project/team roster and durable review/consensus ledger.
7. **Done:** Add Phase 5D1 reviewer workflow scaffold: review request files,
   pending ledger entries, generated reviewer prompts, dry-run planning, and
   path/prompt safety checks without launching real reviewer panes.
8. **Done:** Add Phase 5D2 reviewer spawn/evidence pointer wrapper: launch a
   reviewer pane from an existing request, send the prepared prompt, create an
   evidence pointer, and update the matching pending ledger entry.
9. **Next:** Add review collect/complete plus consensus/cross-team broadcast
   conventions.
10. **Optional later:** Add runtime-specific `.limux/` adapters for Codex, Claude Code, Gemini, and OpenCode.

## Key Files For Context

| File | Purpose |
|---|---|
| `/home/riche/MCPs/limux/rust/limux-cli/src/main.rs` | `agent-team`, `review prepare`, `review spawn`, protocol generation, hook setup, tests. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | GTK bridge command handling; `surface.send_text` now errors if terminal injection reports not-ready. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/terminal.rs` | `TerminalHandle::send_text` returns `false` when the Ghostty surface is not realized. |
| `/home/riche/MCPs/limux/docs/cmux-parity-plan.md` | Roadmap and current open bridge/protocol work. |
| `/home/riche/MCPs/limux/docs/limux-hcom-workflow.md` | Operator workflow for Limux plus hcom. |
| `/home/riche/MCPs/limux/docs/limux-vs-multica-decision-guide.md` | Decision record for Limux vs Multica and selected path. |
| `/home/riche/MCPs/limux/docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.md` | Detailed next-step options after Phase 5C. |
| `/home/riche/MCPs/limux/docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.html` | Dark-mode selectable next-step packet with copy-back payload. |
| `/home/riche/MCPs/limux/docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html` | Dark-mode copy-back packet for selecting the next implementation path. |
| `/home/riche/MCPs/limux/FYI.md` | Append-only session journal. |

## Critical Behavior Rules

- Do not modify repo `AGENTS.md` as part of `agent-team` runtime protocol generation.
- Do not implement hidden prompt inheritance. Use explicit detected source references.
- Do not launch hcom-managed workers for bounded local repo work unless a persistent cross-tool runtime is actually needed.
- Preserve `limux agent-team --dry-run` without a running host.
- Preserve `--no-launch` and `--no-bootstrap` behavior for `agent-team`;
  neither path should send bootstrap prompts.
- Preserve existing `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` by
  default. The ledger is append/manual state and must not be overwritten by
  `agent-team`. `--force-roster-overwrite` is only for marked Limux rosters.
- Preserve `limux review prepare` as a file-first scaffold: create request
  files atomically, append ledger entries, refuse leaf symlink/non-regular or
  overlapping request/ledger paths, and do not launch reviewers from this
  command. Use trusted output directories; parent path components are not
  recursively audited for symlinks.
- Preserve `limux review spawn` as a continuation of an existing generated
  request: refuse `manual` reviewers, keep `--dry-run` host-free, make
  `--no-launch` skip prompt injection, create evidence pointers with
  `create_new`, and update only the matching pending ledger entry rather than
  rewriting unrelated ledger content.
- Use `apply_patch` for manual edits.
- Do not edit `/home/riche/.claude` from this Limux session.

## Known Risks And Blockers

- `ghostty/zig-out/lib/libghostty.so` exists locally after the approved build gate, but it is a generated artifact. Fresh clones or cleaned worktrees must rebuild it through the reviewed lane before host/workspace checks.
- Host-crate tests moved past the prior `pkg-config` and `libghostty` blockers. The `unused_mut` warning at `rust/limux-host-linux/src/window.rs:4340` is fixed.
- The Xvfb smoke harness requires Mesa software OpenGL 4.3 for the pinned Ghostty. It now defaults to `llvmpipe` and can be overridden with `LIMUX_SMOKE_GALLIUM_DRIVER` for local Mesa debugging.
- `zig` is intentionally not on `PATH`; the reviewed lane used project-scoped Zig under `$HOME/.cache/limux-tools`.
- Caller-shell generated snippet tests now cover spaces, quotes, `$`, command substitution, backticks, semicolons, control characters, newlines, exact JSON preservation, and side-effect inertness.
- Typed-PTY control characters are now rejected everywhere the current control surface can inject typed terminal text. Intentional control keys must use `surface.send_key` / `limux send-key`.
- Bootstrap prompt generation now rejects CR, tab, LF, bidi format controls, and zero-width display-spoofing characters even though the broader typed-text policy still allows tab/LF/CR for manual multiline messages. Keep that stricter boundary for generated automatic prompts.
- Instruction-source hashes are deterministic `fnv1a64` metadata for change detection, not cryptographic integrity claims.
- Claude plugin adversarial review did not complete for the shell-quoting lane: normal mode timed out after 180 seconds, and `--bare` mode failed because Claude was not logged in under bare mode. hcom reviewer `kazu` provided the Claude-family shell-safety lens instead. For the typed-PTY lane, the normal plugin review timed out after 240 seconds and is not counted as passed.
- Claude plugin adversarial review completed for Phase 5B. It found no security-blocking defect, but flagged reliability issues that were handled before commit: removed trailing-LF double submission, made fail-fast partial-side-effect behavior explicit in the error path, and widened the command-launch readiness budget. Residual: live smoke uses fake instant agents, so real Codex/Claude cold-start/TUI readiness remains a future robustness target.
- Phase 5C roster/ledger files are Markdown coordination surfaces, not an
  automatic source of truth. Agents still need to keep owners, hcom names,
  related teams, and ledger entries current during work.
- Phase 5D2 starts reviewers and points to evidence, but it does not parse
  reviewer output, collect verdicts, update final verdicts, or resolve
  consensus. Phase 5D3 should build collect/complete and consensus conventions
  on top of the request/evidence/ledger surfaces.

## Morning Resume Prompt

```text
Please resume the Limux work from HANDOFF.md. Phase 5A zero-friction protocol discovery, GTK `surface.send_text` readiness/failure reporting, shell-quoted launch snippets, typed-PTY control-character guards, Phase 5B automatic `agent-team` bootstrap, Phase 5C durable roster/review-ledger seeding, Phase 5D1 `limux review prepare`, and Phase 5D2 `limux review spawn` are implemented and verified. Host prerequisites are installed, the approved Ghostty/Zig gate built `ghostty/zig-out/lib/libghostty.so`, `./scripts/check.sh`, and debug Xvfb smoke pass locally. Recommended next implementation is Phase 5D3: a review collect/complete path that records reviewer verdicts back into the existing ledger entry without rewriting unrelated content, followed by GO/WAIT/NO-GO consensus and targeted hcom pointer conventions.
```
