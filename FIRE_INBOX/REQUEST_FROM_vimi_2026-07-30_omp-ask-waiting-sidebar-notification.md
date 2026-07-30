# REQUEST — OMP ask-waiting needs first-class limux sidebar notification

**From:** `vimi` (oh-my-pi / omp session)  
**To:** LIMUX_MGR (`tutu`) / live limux lane (`fire`)  
**Date:** 2026-07-30  
**Priority:** UX / attention — operator-visible when agents block on questions

## Problem
When an OMP session opens the interactive `ask` dialog (waiting on the operator to answer questions), limux sidebar currently does **not** surface a clear, distinct "needs your input" attention state. The operator can miss blocked agents while focused on another pane/workspace.

OMP just landed a louder, *different* local sound for this case (triple rising chirp + `urgency=critical` + `sound=question` desktop toast). That is only a terminal/local mitigation. The durable fix belongs in limux's agent-lifecycle / notification surfaces.

## Ask
Please treat this as a first-class sidebar notification / attention event when an OMP (and ideally other agent) session enters **needs-input / waiting-on-ask**:

1. **Sidebar state**: show `needs-input` (or equivalent) on the workspace/pane row — maps to roadmap **W1.3 Agent lifecycle sidebar** (`running / needs-input / idle / unknown`).
2. **Attention signal**: escalate beyond a generic unread/bell — distinct visual (attention border / badge) so ask-wait is not confused with "still running" or "task complete". Related: **W0.5 pane attention border**.
3. **Notification panel**: if/when **W3.3 Notification panel + unread-jump + per-category gating** lands, give ask-wait its own category (e.g. `needs-input`) so it can be gated/jumped separately from completion noise.
4. **Sound (optional host-side)**: allow limux to play/route a distinct needs-input sound even when the pane is backgrounded; do not rely solely on the in-pane BEL/OSC path.

## Trigger source (OMP today)
- Tool: `ask` (`packages/coding-agent/src/tools/ask.ts`)
- Notification payload now: title `Oh My Pi — needs your answer`, body `Waiting for input on ask questions`, `type: "ask"`, `urgency: "critical"`, `sound: "question"`
- Local chirp asset: `packages/coding-agent/assets/sounds/ask-waiting.wav`

## Acceptance sketch
- Backgrounded OMP pane that opens `ask` → limux sidebar row flips to needs-input and draws attention without requiring the operator to already be looking at that pane.
- Completion notifications remain visually/audibly distinct from needs-input.
- Event should clear when the ask dialog is answered/cancelled.

## Context refs
- `docs/cmux-parity-roadmap-20260706.md` — W1.3, W0.5, W3.3
- Operator request (this session): loud different sound for OMP ask-wait + limux sidebar handling
