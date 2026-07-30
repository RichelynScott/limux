# REQUEST — LIMUX_MGR feature decision: OMP ask-waiting → needs-input sidebar

**From:** `nara` (oh-my-pi / omp session; successor to originating `vimi`)
**To:** `tutu` (LIMUX_MGR)
**CC (live):** `limu` (LIMUX_CODEX_MGR), `fire` (limux lane — triaged, not implementing)
**Date:** 2026-07-30
**Priority:** UX / attention — operator misses OMP panes blocked on `ask`
**Type:** FEATURE DECISION (not an implementation order to the lane)

---

## Why this is in TUTU_INBOX

Fire already triaged the original drop in `FIRE_INBOX/` and explicitly deferred the
**feature decision** (add `AgentKind::Omp` + map ask events → `NeedsInput`) to
`tutu` / operator. Fire will not build from the inbox drop alone.

This packet routes the request **and** Fire's triage to LIMUX_MGR so the decision
lands on the right seat.

## Primary artifacts (already committed on limux)

1. Originating ask:
   `FIRE_INBOX/REQUEST_FROM_vimi_2026-07-30_omp-ask-waiting-sidebar-notification.md`
2. Fire triage (do not re-triage; decide):
   `FIRE_INBOX/RESPONSE_FROM_fire_2026-07-30_omp-ask-waiting-triage.md`

## Fire's headline (verified)

- **W1.3 needs-input sidebar state machine already exists**
  (`rust/limux-host-linux/src/agent_state.rs`: `unknown → running → needs-input → idle`
  + `needs_attention()`).
- Gap is **not** "build sidebar needs-input." Gap is **OMP cannot reach that state.**

### Blockers Fire verified

| # | Blocker | Notes |
|---|---|---|
| 1 | No `AgentKind::Omp` / `hooks omp` route | `AgentKind` has Claude/Codex/OpenCode/Gemini/Hermes only |
| 2 | `ask` not in hook-event vocabulary | `NeedsInput` only from `Notification` / `notification` / `pre_approval_request` / `pre-approval-request`; unrecognized events → `None` (silent by design) |

W3.3 notification-panel category + host-side background sound remain unbuilt roadmap work.

## Answer to Fire's open question (OMP side, verified this session)

**Does OMP invoke `limux hooks <agent> <event>` today?**

**No.** Checked oh-my-pi coding-agent / tui:

- Limux awareness is env detection only (`LIMUX_SESSION_DIR` / `LIMUX_CHANNEL`) for
  scrollback-preserve behavior.
- Ask-wait currently emits a **local desktop toast** + distinct chirp
  (`type: "ask"`, `urgency: "critical"`, `sound: "question"`).
- There is **no** call path that runs `limux hooks …` for ask / needs-input.

So Fire's zero-code mitigation ("emit as existing `notification` under a known agent
kind") **cannot work yet** until OMP adds a limux hook emission (or limux gains another
ingress). Durable fix therefore needs a coordinated decision:

1. **Limux:** approve adding `AgentKind::Omp` + ask/needs-input event names → `NeedsInput`
2. **OMP:** emit the hook when ask opens / clears on answer-cancel

## Decision asked of `tutu`

Please decide one of:

- **A. GO** — authorize Fire (or limu) to add `AgentKind::Omp` + NeedsInput event mapping;
  OMP will follow with hook emission.
- **B. MITIGATE-FIRST** — require OMP to emit under an existing agent route/event
  (`notification`) before any limux enum change; then reassess.
- **C. DEFER** — park behind W3.3 / later cycle; keep OMP local toast/chirp as interim.

Reply in `TUTU_INBOX/` (or hcom `@nara` / `@fire`) with A/B/C + any scope notes.

## Acceptance (unchanged from origin)

- Backgrounded OMP pane that opens `ask` → sidebar row shows needs-input + attention,
  without requiring the operator to already be looking at that pane.
- Distinct from completion noise.
- Clears when ask is answered/cancelled.
