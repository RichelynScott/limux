# Limux Wave 1 Morning Summary - 2026-07-07

Author: lifo
Branch: `lifo/wave1-morning-summary-20260707`
Base: `origin/main` at `34babf3dd378fe871e950c370d670686fe04008e`

## Current Result

Wave 1 has moved from setup and PRD import into merged runtime fixes and
parallel follow-on implementation lanes. PRD-E is partially landed: 2a core API
and the 2b `window.list` / `window.current` read-only fallthrough slice are
merged to `main`; the registry, remaining read fallthroughs, Wave 1 mutation
set, and kill-switch remain open under TaskMaster #5.

PRD-G slice 1 and PRD-H US-2 are also merged to `main`. The PRD-F F1 decision
doc skeleton now exists, but it is explicitly provisional: no browser spike
code has started, no measurements are filled in, and F2 remains blocked on
ratification.

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
| #27 | `bd368c4` | PRD-G slice 1 agent lifecycle state machine | Added pure agent-state module with aggregation, acknowledgement, stale decay, eviction, and tests; fixed bot-caught `Default` stale-window issue. |
| #28 | `34babf3` | PRD-H US-2 cwd inheritance | Added source-pane/requested-surface cwd inheritance for splits and `pane.create`; fixed bot-caught background-tab `surface_id` cwd issue. |

Open PR check at final summary update: PR #26 is the remaining wrap PR and is
intended to merge last.

## Active TaskMaster State

Tag: `cmux-parity-20260707`

| Task | Status | Current lane |
|---|---|---|
| #1 Runtime Trust | done | Merged in #20. |
| #2 Ghostty resources | done | Merged in #21. |
| #3 Post-install checklist/live run | in-progress | Checklist/run-template staged; first live run still pending. |
| #4 Pane attention overlay | review | Code merged in #22; TaskMaster status should be reconciled after final owner review. |
| #5 Live-bridge parity core fallthrough | in-progress | PRD-E 2a core API and the `window.list` / `window.current` fallthrough slice are merged; registry, remaining reads, Wave 1 mutation set, and kill-switch remain open. |
| #6 Browser command bridge ratification | in-progress | F1 decision-doc skeleton added at `docs/decisions/browser-pane-architecture-20260707.md`; evidence and ratification pending. |
| #7 Agent lifecycle/sidebar | in-progress | Slice 1 state machine merged in #27; hook integration, sidebar rendering, socket exposure, and CLI convenience remain. |
| #8 Session restore pack | in-progress | US-2 cwd inheritance merged in #28; round-trip restore tests, recently-closed/focus-history, and restart harness remain. |

## Worktree State

| Lane | Path | Branch | State |
|---|---|---|---|
| PRD-G first slice | `WORKTREES/prd-g-agent-sidebar-20260707` | `lifo/prd-g-agent-sidebar-20260707` | Clean, merged in #27. |
| PRD-H first slice | `WORKTREES/prd-h-restore-pack-20260707` | `lifo/prd-h-restore-pack-20260707` | Clean, merged in #28. |
| PRD-E 2b spent lane | `WORKTREES/prd-e-fallthrough-2b-20260707` | `lifo/prd-e-fallthrough-2b-ffupdate-20260707` | Clean; duplicate accidental TaskMaster edit reconciled. |
| Morning summary | `WORKTREES/wave1-morning-summary-20260707` | `lifo/wave1-morning-summary-20260707` | Owns this final summary, TaskMaster sync, and PRD-F skeleton. |

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

PRD-G slice 1 before merge:

- `cargo fmt --check`
- `cargo test -p limux-host-linux agent_state -- --nocapture` (22 passed)
- `cargo test -p limux-host-linux -- --nocapture` (279 passed)
- `cargo clippy -p limux-host-linux --all-targets -- -D warnings`

PRD-H US-2 before merge:

- `cargo fmt --check`
- `cargo test -p limux-host-linux cwd_inheritance -- --nocapture` (7 passed)
- `cargo test -p limux-host-linux pane_create_surface_cwd_override -- --nocapture` (1 passed)
- `cargo test -p limux-host-linux -- --nocapture` (265 passed)
- `cargo clippy -p limux-host-linux --all-targets -- -D warnings`

## PRD-C Live-Run Instructions

The first live post-install verification run is still pending. Use:

- Checklist: `docs/verification/post-install-checklist-v1.md`
- Run template: `docs/verification/run-template.md`
- Run output path:
  `docs/verification/runs/<YYYYMMDD>-<install-id>.md`

Required shape:

1. Update `main` to the source SHA under test.
2. Build Ghostty resources and release Limux binaries as shown in the checklist.
3. Install the isolated preview lane with
   `scripts/user-local-install/install-user-local.sh --apply --profile release --channel preview --install-id "$install_id" ...`.
4. Run only `~/.local/bin/limux-preview` and
   `~/.local/bin/limux-preview-cli` for the checklist.
5. Record every checklist item as `PASS`, `FAIL`, or `N/A` in a copied run
   file.
6. Promote to stable only after a full-run `PASS` from the same source SHA.

## PRD-F F1 Decision Skeleton

Skeleton added:
`docs/decisions/browser-pane-architecture-20260707.md`

Status: provisional and pending ratification. The skeleton records candidate
architectures, WSLg-weighted criteria, measurement plans, an empty decision
table, existing WebKit-pane disposition options, and explicit ratification
gates. It does not contain measurements, a recommendation, or spike code.

## Known Caveats

- Task #4 is still `review` in TaskMaster even though PR #22 is merged. Treat
  it as a status reconciliation item, not necessarily an implementation gap.
- PRD-C still needs the first live post-install run before any stable install
  promotion decision.
- PRD-E is not done. Treat merged PRs #24/#25 as the foundation plus the first
  fallthrough slice only; registry, remaining read fallthroughs, Wave 1 mutation
  set, and kill-switch still need follow-up under #5.
- PRD-G is not done. Slice 1 is merged, but hooks/sidebar/socket/CLI work
  remains under #7.
- PRD-H is not done. US-2 cwd inheritance is merged, but restore round-trip,
  recently-closed/focus-history, and restart harness work remain under #8.
- PRD-F F1 has a skeleton only. The decision evidence and ratification remain
  required before implementation.

## Next Actions

1. Run the PRD-C post-install checklist against the intended runtime candidate
   before promoting any build into the user's stable Limux install.
2. Ratify or revise the PRD-F F1 decision skeleton before starting browser
   architecture measurements or spike code.
3. Reconcile TaskMaster statuses for #4 and #5 after owner review confirms
   whether the remaining PRD-E slices should stay under #5 or become separate
   tasks.
4. Continue PRD-G hooks/sidebar/socket/CLI and PRD-H restore/focus-history work
   as separate follow-up slices.
