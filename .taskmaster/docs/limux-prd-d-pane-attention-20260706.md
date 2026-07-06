# PRD-D: Pane Attention Border Layering + Per-Pane Color Flags

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:20 UTC
**Purpose:** Fix the one live operator-visible UI defect promoted into Wave 0 —
the pane attention border renders behind pane content (mostly invisible on
right-hand panes) — and add deterministic per-pane color flags, per TaskMaster
task #20 and the existing plan note.

- **Priority:** P0 (Wave 0 — roadmap W0.5; TaskMaster #20)
- **Dependencies:** none. Verified live via the PRD-C checklist once both land.
- **Effort:** M (Codex-revised from S: includes the tab.action bridge route +
  ControlCommand + window wiring that does not exist today — see US-2)
- **Execution model:** lifo + subagents; `./scripts/check.sh` gate per commit
- **Channel targeting:** preview build; live confirmation via PRD-C checklist
- **Gate policy:** `./scripts/check.sh` per commit with **no NEW failures vs
  baseline** (repo CLAUDE.md documents a known-failing baseline test)

## Problem Statement

(Codex-revised — current-state baseline) The attention system marks a pane
needing operator input by adding the `limux-pane-content-attention` CSS class
(`rust/limux-host-linux/src/pane.rs:29`) to the pane's `content_overlay`,
rendered as an **inset box-shadow** (`PANE_CSS`, pane.rs:388-390), with a
hover-clear timer (`mark_pane_needs_attention` pane.rs:201,
`schedule_attention_clear` pane.rs:246). A partial retarget already landed
(commits `cedcb3a`/`c0a294c` lineage) — the OLD class name
`.limux-pane-attention` no longer exists, and the guard test
`pane_attention_css_targets_content_overlay_not_outer_shell` (pane.rs:3583)
asserts its ABSENCE (executors: update/extend that guard test, do not trip
it). The defect persists because a parent's inset box-shadow paints beneath
its children, including the GLArea terminal. Live evidence (screenshot
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
      nested 3-pane layout, and after drag-resizing the split — PRIMARY
      acceptance is a widget-tree unit test asserting the overlay z-order
      (overlay indicator is an `add_overlay` child of the pane's
      `content_overlay`, which is already a `gtk::Overlay`, pane.rs:552 —
      precedent: `content_drop_overlay` uses `add_overlay` +
      `set_can_target(false)`, pane.rs:561-562). (Codex-revised) Do NOT use
      `debug.window.screenshot`/`debug.panel_snapshot` as verification —
      they return mock/core-state output (`write_mock_png`, lib.rs:4821) and
      are not routed on the live bridge; real GTK pixel capture is out of
      scope for this PRD.
- [ ] Hover-clear behavior unchanged (existing timer semantics preserved —
      regression test).
- [ ] Overlay indicator widget is opaque and independent of terminal
      background opacity (`sanitize_background_opacity` path,
      window.rs:2137) — asserted structurally (widget opacity property),
      not visually.
- [ ] No `Gtk-CRITICAL` output introduced across the Xvfb suite.

### US-2: As the operator, I can color-tag panes to tell agents apart
- [ ] New `tab-action` actions: `set_flag_color <color>` and `clear_flag_color`
      (CLI: `limux tab-action --action set_flag_color --color <named>`).
      (Codex-required — DEPENDENCY CALLOUT) `tab.action` does NOT route on
      the live GTK bridge today: `control_bridge.rs` `METHODS` (lines 20-39)
      has no `tab.action` and the bridge never forwards to core — it only
      exists in the standalone dispatcher (`limux-core` lib.rs:6126). This
      PRD therefore includes: a new bridge route + `ControlCommand` variant +
      window.rs metadata application for `tab.action` (ALL existing actions:
      rename/clear_name/pin/unpin/mark_unread/mark_read, plus the two new
      ones) — this is the bulk of US-2's work, not an add-two-verbs edit.
      If PRD-E's registry lands first, classify there instead of ad-hoc.
- [ ] (Codex-revised — kills the CSS-injection surface) v1 accepts the fixed
      NAMED palette only (≥6 colors); arbitrary `#rrggbb` is deferred.
      Validation is an allowlist match BEFORE any string reaches CSS; invalid
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
- [ ] Flag color is carried additively in the live bridge's `pane.surfaces`
      response (per-surface `flag_color` field) — `sidebar-state` is a
      CLI-side aggregation over `workspace.list` (main.rs:4660) and holds no
      per-tab data, so `pane.surfaces` is the named carrier; `sidebar-state`
      folds it in from there. No existing fields renamed.

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
older builds load it safely (`TabState` uses serde defaults, no
`deny_unknown_fields`, layout_state.rs:159) but an older build that
loads-then-saves silently DROPS `flag_color` (accepted). If the overlay
approach causes regressions on WSLg, fall back to the current inset
box-shadow rendering (defect severity returns to status quo, not worse).

## Open Questions

1. Named palette choice — propose: red, orange, yellow, green, blue, purple
   (GNOME accent-adjacent). Operator may adjust at review.
2. Should flag color also tint the workspace sidebar row when any flagged pane
   exists in it? (Default: no — keep sidebar semantics untouched in v1.)
