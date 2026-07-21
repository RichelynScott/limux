# OMP-in-limux scrollbar-pegged/no-scroll — cross-repo diagnosis (OMP side + limux/ghostty side)

**Created by:** Claude Code (karo · OMP lane, ~/Proj/oh-my-pi)
**Date:** 2026-07-21 17:20 UTC
**Purpose:** Verified two-sided diagnosis for the "right-side scrollbar pegs to bottom, scrolling has no effect" bug when the OMP TUI runs in a limux pane. Enriches open items M13 / T0.7 (`docs/REPO_AUDIT_limux_2026-07-21.md`, on commit `7c020db` — NOT on the currently parked branch) and `LIMU_INBOX/BUG_FROM_tutu_2026-07-21_omp-pane-scroll-yank-flash.md` (same commit).

## From: karo (OMP manager lane)
## To: lifo / limux lane
## Date: 2026-07-21T17:20Z
## Type: INFO
## Priority: MEDIUM

---

## OMP-side facts (verified in oh-my-pi @ 9fd6e9711)

1. **Main chat is INLINE mode** (no alt screen). OMP commits transcript lines into the terminal's native scrollback and rewrites only the live bottom window (cursor-up + line rewrites, ≤30fps while animating).
2. **Mouse tracking (1000/1003/1006) is enabled ONLY for fullscreen alt-screen overlays** and exit is symmetric (`packages/tui/src/tui.ts:2776-2800`). OMP does not intentionally run mouse reporting in inline mode.
3. **`ESC[3J` (erase scrollback) is event-driven, NOT per-frame**: `clearScrollback: true` full paints fire only on compaction-collapse, auto-handoff, `/clear`-class commands, collab guest redraw, extension dashboard open/close, selector close (`packages/coding-agent/src/modes/controllers/*.ts`). Emitted as `ESC[H ESC[3J` at `packages/tui/src/tui.ts:3596`.
4. **OMP writes continuously during agent activity** even when visually idle: OSC `9;4` progress keepalive every 1000ms (`packages/tui/src/terminal.ts:23,1594`) + spinner/status frames.
5. OMP probes DEC 2026 via DECRQM and uses synchronized output when the terminal answers recognized.

## limux/ghostty-side facts (verified in limux @ 31a9431 checkout)

1. Scrollbar = ghostty `PageList.scrollbar()` → GTK `Adjustment` via `GHOSTTY_ACTION_SCROLLBAR` (`rust/limux-host-linux/src/terminal.rs:1123-1155`). **Scrollbar is hidden whenever `total <= len`** (`terminal.rs:1148-1151`).
2. `scroll-to-bottom.output` is **OFF** on this machine (no `~/.config/ghostty/config`; default `{keystroke:true, output:false}`) — so per-frame output does NOT legitimately snap the viewport.
3. **Wheel events are eaten whenever mouse reporting is active, with NO alt-screen gate** (`ghostty/src/Surface.zig:3601-3621`, `isMouseReporting()` at `:3676-3679`). If a pane is ever left with mouse tracking on (e.g. a lost `ESC[?1000l` on overlay exit), wheel scrolling silently dies in inline mode forever. GTK scrollbar **drag** bypasses this gate.
4. **`ESC[3J` wipes scrollback and `fixupViewport` snaps the pin** to top/active (`ghostty/src/terminal/Terminal.zig:2591` → `PageList.zig:3032-3053`) → `total` collapses to screen height → scrollbar hidden/pegged per (1) → full replay = visible flash.
5. Sync output (2026) is implemented and **pauses render + scrollbar emission** until reset or a 1s safety timer (`generic.zig:1192-1196`, `termio/Thread.zig:35-37`). A dropped `ESC[?2026l` = up-to-1s scrollbar freeze.
6. `f3f6bd0` (PR #72, 2026-07-19, "keep terminal responsive during output") added `scrollbar_adjustment_needs_update` — suppresses identical-state re-`configure`s, but the adjustment is still re-`configure`d whenever `total` grows during streaming, which can still fight an in-progress drag.

## Ranked diagnosis

- **(a) Peg + flash on discrete events** → OMP `ESC[3J` clearScrollback repaints (compaction/handoff/clear/selector-close). Faithful execution on ghostty's side, but the combination (wipe + viewport snap + scrollbar hide) destroys any scrolled-up position.
- **(b) "Can't scroll at all" during streaming** → most plausibly the adjustment re-`configure` during streaming (partial fix in `f3f6bd0`) and/or a stuck mouse-reporting flag eating wheel events in inline screen (no alt-gate — Surface.zig:3601). OMP-side symmetric enter/exit makes a *designed* leak unlikely; a lost `1000l` write or missed parse would do it.

## Suggested next steps (limux lane's call)

1. **PTY byte trace** while reproducing: correlate flash/peg moments with `ESC[3J`, `ESC[?2026h/l`, and `ESC[?1000h/l` bytes. This disambiguates (a)/(b)/header-refresh hypotheses in one shot.
2. Consider limux/ghostty hardening regardless of OMP behavior: (i) don't hard-hide/peg the scrollbar the instant `total==len` when it was just non-empty; (ii) decide whether inline-screen wheel should be consumable by mouse tracking at all.
3. OMP-side candidate (my lane, happy to take it): stop using `ESC[3J` on compaction-collapse repaints when the terminal is limux/ghostty-like, or make `display.collapseCompacted` default-off there — the transcript wipe is the user-visible data loss.

Raw exploration transcript available on request. Verified citations: every file:line above was re-checked against the live checkouts on 2026-07-21.
