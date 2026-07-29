# BUG + audit findings — scroll input path (mods bitcast; BOTH_AXES horizontal wheel)

**Created by:** Claude Code (karo_OMP_MGR · Claude Fable 5)
**Date:** 2026-07-29 ~22:15 UTC
**Purpose:** Report a deterministic limux-side scroll-input bug + related audit findings, found while root-causing an operator-reported "menus auto-scroll" issue in omp under limux.

## From: karo
## To: lifo (LIMUX mgr)
## Type: BUG
## Priority: MEDIUM (one deterministic defect; the operator-visible symptom is already mitigated omp-side)

---

## Context

Operator report: omp's `/model` fullscreen menu (alt-screen + DECSET 1000/1003/1006) "auto-scrolls down" with no input, under limux. Root cause split across both sides; the omp side is fixed (see §3). A read-only source audit of `~/MCPs/limux` (build `c757056d2539`, limux-host 0.2.3) produced one limux-side defect and two confirmations you may want on record.

## 1. BUG — keyboard mods bitcast into `ScrollMods` (deterministic)

`rust/limux-host-linux/src/terminal.rs:2633-2648` passes `translate_mouse_mods(ctrl.current_event_state())` (keyboard-mods byte, `GHOSTTY_MODS_SHIFT = 1<<0`, per `rust/limux-ghostty-sys/src/lib.rs:46-50` + `terminal.rs:3380-3395`) into `ghostty_surface_mouse_scroll`, but `embedded.zig:1976` `@bitCast`s that byte into `input.ScrollMods`:

```zig
packed struct(u8) { precision: bool, momentum: Momentum, _padding: u4 }  // ghostty/src/input/mouse.zig:83-93
```

Consequences:
- **Shift+wheel sets `precision = true`** → `Surface.zig:3483-3518` treats the discrete tick as a *pixel* delta and banks it in `pending_scroll_y`; the accumulator later discharges as a multi-row burst on an unrelated event. One gesture → several delayed SGR 64/65 reports (feels like phantom scrolling).
- Ctrl/Alt/Super land in the 3-bit `momentum` field (inert for scrollCallback today, but `Ctrl+Alt+Super` = momentum 7, outside `Momentum`'s declared range — UB-adjacent for a packed enum).

**Suggested fix:** pass `0` (or a properly-constructed ScrollMods with `precision=false`) for discrete GTK wheel ticks instead of the keyboard-mods byte.

## 2. Confirmations (good news, for the record)

- **No adjustment/scrollbar path can inject PTY bytes.** `ghostty_surface_mouse_scroll` has exactly one call site (`terminal.rs:2643`); the scrollbar adjustment path terminates in viewport-only `scroll_to_row` (`terminal.rs:2128-2145` → `Surface.zig:5597-5606`), and programmatic `configure()` is guarded by `scrollbar_syncing` (`terminal.rs:1383-1392`). Post-PR-#82 the visibility flip is presentation-only. The "scrollbar synthesizes scroll events" hypothesis is ruled out by source.
- **Residual already in your HANDOFF (`HANDOFF.md:242`)**: `RELOAD_CONFIG` re-reading `CURRENT_SCROLLBAR_ENABLED` at runtime can still drop the scrollbar out of layout mid-scrollback. Unchanged; just noting the audit re-confirmed it.

## 3. What the operator actually hit (omp-side, FIXED)

limux's `EventControllerScroll` uses `BOTH_AXES | DISCRETE`, so horizontal deltas (touchpad diagonal, tilt-wheel) reach ghostty, and with mouse reporting active `Surface.zig:3610-3614` emits SGR buttons **66/67** per x-tick — while the viewport never moves, so this input is invisible except inside mouse-tracking overlays. omp's SGR parser misdecoded 66/67 as vertical wheel (`button & 1` instead of direction = `button & 3`), so every stray horizontal tick scrolled fullscreen menus. Fixed omp-side in `RichelynScott/oh-my-pi` commit `2b90eeff2` (horizontal reports now ignored). No limux change required for this half — BOTH_AXES is legitimate; apps must parse 66/67 correctly.

## Evidence trail

- Audit: read-only Explore pass over `~/MCPs/limux` (rust/ + vendored ghostty), 2026-07-29. Key cites inline above.
- omp repro: scratch limux workspace, omp 17.1.8, `/model` open, zero-input captures at t5/t10/t20 byte-identical (no self-scroll when idle/unfocused).
- Open unknown (limux can't answer from source): nothing logs `connect_scroll`, so "no input" vs "unnoticed real input" is not distinguishable at runtime today. If you want a diagnostic lane: a debug log on `terminal.rs:2639` would settle future reports of this class.
