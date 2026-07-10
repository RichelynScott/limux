# Limux Crash/Restore Log Flood Incident - 2026-07-10

Owner: lifo
TaskMaster: master tag, task 21, high priority, review
Live build: `main-068872a1-reviewed` at `068872a1e162`

## Observed Incident

- The pre-restart Limux control request timed out while the GUI was frozen.
- The operator restarted Limux after installing the reviewed `068872a1` build.
- `/home/riche/.local/state/limux/logs/limux-host.log` reached
  `7,230,538,457` bytes and `123,596,797` lines.
- A 20,000-line tail sample contained 9,974 normalized repetitions of:
  `gtk_widget_get_parent: assertion 'GTK_IS_WIDGET (widget)' failed` from
  first restored host PID 96254 at 08:38:30 EDT.
- A three-second growth probe after recovery measured zero new bytes. The log
  flood is currently stopped; the evidence file has not been truncated,
  deleted, moved, or otherwise mutated.
- Replacement host PID 99037 restored the legacy socket and sessions. This
  Codex sandbox cannot see host PIDs because it has a private PID namespace;
  socket `system.identify` is the authoritative live-host check here.

## Root-Cause Lead

Two GTK reorder paths used the same unbounded `first_child/remove` shape:

- `SplitTreeContainer::trigger_rebuild` in
  `rust/limux-host-linux/src/split_tree.rs`;
- `sync_sidebar_row_order` in `rust/limux-host-linux/src/window.rs`.

If GTK rejects removal of a stale/invalid child, `first_child()` can return the
same child forever. That no-progress shape directly matches the sub-second
`gtk_widget_get_parent` flood and GUI freeze. The log does not identify which
caller entered the loop, so both paths require the same bounded guard.

## Restart Smoke

- Lifo hcom session `019ee13a-f948-7080-a37d-20dfad526aa1` stopped at event
  `#386488`, was recreated at `#386509`, and became ready at `#386511` with
  process binding, live delivery, hooks, terminal control, and the same session
  ID.
- Rumi restore showed a separate first-ready/PTY-killed/second-ready sequence
  plus pre-crash hcom DB-lock telemetry loss. Dino owns that correlation.
- Per Gile's P0 containment, Rumi surface
  `surface:74:007de0ea-4ac1-4c08-86b0-673c25464d45` was visually identified,
  then exited once through Limux-native `/exit`. The Codex TUI returned to the
  original `hermes-agent` shell; adjacent surface 78 was untouched.
- `target-info` reporting `connects=false` is intentional: it is a no-connect
  targeting command. `identify` without explicit caller flags intentionally
  reports the focused pane. Neither is a restart defect.

## Ephemeral Worktree Exception

Branch-in-place is insufficient because the primary checkout is on stale branch
`lifo/hermes-workspace-highlight-resize-20260627` with broad peer/unknown dirty
state. The P0 implementation therefore uses this short-lived exception:

- owner: lifo
- branch: `lifo/fix-restore-logflood-p0-20260710`
- base: fresh `origin/main`
- path: `/tmp/limux-restore-logflood-p0-20260710`
- shared Cargo target: `/tmp/limux-shared-target-20260710`
- expected lifetime: current P0 implementation/review session only
- durable pointer: pushed GitHub branch and PR
- cleanup condition: tests pass, branch is pushed, PR exists, and the worktree
  is clean; remove in the same session when no longer needed, otherwise record
  the exact blocker without force-removal

## Acceptance

1. Restored split-tree teardown cannot spin when child removal makes no
   progress.
2. Failure is bounded and diagnostic without a high-volume log loop.
3. Normal split/close/rebuild behavior and Ghostty GLArea lifecycle remain
   intact.
4. Focused host tests and the Xvfb restored-session smoke pass.
5. Restore retry/dedup behavior is classified separately with Dino; no live
   rollout occurs before source review.

## Implemented P0 Guard

`SplitTreeContainer::trigger_rebuild` and `sync_sidebar_row_order` now drain
children through one bounded progress guard. It aborts after the first
unchanged child or after a caller-sized attempt budget and emits one
caller-specific diagnostic. The split-tree caller supplies its one-direct-child
invariant; the sidebar supplies the current workspace count, so more than 64
valid rows remain supported. Split-tree teardown does not schedule a rebuild
after failure, and sidebar reorder does not append rows after failure. This
converts both observed unbounded-loop candidates into fail-closed bounded
failures while preserving the existing one-tick Ghostty/GLArea rebuild path
when normal removal succeeds.

Regression tests cover:

- a removal callback that makes no progress;
- normal multi-child draining;
- alternating children that evade a same-child check but hit the caller-sized
  attempt budget;
- 65 distinct children that all make progress and drain successfully.

## Verification

- RED: focused tests failed because `drain_children_with_progress` did not
  exist.
- GREEN: 4/4 focused child-drain tests passed.
- `cargo fmt --check`: passed.
- workspace clippy: passed with only the pre-existing
  `clippy::single_element_loop` baseline explicitly allowed at
  `control_bridge.rs:2411`.
- workspace tests: 524 passed, 0 failed (109 CLI, 26 control, 5 socket, 33
  core, 342 host, 9 protocol; doc tests also passed).
- safety-adapted Xvfb smoke: all stages passed twice. The stock harness was not run
  because it contains delete commands prohibited by current policy; a temporary
  archive-only copy passed `bash -n` and the no-delete static scan before use.
  The initial artifact is
  `/tmp/limux-smoke-archives-20260710/limux-smoke-WA6VhR-1783688407`; the
  post-sidebar-guard artifact is
  `/tmp/limux-smoke-archives-20260710/limux-smoke-Eg27JQ-1783688891`.
- No build was installed and the operator's live Limux runtime was not
  replaced or restarted by this implementation lane.
