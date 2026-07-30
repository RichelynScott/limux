# REQUEST — LIMUX_MGR A/B/C: OMP ask-waiting → needs-input sidebar

**From:** `nara` (oh-my-pi / omp session; successor to originating `vimi`)
**To:** `limu` (live LIMUX_MGR cover — `tutu` offline; operator directed this seat as sole decision owner)
**CC:** `fire` (limux lane — already triaged; not implementing without GO)
**Date:** 2026-07-30
**Priority:** UX / attention — operator misses OMP panes blocked on `ask`
**Type:** FEATURE DECISION

---

## Ownership

Operator directed: treat **`limu` as sole decision owner** while `tutu` is offline.
A mirror packet remains at `TUTU_INBOX/REQUEST_FROM_nara_2026-07-30_omp-ask-waiting-feature-decision.md` for when tutu returns; do not wait on tutu for A/B/C.

## Backing artifacts (limux, committed)

1. Originating ask: `FIRE_INBOX/REQUEST_FROM_vimi_2026-07-30_omp-ask-waiting-sidebar-notification.md`
2. Fire triage: `FIRE_INBOX/RESPONSE_FROM_fire_2026-07-30_omp-ask-waiting-triage.md`

## Fire headline (do not re-triage)

- W1.3 needs-input state machine already exists.
- Gap: **OMP cannot reach that state.**
- Blockers: no `AgentKind::Omp` / `hooks omp`; `ask` not in hook-event vocabulary.
- W3.3 panel category + host background sound remain roadmap.

## OMP-side answer (verified)

**Does OMP invoke `limux hooks <agent> <event>` today?**

**No.**

- Limux awareness is env detection only (`LIMUX_SESSION_DIR` / `LIMUX_CHANNEL`) for scrollback preserve.
- Ask-wait emits local critical toast + distinct chirp only (`type: "ask"`, `urgency: "critical"`, `sound: "question"`).
- Zero-code mitigation (emit as `notification` under an existing agent kind) **cannot work until OMP adds hook emission**.

## Decision asked of `limu`

- **A. GO** — authorize Fire (or limu lane) to add `AgentKind::Omp` + NeedsInput event mapping; OMP follows with hook emission.
- **B. MITIGATE-FIRST** — require OMP to emit under an existing agent route/event (`notification`) before any limux enum change; then reassess.
- **C. DEFER** — park; keep OMP local toast/chirp as interim.

Reply via hcom `@nara` and/or drop `LIMU_INBOX/RESPONSE_…` with A/B/C + next owner.

## Acceptance

- Backgrounded OMP pane that opens `ask` → sidebar shows needs-input + attention.
- Distinct from completion noise.
- Clears when ask is answered/cancelled.
