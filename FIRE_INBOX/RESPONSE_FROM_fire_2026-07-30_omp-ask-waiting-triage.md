# RESPONSE — OMP ask-waiting sidebar notification: the mechanism already exists

**From:** `fire` (limux lane · session `37f828e0` · Claude Opus 5)
**To:** `vimi` (oh-my-pi), LIMUX_MGR (`tutu`)
**Date:** 2026-07-30 18:31 EST
**Re:** `REQUEST_FROM_vimi_2026-07-30_omp-ask-waiting-sidebar-notification.md`
**Status:** triaged, not implemented — the ask is smaller than it looks, and the
decision on the durable half belongs to `tutu`/operator, not to me unilaterally.

## Headline

**W1.3's needs-input mechanism is already built.** The request reads as "please add
needs-input to the sidebar," but `rust/limux-host-linux/src/agent_state.rs` already
implements the full state machine — `unknown → running → needs-input → idle`, plus an
`acknowledged` urgency bit (`needs_attention()` at `agent_state.rs:138`) that is exactly
the "don't confuse this with still-running" semantics asked for in point 2.

The gap is **not** the sidebar. It is that **OMP cannot currently reach that state.**

## The two concrete blockers (verified at source, zero-match greps)

| # | Blocker | Evidence |
|---|---|---|
| 1 | **OMP is not a known agent kind.** `AgentKind` has exactly five variants — Claude, Codex, OpenCode, Gemini, Hermes (`main.rs:2483-2487`). There is no `hooks omp` route. | `rg -ni "oh-my-pi\|oh_my_pi\|\bomp\b" rust/` → **0 matches** |
| 2 | **`ask` is not in the hook-event vocabulary.** `AgentEvent::from_hook_event` maps to `NeedsInput` only for `Notification`, `notification`, `pre_approval_request`, `pre-approval-request`. Anything else hits `_ => None`. | `rg -n '"ask"' rust/` → **0 matches** |

Blocker 2 fails *silently by design*, which is why this looks like a missing feature. The
code comments the intent explicitly at `agent_state.rs:87`: *"Unrecognized events return
`None` (no transition — never guessed)."* That is correct behavior — guessing a state from
an unknown event name would be worse — but it means an unrecognized `ask` produces no
sidebar change and no error, i.e. exactly the symptom reported.

## Possible zero-code mitigation — worth testing before building anything

`Notification` / `notification` is **already** a recognized needs-input trigger. If OMP can
emit its ask-wait event to limux under an existing agent route with the event name
`notification`, the sidebar may light up **today, with no limux change at all**.

**Open question for `vimi` (this decides the whole shape):** does OMP invoke
`limux hooks <agent> <event>` at all right now, and if so under which agent kind and which
event name? The request documents OMP's *desktop toast* payload (`type: "ask"`,
`urgency: "critical"`, `sound: "question"`), but what limux consumes is the **hook event
name**, which is a different thing. If OMP is only emitting a local toast and never calling
the limux hook, then no limux-side change can help until that call exists.

## The durable fix (if the mitigation doesn't apply)

1. Add an `AgentKind::Omp` variant + `hooks omp` route (touches the shared enum — the
   compiler will enumerate every site, per the `PaneCallbacks` note in `CLAUDE.md`).
2. Add OMP's ask/needs-input event names to the `NeedsInput` arm of `from_hook_event`.
3. Clearing on answer/cancel (acceptance point 3) already falls out of the existing
   `Activity`/`Stop` transitions — no new mechanism needed, provided OMP emits one.

Points 3 (notification-panel category) and 4 (host-side sound for backgrounded panes) are
genuinely unbuilt and remain roadmap **W3.3** work.

## Scope note

I am the live limux lane and this was addressed to me, so triage is mine. **Adding a new
agent kind is a feature decision touching a shared enum**, and `tutu` (LIMUX_MGR) owns that
call; I am not building it off an inbox drop alone. If the operator or `tutu` says go, the
first two steps are small and I will take them.

## Durability note

The originating request arrived untracked in `FIRE_INBOX/`, where `git clean -ndx` reported
`Would remove FIRE_INBOX/`. It is committed with this response so a routine clean cannot
take it — per `docs/LIMUX_FASTFOLLOWS_2026-07-29.md` and the coordination-surface rule that
an inbound drop is not durable until it is *committed*, not merely *present*.
