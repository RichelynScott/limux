# Hamo Limux Manager Handoff

Last updated: 2026-07-19 EDT
Owner/session: hamo / LIMUX_MGR

## Immediate Next Action

The Limux v0.2.3 release unit is complete. Do not redo the release, merge, or
stable install. For new Limux work, start from current `origin/main`, run the
repository lane preflight, and inspect the active TaskMaster tag through the
reviewed wrapper's list command. Stable v0.2.3 is now the active runtime after
an operator-authorized force restart. Before restarting it again, inspect the
current user service, normal stable socket, `doctor`, and restored workspace
state rather than falling back to the legacy launcher.

## Current State

| Area | Verified state |
|---|---|
| Repository | `/home/riche/MCPs/limux` |
| Release/task-closeout baseline | `7c760f0c5d98f0e5f7a28c1849d0f4dc513d007e` |
| Release merge | PR #73, merge SHA `1a26bda0bd1c3d256f91eaa9c1fbc444d1375e6a` |
| Task closeout | PR #74, merge SHA `7c760f0c5d98f0e5f7a28c1849d0f4dc513d007e` |
| Stable install | `/home/riche/.local/limux-reviewed/stable/main-1a26bda0-v0.2.3-20260719` |
| Stable launchers | `/home/riche/.local/bin/limux-stable`, `/home/riche/.local/bin/limux-stable-cli` |
| Stable identity | `limux-cli 0.2.3 (1a26bda0bd1c, release) install-id=main-1a26bda0-v0.2.3-20260719 channel=stable` |
| Stable runtime state | Active through user unit `limux-stable-force-restart-20260719.service` on `/run/user/1000/limux/stable/limux.sock`; force-restart verification returned `doctor` `ok=true`, `exit_code=0` |
| Restored state | All 28 legacy workspaces were copied hash-identically into `/home/riche/.local/share/limux/stable/session/session.json`; focused and selected terminal surfaces were healthy after launch |
| Legacy runtime | v0.2.2 host stopped by explicit operator-authorized force restart; its install and launcher remain available but are not active |
| TaskMaster | Master-tag tasks 27 and 28 are `done`; active tag restored to `limux-resource-crash-20260716` |
| Boundary review | Dino GO #581117, `Boundary-Review: hcom` |

## Completed This Session

| Sequence | Item | Evidence |
|---|---|---|
| 1 | Committed the reviewed v0.2.3 release bytes. | `09d91c495bef7069c2f0b11008d62a6861947f4b` |
| 2 | Reconciled current `origin/main` plus the independently owned GTK input-lock/scrollbar fix from PR #72. | Reconciliation commit `0c06ddc14f96cb1b78d602d07fd79fb1906b5273`; PR #73 |
| 3 | Merged the release. | PR #73 merge SHA `1a26bda0bd1c3d256f91eaa9c1fbc444d1375e6a` |
| 4 | Ran the canonical release gates. | `./scripts/check.sh`; Ghostty resource validator; debug and release Xvfb smokes all passed |
| 5 | Built and installed exact merged release binaries to the isolated stable channel. | CLI and host identify as v0.2.3 at `1a26bda0bd1c` with no dirty marker |
| 6 | Verified the installed runtime under isolated D-Bus/Xvfb state. | Live socket identify passed; workspace restored; `surface-health` returned healthy; `doctor` returned `ok=true`, `exit_code=0` |
| 7 | Closed the release-related TaskMaster review tasks. | Tasks 27 and 28 marked `done`; PR #74 merged |
| 8 | Force-restarted the daily runtime onto stable v0.2.3. | Legacy PID 29087 stopped; stable host launched under `limux-stable-force-restart-20260719.service`; 28 workspaces restored; no stale sockets; `doctor` green |

## Key Files And Artifacts

- `/tmp/limux-release-0.2.3-evidence-20260719/` — retained boundary-review and
  release evidence moved outside the checkout.
- `/tmp/limux-installed-stable-smoke.BbXdSz/` — retained successful installed
  stable-channel smoke artifacts, including `doctor.json`.
- `/tmp/limux-release-0.2.3-20260719` — ephemeral release worktree retained for
  no-loss closeout; current tracked content is clean after this handoff lands.
- `/run/user/1000/limux-archive/force-restart-20260719T0515/` — archived,
  confirmed non-connectable legacy and failed-launch socket residuals, kept
  outside Limux's runtime scan root.
- `/home/riche/MCPs/limux/docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html`
  — Lifo-owned untracked file in the primary checkout; do not stage, modify, or
  remove it without Lifo/operator authorization.

## Critical Behavior Rules

- Stable is now the active daily runtime by explicit operator authorization.
  Keep the legacy launcher/install as rollback provenance; do not relaunch it
  concurrently or overwrite stable state without a new runtime decision.
- Do not merge, activate, or broaden HCOM/OMP behavior from this release
  closeout; the release only integrated the reviewed Limux product bytes.
- Preserve peer-owned dirt and use exact-path staging.
- Do not delete the retained `/tmp` worktree or evidence. A later cleanup owner
  may remove the registered worktree only after the documented no-loss gate and
  normal destructive/owner authorization.

## Separate Pending Lane

The active TaskMaster tag `limux-resource-crash-20260716` still contains its own
renderer, clean/unclean restore, pressure, and staged-preview tasks. Those are
separate from the completed v0.2.3 release unit and must be claimed and verified
under their existing task definitions rather than folded into release closeout.
