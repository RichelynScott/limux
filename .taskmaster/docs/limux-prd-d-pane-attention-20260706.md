# PRD-D: Pane Attention Border Layering + Per-Pane Color Flags

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:20 UTC
**Purpose:** Fix the one live operator-visible UI defect promoted into Wave 0 —
the pane attention border renders behind pane content (mostly invisible on
right-hand panes) — and add deterministic per-pane color flags, per TaskMaster
task #20 and the existing plan note.

- **Priority:** P0 (Wave 0 — roadmap W0.5; TaskMaster #20)
- **Dependencies:** none. Verified live via the PRD-C checklist once both land.
- **Effort:** M (Codex-revised from S: includes the pane.action bridge route +
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
- [ ] New pane-scoped action route: `pane.action` with
      `set_flag_color <color>` and `clear_flag_color` (CLI:
      `limux pane-action --pane <id|ref> --action set_flag_color --color <named>`;
      inside a Limux pane, `LIMUX_PANE_ID` may be the default target).
      (Codex-required — DEPENDENCY CALLOUT) no live GTK bridge route exists
      for pane metadata actions today; this PRD therefore includes a new
      `pane.action` core method, live bridge route, `ControlCommand` variant,
      and window.rs metadata application. Do NOT piggyback on `tab.action` for
      the color flag: a GTK pane owns multiple tabs, and the flag is a
      per-pane operator marker. If PRD-E's registry lands first, classify the
      new `pane.action` mutation there instead of ad-hoc.
- [ ] (Codex-revised — kills the CSS-injection surface) v1 accepts the fixed
      NAMED palette only (≥6 colors); arbitrary `#rrggbb` is deferred.
      Validation is an allowlist match BEFORE any string reaches CSS; invalid
      color → clean error, no partial state.
- [ ] Flag color renders as a compact, always-visible indicator on the pane
      chrome/tab-strip area AND a subtle pane-edge accent, visually distinct
      from both the attention border and unread styling. The indicator follows
      the pane across active-tab switches; it is not stored on individual tabs.
- [ ] Flag color persists in `session.json` (layout_state) and restores on
      relaunch — restore test added beside the existing session-restore tests.
- [ ] GUI affordance: context-menu entry on the pane chrome/content area
      (submenu of palette colors + clear). If the visible affordance is placed
      in the tab strip for convenience, the command still targets the owning
      pane id, not the clicked tab id.

### US-3: As a developer, semantics stay clean
- [ ] Attention (transient) and flag color (durable) are separate state
      fields — setting/clearing one never mutates the other (unit test).
- [ ] Unread dot/styling unchanged when a flag color is set (regression test
      on the sidebar/tab unread path).
- [ ] Flag color is carried additively in live bridge pane metadata:
      `pane.list` rows gain a pane-scoped `flag_color`, and `pane.surfaces`
      may mirror it as `pane_flag_color` for each returned row only as
      redundant pane context. There is no per-surface/per-tab `flag_color`
      field. `sidebar-state` folds pane metadata in from `pane.list` or the
      bridge snapshot path. No existing fields renamed.

## Functional Requirements

1. Rendering fix in `rust/limux-host-linux/src/pane.rs` (+ `window.rs` wiring
   if the overlay must be owned by the split container in
   `split_tree.rs`).
2. State: extend the pane model + `layout_state.rs` serialization with
   `flag_color: Option<String>`; additive `session.json` field (older builds
   ignore it — confirm serde tolerates unknown fields both directions).
3. Protocol: add `pane.action` in `rust/limux-core/src/lib.rs` AND the GTK
   bridge path so both the standalone dispatcher and live GUI accept the new
   actions; CLI gains `pane-action --color`. `tab.action` remains out of scope
   except for any existing behavior PRD-E later classifies.
4. CSS: new classes (e.g. `.limux-pane-flag-<name>`), colors defined once.

## Non-Goals

- No per-WORKSPACE color flags (sidebar workspace coloring exists via
  highlight color; this PRD is pane-scoped).
- No auto-assignment of colors by agent type (manual only in v1).
- No attention-behavior redesign (sound, blink, escalation — unchanged).
- No settings-editor UI for the palette (fixed palette v1).

## Technical Considerations

- ID plumbing: target by pane id/ref (`pane_id`, `pane_ref`, or
  `LIMUX_PANE_ID`). A surface id may be accepted only as a convenience for
  resolving its owning pane; the stored state remains pane-scoped. No new ID
  scheme.
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
cargo test -p limux-core pane_action -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended with flag-color + attention-overlay assertions
```

## Rollback Plan

`git revert` the feature commits. `session.json` field is additive/optional —
older builds load it safely (`PaneState` uses serde defaults, no
`deny_unknown_fields`, layout_state.rs:149) but an older build that
loads-then-saves silently DROPS `flag_color` (accepted). If the overlay
approach causes regressions on WSLg, fall back to the current inset
box-shadow rendering (defect severity returns to status quo, not worse).

## Open Questions

1. Named palette choice — propose: red, orange, yellow, green, blue, purple
   (GNOME accent-adjacent). Operator may adjust at review.
2. Should flag color also tint the workspace sidebar row when any flagged pane
   exists in it? (Default: no — keep sidebar semantics untouched in v1.)
