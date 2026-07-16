# TASK — Ctrl+C acts as clear/interrupt instead of copy in Claude Code sessions (operator-routed)

**Created by:** Claude Code (zen_master / ZEN_MASTER_PAL_MCP_MGR · 46ec0955)
**Date:** 2026-07-16 ~17:05 UTC
**Purpose:** Operator explicitly routed this investigation+fix to the limux manager lane ("Send the ctrl+c investigation and fix to proper lane which is limux mgr").

## From: zen_master
## To: lifo (LIMUX_MGR)
## Date: 2026-07-16
## Type: TASK
## Priority: HIGH (operator-reported UX defect, verbatim complaint below)

---

## Operator report (verbatim)

> "WTF, fix cntrl+c for claude sessions, this is copy, but you are treating it like clear"

Context: the operator selects text in a Claude Code session (limux-managed pane/terminal) and presses Ctrl+C expecting COPY; instead the keystroke reaches the Claude Code TUI, which treats Ctrl+C as interrupt/clear-input — the selection is not copied and the input line is cleared.

## Observations from my side (unverified leads, your lane to confirm)

1. Claude Code's TUI consumes Ctrl+C as interrupt/clear by design; copy must be handled at the TERMINAL layer before the keystroke reaches the PTY.
2. Windows Terminal's default behavior (Ctrl+C = copy WHEN a selection exists, SIGINT otherwise) is the UX the operator expects. Whatever terminal/pane stack limux drives for Claude sessions apparently forwards Ctrl+C to the PTY unconditionally.
3. Candidate fix directions: (a) terminal/pane keybinding: bind Ctrl+C to copy-when-selection-exists, pass-through otherwise; (b) enable copy-on-selection so Ctrl+C intent is moot; (c) if limux's renderer/pane layer intercepts keys, add the selection-aware branch there.
4. Related prior art: `~/.claude/skills/fix-cursor-input` (Cursor+WSL2 keybinding layers) and `~/.claude/keybindings.json` support in Claude Code (`keybindings-help`) — but rebinding Claude Code's interrupt away from Ctrl+C is probably WRONG (Ctrl+C-as-cancel is load-bearing); the selection-aware terminal-layer fix preserves both.

## Ask

Investigate + fix in the limux-managed terminal stack so Ctrl+C copies when a selection exists in Claude Code sessions (and still interrupts when no selection). Operator-visible defect; please report disposition to the operator and/or zen_master when landed.

Delivery: this INBOX drop + hcom doorbell. My hcom `send` is broken (defect already filed with dino); reach me via hcom normally (inbound works) or term-inject.
