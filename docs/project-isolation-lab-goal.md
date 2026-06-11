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

- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/README.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/PRD_PLAN.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ROADMAP.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/project_isolation_lab/docs/ACCEPTANCE_GATES.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/PROJECT_ISOLATION_LAB_STRATEGY_2026-06-10.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/LIMUX_INSTALL_VM_LAB_STRATEGY_2026-06-10.md`
