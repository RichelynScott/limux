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

1. **Split geometry/identity fidelity — VERIFY + REGRESSION-TEST, not
   implement** (Codex-revised after code review): the machinery already
   exists — ratios persist as f64 fractions (`SplitState.ratio`,
   layout_state.rs:133-139), drag adjustments write back and trigger a save
   (`attach_split_position_persistence` → `request_session_save`,
   split_tree.rs:461-492 + window.rs:1235-1256), and restore reserves
   persisted pane ids (`pane_id_for_initial_state`, pane.rs:51-59). What is
   MISSING is any full-layout save→restore round-trip test (only
   agent-metadata and keybind-tab round-trips exist, layout_state.rs:1664,
   1902) — so fidelity is unproven, not known-broken.
2. **cwd loss on split — REAL** (cmux PR #7165 / issue #7155 class): Limux
   tracks live per-tab cwd via Ghostty's `GHOSTTY_ACTION_PWD` →
   `on_pwd_changed` → `term_cwd` (terminal.rs:908-921, pane.rs:1186-1189,
   exposed as `pane::tab_working_directory()`, pane.rs:1728-1736), yet
   `split_pane` seeds new panes from the WORKSPACE `folder_path`/`cwd`
   only (window.rs:5653-5679) — the tracked live cwd is never consulted.
   High-friction for the agent-heavy workflow Limux exists for.
3. **No recovery from accidental close — REAL** (cmux v0.64.11 class):
   zero recently-closed / focus-history code exists (verified: no
   `workspace.back/forward`, no `recent*` in core or bridge).

## Goals

1. Save→restore round-trips split structure exactly: order, orientation,
   ratios (within float tolerance), pane identity, tab metadata, and pane
   metadata (including PRD-D pane-scoped `flag_color` if landed).
2. New splits/tabs inherit the source pane's current working directory.
3. Recently-closed workspaces/tabs are listed and reopenable; workspace focus
   history supports back/forward.

## User Stories

### US-1: As the operator, my layout survives a restart exactly
- [ ] A scripted kill/restart harness (Xvfb) — (Codex-revised) this is a
      NAMED NEW DELIVERABLE, not an extension: `xvfb-smoke-test.sh` launches
      once and has no restart flow today, so the harness gains a
      second-launch mode (same session dir, ready-wait, post-restore
      snapshot diff). Scenario: 3-workspace layout with mixed H/V nested
      splits at non-default ratios, pinned + renamed tabs; snapshot
      `session.json`; kill host; relaunch; assert equivalence.
- [ ] Equivalence is defined against NORMALIZATION (Codex-revised —
      `normalize_session` mutates on save/load: ratio clamp 0.08–0.92,
      version stamp, active-tab fallback, within-pane tab-id dedupe,
      layout_state.rs:520-588, so byte equality fails spuriously): compare
      normalized-snapshot to normalized-snapshot; `pane_id` and
      `active_tab_id` ARE in the equality set; ratio tolerance ±0.5% with
      fixtures inside the clamp bounds (0.08–0.92).
- [ ] Split RATIOS restore to their drag-adjusted values (explicit assertion —
      not just structure).
- [ ] Restore preserves per-tab metadata: title, pinned, unread. When PRD-D is
      merged, restore also preserves pane-scoped `flag_color` as pane metadata,
      not a duplicate tab/surface field (feature-gated assertion).
- [ ] Atomic save + versioning are ALREADY IMPLEMENTED (Codex-revised:
      `save_session_atomic_in` does write-temp + rename,
      layout_state.rs:418-429; `AppSessionState.version` with
      `SESSION_VERSION=1` + serde default exists, layout_state.rs:11,45-46,
      stamped by `normalize_session:521`) — REUSE `version`, do NOT add a
      second `schema_version` field; add tests for the atomic path if
      untested. Loading is all-or-nothing today (any parse failure defaults
      the whole session) — partial restore is explicitly OUT of scope v1.
- [ ] Corrupt-file handling: degraded start already exists and is tested
      (`load_returns_empty_state_for_corrupt_canonical_file`,
      layout_state.rs:1254). The REAL gap (Codex-revised): the corrupt file
      is silently CLOBBERED by the next save. New work = preserve it as
      `session.json.corrupt-<ts>` + one log line before proceeding (test).
- [ ] Contributor-docs note added: pane-local tab ids (e.g. multiple
      `terminal-0` across panes) are VALID — restore code must never
      "deduplicate" them (codifies the 2026-06-22 lesson).

### US-2: As the operator/agent, new panes start where I am
- [ ] (Codex-revised — use the SHIPPED mechanism, not `/proc`: no shell/child
      pid handle exists anywhere in host-linux; libghostty spawns the shell
      internally. Limux already tracks live per-tab cwd via Ghostty's
      `GHOSTTY_ACTION_PWD` → `on_pwd_changed` → `term_cwd`,
      terminal.rs:908-921 / pane.rs:1186-1189, exposed as
      `pane::tab_working_directory()`, pane.rs:1728-1736 — already used for
      tab-drag workspace seeding, window.rs:3922.) Splits/new panes (no
      explicit `--cwd`) inherit the SOURCE pane's `term_cwd`.
- [ ] Fallbacks in order: source-pane `term_cwd` → workspace cwd → `$HOME`;
      resolution failure is silent-fallback, never an error to the user.
      Honest caveat, documented: pwd reporting requires shell integration
      (PRD-B ships it); a shell that never reports pwd falls back to
      workspace cwd — acceptable, stated.
- [ ] Dual/triple landing called out (Codex-required): the inheritance lands
      in (a) the GUI split path (`split_pane`, window.rs:5653-5679), (b) the
      bridge's `pane.create` route, and (c) core `surface.split`
      (core-only today, lib.rs:44 — zero matches in control_bridge.rs) —
      three distinct code paths, one shared resolution helper.
- [ ] Explicit `--cwd` always wins (regression test).
- [ ] `new-workspace` behavior UNCHANGED (workspace-level cwd semantics stay;
      test pins this).
- [ ] Works from resumed-agent panes (fixture: pane whose shell cd'd post-
      spawn; split inherits the new dir — the exact cmux #7155 case).
- [ ] Restored panes restoring last-known cwd is ALREADY IMPLEMENTED per-tab
      (Codex-revised: `snapshot_pane_state` saves live `state.cwd` into
      `TabContentState::Terminal { cwd }`, pane.rs:1664-1694; restore spawns
      with `cwd.as_deref().or(working_directory)`, pane.rs:1078) — REFRAME
      to verify + regression-test; no new field.

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
   headless. Cite `normalize_pane_tab_ids` (layout_state.rs:567-588) in the
   contributor-docs note: dedupe is within-one-pane only — cross-pane
   duplicate tab ids are valid.
2. cwd inheritance: shared resolution helper (`term_cwd` → workspace cwd →
   `$HOME`) in a pure module; per-tab persistence already exists (verify +
   test, US-2).
3. Recently-closed/focus-history state machine: pure module in
   **limux-host-linux** (Codex-revised — persistence lands in host
   `session.json` and `workspace.select` has two independent handlers, core
   lib.rs:4972 + bridge control_bridge.rs:587). The live bridge implements
   `workspace.back/forward` + recently-closed list/reopen; the core
   dispatcher gets mirror registrations for headless tests. The
   user/CLI-vs-programmatic push flag threads BOTH `workspace.select`
   handler paths. Ring-buffer caps and serialization tested without GTK;
   layout snapshots capped in depth to avoid pathological nesting.
4. Method landing coordinates with the PRD-E registry if merged first — do
   not fork the method-adding pattern.
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

- cwd signal semantics: `term_cwd` reflects the shell's OSC-reported pwd —
  the correct signal (agents cd via their shell); no process walking, no
  `/proc` (Codex-revised — the earlier `/proc/<shell-pid>/cwd` design was
  based on a false premise; no pid handle exists in host-linux).
- Ratio serialization: already f64 fractions (`SplitState.ratio`) — no
  discovery needed; never introduce pixel storage (WSLg scale changes are
  common).
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
