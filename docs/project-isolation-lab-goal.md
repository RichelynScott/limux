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

As of 2026-06-10 21:19 EDT, SCS commit
`62440b6 docs(lab): record isolation goal and hyper-v packet` is pushed. The
SCS-owned Hyper-V host mutation packet exists as a draft with decision `WAIT`.
It is not approved to run.

Recorded hashes:

- `ACTIVE_GOAL.md`:
  `4504bed0ce5cac9bca42974f3d4655296251ddc949515af489cce6bcba732da2`
- `HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md`:
  `badc15cf44a8a6be8f56231b09899ee3c2e756ff0308dd6758b70ccd9d3ca678`

A preliminary Limux-side read found the earlier WinNAT/static-guest-network gap
addressed: the draft defines guest IP `172.29.240.10/24`, gateway
`172.29.240.1`, DNS, GUI configuration steps, and a netplan fallback with an
interface-name stop condition. Gumo also added HNS/WSL2 NAT reconciliation,
separate switch/NAT names, control-plane distro and hcom placeholders, and
offline install posture language.

Halo sent a Codex single-reviewer system-mutation pre-exec review to gumo on
hcom thread `project-isolation-lab-goal`. Decision: `WAIT`, not formal
`$mutation-script-wave`. Must-fix items sent to gumo:

1. Path B PowerShell block must hard-stop after
   `Enable-WindowsOptionalFeature -NoRestart` or be split into stages.
2. Documented stop conditions for free space, RAM, Windows edition,
   virtualization, BitLocker recovery, pending reboot, and host network health
   need fail-closed enforcement or explicit manual-gate separation.
3. Evidence capture should not use reusable paths that can overwrite pre-state
   evidence on rerun.
4. `HCOM_CHECK_NAME` and `CONTROL_PLANE_WSL_DISTRO` need conservative
   validation before WSL/bash interpolation.
5. Path A should make the first-boot network-disconnected state deterministic.

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

Before any execution can be considered, the packet still needs freeze +
SHA256, exact Ubuntu ISO provenance and SHA256, formal `$mutation-script-wave`,
resolution of any high/critical findings, and explicit operator approval for
the execution window.

Verify SCS state before relying on the recorded pointers:

```bash
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY status --short --branch
git -C /home/riche/Proj/SUPPLY_CHAIN_SECURITY log -5 --oneline --decorate
sha256sum /home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/HYPERV_HOST_MUTATION_PACKET_DRAFT_2026-06-10.md
hcom --version --name halo
hcom list --name halo
wsl.exe --list --verbose
wsl.exe --status
hcom events --last 80 --agent gumo --name halo
```
