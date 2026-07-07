# Limux Wave 1 Morning Summary - 2026-07-07

Author: lifo
Branch: `lifo/wave1-morning-summary-20260707`
Base: `origin/main` at `eff00d788b22159fa1c97df88e9ce0273d2a1f48`

## Current Result

Wave 1 has moved from setup and PRD import into merged runtime fixes and
parallel follow-on implementation lanes. PRD-E 2a and 2b are merged to `main`;
PRD-G and PRD-H are active in separate worktrees.

No live user Limux install was restarted or replaced as part of this manager
lane.

## Merged To Main

| PR | Main SHA | Scope | Notes |
|---|---:|---|---|
| #20 | `4f5d27a` | Runtime trust and build identity | Shipped build identity and doctor support. |
| #21 | `897fccd` | Ghostty resource packaging | Shipped valid Ghostty resources and terminfo. |
| #22 | `67508d6` | Pane attention overlay and flag colors | Pane notification overlay/color groundwork merged. TaskMaster still needs final status reconciliation. |
| #23 | `bde48a5` | Post-install checklist staging | Added `docs/verification/post-install-checklist-v1.md`, `run-template.md`, and runs directory. |
| #24 | `c0ac4f2` | PRD-E 2a core mirror API | Added `limux-core` control-state mirror API and fixed bot-caught dangling current-id import validation. |
| #25 | `eff00d7` | PRD-E 2b host read-only fallthrough | Added deny-by-default route registry and read-only core fallthrough for `window.list` and `window.current`; fixed bot-caught focused-pane snapshot issue. |

Open PR check at summary creation: no open PRs.

## Active TaskMaster State

Tag: `cmux-parity-20260707`

| Task | Status | Current lane |
|---|---|---|
| #1 Runtime Trust | done | Merged in #20. |
| #2 Ghostty resources | done | Merged in #21. |
| #3 Post-install checklist/live run | in-progress | Checklist/run-template staged; first live run still pending. |
| #4 Pane attention overlay | review | Code merged in #22; TaskMaster status should be reconciled after final owner review. |
| #5 Live-bridge parity core fallthrough | in-progress | PRD-E 2a/2b merged; later mutation/capability/kill-switch slices remain. |
| #6 Browser command bridge ratification | pending | Not started in this wave. |
| #7 Agent lifecycle/sidebar | in-progress | Worker `mupa` started in `WORKTREES/prd-g-agent-sidebar-20260707`. |
| #8 Session restore pack | in-progress | Worker `hena` started in `WORKTREES/prd-h-restore-pack-20260707`. |

## Active Worktrees

| Lane | Path | Branch | Owner | Definition of done |
|---|---|---|---|---|
| PRD-G first slice | `WORKTREES/prd-g-agent-sidebar-20260707` | `lifo/prd-g-agent-sidebar-20260707` | `mupa` | Pure agent lifecycle state module, focused tests, clippy/fmt, pushed PR. |
| PRD-H first slice | `WORKTREES/prd-h-restore-pack-20260707` | `lifo/prd-h-restore-pack-20260707` | `hena` | Cwd inheritance helper/wiring, focused tests, clippy/fmt, pushed PR. |
| Morning summary | `WORKTREES/wave1-morning-summary-20260707` | `lifo/wave1-morning-summary-20260707` | `lifo` | This summary plus durable TaskMaster status sync. |

## Verification Evidence

PRD-E 2a before merge:

- `cargo fmt --check`
- `cargo test -p limux-core --lib -- --nocapture`
- `cargo clippy -p limux-core --all-targets -- -D warnings`
- `git diff --check`

PRD-E 2b before merge:

- `cargo fmt --check`
- `cargo test -p limux-host-linux -- --nocapture`
- `cargo clippy -p limux-host-linux --all-targets -- -D warnings`
- `cargo test -p limux-host-linux snapshot_current_pane_id_prefers_valid_focused_pane -- --nocapture`
- `git diff --check`

## Known Caveats

- The spent PRD-E 2b worktree
  `WORKTREES/prd-e-fallthrough-2b-20260707` still has an uncommitted
  `.taskmaster/tasks/tasks.json` edit from initially marking #7 and #8
  in-progress there. The same intended TaskMaster state is now being recorded
  in this clean summary branch. Do not remove that spent worktree until the
  duplicate dirty state is reviewed and reconciled.
- Task #4 is still `review` in TaskMaster even though PR #22 is merged. Treat
  it as a status reconciliation item, not necessarily an implementation gap.
- PRD-C still needs the first live post-install run before any stable install
  promotion decision.
- PRD-F is still pending and should not be conflated with PRD-E fallthrough.

## Next Actions

1. Let `mupa` and `hena` finish their bounded PRD-G/PRD-H slices, then verify
   their commits, PRs, and test output before reporting completion.
2. Run the PRD-C post-install checklist against the intended runtime candidate
   before promoting any build into the user's stable Limux install.
3. Reconcile TaskMaster statuses for #4 and #5 after owner review confirms
   whether the remaining PRD-E slices should stay under #5 or become separate
   tasks.
4. Resolve the duplicate dirty TaskMaster edit in the spent PRD-E worktree only
   after this summary branch is pushed and the state is durable.
