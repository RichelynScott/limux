# PRD-H: Session Restore Correctness Pack

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-07 00:00 UTC
**Purpose:** Make session restore trustworthy across the operator's normal life
event — WSL2/host restarts — by guaranteeing split geometry + pane identity
round-trips, preserving cwd when splitting from live panes, and adding
recently-closed / focus-history recovery. Research db cmux-20260702-004/008/009
(all direct/high); cmux precedent #4130/#6146/#7165/#6892.

- **Priority:** P1 (Wave 1 — roadmap W1.4). Independent of PRD-E/F/G — parallelizable.
- **Dependencies:** none hard. PRD-C checklist gains a restore section from this PRD.
- **Effort:** M
- **Channel targeting:** preview until PRD-C checklist passes

## Problem Statement

Limux already restores workspaces from `session.json`
(`rust/limux-host-linux/src/layout_state.rs`; a 10-workspace restore was
verified headless on 2026-06-22). But three correctness gaps remain, mirrored
by cmux fixes upstream:

1. **Split geometry/identity fidelity** (cmux PR #6892 class): split order,
   ratios, and pane identity are not guaranteed to round-trip exactly —
   there is no test asserting byte-level layout equivalence after
   save→restore, and drag-adjusted ratios are the operator-visible casualty.
2. **cwd loss on split/new-tab** (cmux PR #7165 / issue #7155 class): a new
   split/tab from a pane hosting a resumed agent or deep working directory
   starts in the default cwd, not the source pane's — high-friction for the
   agent-heavy workflow Limux exists for.
3. **No recovery from accidental close** (cmux v0.64.11 class): closing a
   workspace/tab is irreversible — no recently-closed list, no focus history.

## Goals

1. Save→restore round-trips split structure exactly: order, orientation,
   ratios (within float tolerance), pane identity, tab metadata (incl. PRD-D
   `flag_color` if landed).
2. New splits/tabs inherit the source pane's current working directory.
3. Recently-closed workspaces/tabs are listed and reopenable; workspace focus
   history supports back/forward.

## User Stories

### US-1: As the operator, my layout survives a restart exactly
- [ ] A scripted kill/restart harness (Xvfb): build a 3-workspace layout with
      mixed H/V nested splits at non-default ratios, pinned + renamed tabs;
      snapshot `session.json`; kill host; relaunch; assert the restored
      layout's serialized state is equivalent (structural equality; ratio
      float tolerance ±0.5%).
- [ ] Split RATIOS restore to their drag-adjusted values (explicit assertion —
      not just structure).
- [ ] Restore preserves per-tab metadata: title, pinned, unread, and
      `flag_color` when PRD-D is merged (feature-gated assertion).
- [ ] Save is atomic (write-temp + rename) and versioned: `session.json`
      gains an additive `schema_version` field; loader accepts absent version
      (current files) — never refuses to load an old file it can partially
      restore.
- [ ] A deliberately truncated/corrupt `session.json` produces a clean
      degraded start (empty session + renamed `.corrupt-<ts>` preservation of
      the bad file + one log line) — never a crash loop (test).
- [ ] Contributor-docs note added: pane-local tab ids (e.g. multiple
      `terminal-0` across panes) are VALID — restore code must never
      "deduplicate" them (codifies the 2026-06-22 lesson).

### US-2: As the operator/agent, new panes start where I am
- [ ] `surface.split` / `new-pane` (no explicit `--cwd`) inherits the SOURCE
      pane's live cwd, resolved via the pane's shell-process cwd
      (`/proc/<shell-pid>/cwd` readlink; the pane's child pid is already
      tracked for PTY lifecycle — confirm the exact handle at implementation
      and document it in the brief).
- [ ] Fallbacks in order: source-pane live cwd → workspace cwd → `$HOME`;
      resolution failure is silent-fallback, never an error to the user.
- [ ] Explicit `--cwd` always wins (regression test).
- [ ] `new-workspace` behavior UNCHANGED (workspace-level cwd semantics stay;
      test pins this).
- [ ] Works from resumed-agent panes (fixture: pane whose shell cd'd post-
      spawn; split inherits the new dir — the exact cmux #7155 case).
- [ ] Restored panes (from session.json) restore their last-known cwd as the
      spawn cwd (captured at save time per pane, additive field).

### US-3: As the operator, I can undo a close and walk focus history
- [ ] Closing a workspace or tab pushes a recently-closed entry (name, cwd,
      layout snapshot for workspaces; title/cwd for tabs), capped ring buffer
      (default 20).
- [ ] `limux recent-closed [--json]` lists entries; `limux reopen-closed
      [--index N]` re-creates the most-recent (or Nth) entry — workspace
      reopen restores its layout snapshot (terminal processes start fresh;
      no process resurrection).
- [ ] Focus history: `workspace.select` pushes history;
      new methods `workspace.back` / `workspace.forward` navigate it
      (classified in the PRD-E registry when present; additive methods
      otherwise), with CLI verbs + default keybinds wired through the
      existing shortcut-config system (rebindable, no reserved-terminal
      chords per `docs/shortcut-remap-testing.md` rules).
- [ ] Recently-closed + focus history persist across restart in
      `session.json` (additive fields; ring caps enforced on load).
- [ ] GUI affordance: recently-closed submenu in the existing workspace/
      sidebar menu surface (match wherever workspace-level menus live today).

## Functional Requirements

1. Geometry/identity work confined to `layout_state.rs` + `split_tree.rs`
   (+ `window.rs` restore wiring); pure round-trip logic unit-testable
   headless.
2. cwd capture: per-pane additive `last_cwd` in `session.json` at save; live
   inheritance at split time from `/proc`; keep the resolution helper in a
   pure module with injected proc-root for tests.
3. Recently-closed/focus-history state machine in a pure module; ring-buffer
   caps and serialization tested without GTK.
4. New socket methods (`workspace.back/forward`, recently-closed list/reopen)
   land in BOTH the core dispatcher and the live bridge path (or PRD-E
   registry if merged first — coordinate at import time; do not fork the
   method-adding pattern).
5. All `session.json` changes additive; older builds must load newer files
   (serde unknown-field tolerance — verified by a cross-version fixture test)
   with the documented caveat that older builds drop the new fields on save.

## Non-Goals

- No scrollback-content restore (cmux #4130 class — explicitly W2+/stretch;
  large storage + privacy surface).
- No process/agent-session resurrection (reopen = fresh shells; agent resume
  is W3.1 territory).
- No crash-snapshot service (cmux #5175 class — later).
- No recently-closed for individual SPLITS (workspace + tab granularity only
  in v1).

## Technical Considerations

- `/proc/<pid>/cwd` readlink races with process exit — treat every step as
  fallible with the fallback chain; never block the split on it (readlink is
  O(1), but guard with a short timeout pattern anyway if the handle requires
  process-group walking).
- Shell vs foreground process: the SHELL's cwd is the correct signal (agents
  cd via their shell); do not walk to the foreground child. Document this
  choice — it is the cmux-compatible semantic.
- Ratio serialization: store as f64 fractions of the parent allocation (or
  keep whatever `split_tree.rs` uses today if already fractional — discovery
  step in the first brief; do NOT store pixels, which break across monitor/
  scale changes — WSLg scale changes are common).
- Focus-history push must ignore programmatic rapid-fire selects (e.g.
  restore-time selection storm) — only user/CLI-initiated `workspace.select`
  pushes (flag on the internal call).
- session.json size: layout snapshots for 20 recently-closed workspaces are
  small (structure only, no scrollback) — assert serialized size stays <256KB
  in the ring-buffer test to catch accidental content capture.

## Success Metrics

- PRD-C checklist restore section: operator closes Limux with a customized
  3-workspace layout, relaunches, and confirms identical layout + a split
  from a deep-cwd pane landing in that cwd.
- Zero restore-related data-loss reports across the following month of
  operator use (tracked via FYI).

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-host-linux layout_roundtrip -- --nocapture
cargo test -p limux-host-linux cwd_inheritance -- --nocapture
cargo test -p limux-host-linux recent_closed -- --nocapture
cargo test -p limux-core focus_history -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended: kill/restart round-trip + split-cwd assertion
```

## Rollback Plan

All session.json fields additive — revert commits and older behavior returns;
existing session files remain loadable throughout (both directions covered by
the cross-version fixture test). Focus-history/recent-closed methods
unclassified → `-32601` after revert.

## Open Questions

1. Default keybinds for workspace back/forward — propose Ctrl+Alt+Left/Right
   (avoid reserved terminal chords); confirm against the shortcut-contract
   review (W2.3) if it lands first.
2. Should `reopen-closed` restore the workspace at its original sidebar
   position or append at end? Proposed: original position when free, else end.
