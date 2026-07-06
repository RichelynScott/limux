# PRD-D: Pane Attention Border Layering + Per-Pane Color Flags

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:20 UTC
**Purpose:** Fix the one live operator-visible UI defect promoted into Wave 0 —
the pane attention border renders behind pane content (mostly invisible on
right-hand panes) — and add deterministic per-pane color flags, per TaskMaster
task #20 and the existing plan note.

- **Priority:** P0 (Wave 0 — roadmap W0.5; TaskMaster #20)
- **Dependencies:** none. Verified live via the PRD-C checklist once both land.
- **Effort:** S
- **Execution model:** lifo + subagents; `./scripts/check.sh` gate per commit
- **Channel targeting:** preview build; live confirmation via PRD-C checklist

## Problem Statement

The attention system marks a pane needing operator input by adding the
`.limux-pane-attention` CSS class with a hover-clear timer
(`rust/limux-host-linux/src/pane.rs` — `mark_pane_needs_attention` lineage;
`window.rs` focus/attention wiring). Live evidence (screenshot
`docs/future-improvements/screenshots/limux-pane-attention-border-layering-20260701.png`,
plan note `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md`)
shows the border drawn *behind* pane content — largely hidden on the right
pane — so the operator misses exactly the signal the feature exists to give.
Separately, with many concurrent agent panes there is no way to visually tag
panes (research db item cmux-20260702-013: deterministic workspace/pane color
flags, preserving unread-state semantics; upstream precedent cmux PR
#6994/#6981 and upstream-Limux PR #92).

## Goals

1. The attention border is always fully visible above pane content, on every
   pane position (left/right/top/bottom splits), at every split ratio.
2. Operator can assign a per-pane color flag that persists across restarts and
   is visible on the pane and its tab, without breaking unread semantics.
3. Keep attention (transient, auto-clearing) and color flags (durable,
   user-set) semantically distinct.

## User Stories

### US-1: As the operator, I can actually SEE which pane needs attention
- [ ] Attention border renders above terminal content on all four sides of
      the flagged pane — implementation direction: draw the indicator on a
      `GtkOverlay` layer above the pane's content widget (or equivalent
      top-layer widget), not as a CSS border on a node that content overlaps.
- [ ] Verified for: single pane, H-split right pane, V-split bottom pane,
      nested 3-pane layout, and after drag-resizing the split — via Xvfb
      screenshot assertions (`debug.window.screenshot` +
      `debug.panel_snapshot` already exist in the debug surface) or a
      widget-tree unit test asserting the overlay z-order.
- [ ] Hover-clear behavior unchanged (existing timer semantics preserved —
      regression test).
- [ ] Border remains visible with background opacity configured
      (`sanitize_background_opacity` path).
- [ ] No `Gtk-CRITICAL` output introduced across the Xvfb suite.

### US-2: As the operator, I can color-tag panes to tell agents apart
- [ ] New `tab-action` actions: `set_flag_color <color>` and `clear_flag_color`
      (CLI: `limux tab-action --action set_flag_color --color <named|#hex>`),
      routed like existing `tab.action` verbs (CLI → bridge → core).
- [ ] A small fixed named palette (≥6 colors) + `#rrggbb` accepted; invalid
      color → clean error, no partial state.
- [ ] Flag color renders as a compact, always-visible indicator on the pane's
      tab AND a subtle pane-edge accent, visually distinct from both the
      attention border and unread styling.
- [ ] Flag color persists in `session.json` (layout_state) and restores on
      relaunch — restore test added beside the existing session-restore tests.
- [ ] GUI affordance: context-menu entry on the tab (submenu of palette
      colors + clear) — matching wherever existing tab actions (pin/rename)
      surface today.

### US-3: As a developer, semantics stay clean
- [ ] Attention (transient) and flag color (durable) are separate state
      fields — setting/clearing one never mutates the other (unit test).
- [ ] Unread dot/styling unchanged when a flag color is set (regression test
      on the sidebar/tab unread path).
- [ ] `sidebar-state --workspace <id>` output extended additively with flag
      color info; no existing fields renamed.

## Functional Requirements

1. Rendering fix in `rust/limux-host-linux/src/pane.rs` (+ `window.rs` wiring
   if the overlay must be owned by the split container in
   `split_tree.rs`).
2. State: extend the pane/tab model + `layout_state.rs` serialization with
   `flag_color: Option<String>`; additive `session.json` field (older builds
   ignore it — confirm serde tolerates unknown fields both directions).
3. Protocol: extend `tab.action` in `rust/limux-core/src/lib.rs` AND the GTK
   bridge path so both the standalone dispatcher and live GUI accept the new
   actions; CLI `tab-action` gains `--color`.
4. CSS: new classes (e.g. `.limux-pane-flag-<name>`), colors defined once.

## Non-Goals

- No per-WORKSPACE color flags (sidebar workspace coloring exists via
  highlight color; this PRD is pane/tab-scoped).
- No auto-assignment of colors by agent type (manual only in v1).
- No attention-behavior redesign (sound, blink, escalation — unchanged).
- No settings-editor UI for the palette (fixed palette v1).

## Technical Considerations

- ID plumbing: tab ids are uuid `String`s pane-locally (`LIMUX_SURFACE_ID` =
  `"{pane_id}:{tab_id}"`); reuse the exact resolution path `tab.action`
  already uses — no new ID scheme.
- The 2026-06 session.json lesson: 9× `terminal-0` tab ids are pane-local and
  VALID — do not "normalize" existing session files while adding the field
  (defect inventory §E5).
- `PaneCallbacks` has one constructor — adding a field will surface every
  construction site via the compiler (repo CLAUDE.md pitfall; expected).
- Attention overlay must not steal input: ensure the overlay layer is
  non-interactive (`set_can_target(false)`) so clicks pass through.

## Success Metrics

- Operator confirms (PRD-C checklist item) the attention border is clearly
  visible on a right-hand split pane — the exact reported failure case.
- Color flags survive a full close/relaunch cycle on the operator machine.

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-host-linux attention -- --nocapture
cargo test -p limux-host-linux flag_color -- --nocapture
cargo test -p limux-core tab_action -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended with flag-color + attention-overlay assertions
```

## Rollback Plan

`git revert` the feature commits. `session.json` field is additive/optional —
older builds ignore it; no migration needed. If the overlay approach causes
regressions on WSLg, fall back to the current CSS-border rendering (defect
severity returns to status quo, not worse).

## Open Questions

1. Named palette choice — propose: red, orange, yellow, green, blue, purple
   (GNOME accent-adjacent). Operator may adjust at review.
2. Should flag color also tint the workspace sidebar row when any flagged pane
   exists in it? (Default: no — keep sidebar semantics untouched in v1.)
