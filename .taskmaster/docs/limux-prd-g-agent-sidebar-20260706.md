# PRD-G: Agent Lifecycle Sidebar — Per-Workspace Agent States + Sidebar Scalability

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:50 UTC
**Purpose:** Make 10–20 parallel agents legible at a glance: sidebar rows show
each workspace's agent lifecycle state (running / needs-input / idle / unknown)
fed by the existing hook pipeline, and sidebar updates scale without rebuilding
list models on every title/output tick. Research db cmux-20260702-002 +
cmux-20260702-007 (both high; "merge into agent sidebar PRD" was the db's own
next-action).

- **Priority:** P1 (Wave 1 — roadmap W1.3). Independent of PRD-E/F — parallelizable.
- **Dependencies:** none hard; PRD-D's flag-color rendering should land first
  to settle sidebar row layout once (soft ordering).
- **Effort:** M
- **Channel targeting:** preview until PRD-C checklist passes

## Problem Statement

cmux's core value proposition is the metadata-rich sidebar that tells you which
of 20 agents needs you (cmux PR #6480/#7005 lifecycle states; README sidebar
identity). Limux's sidebar today shows name, favorite star, notification dot +
last-message label, unread state, highlight color, and path
(`rust/limux-host-linux/src/window.rs` sidebar section) — but no notion of
*agent state*: a workspace whose agent has been idle for an hour looks
identical to one burning tokens or one blocked on a question. The raw signal
already exists: `limux hooks` installs Claude/Codex/Gemini/OpenCode hook
configs and `hermes-hook` accepts lifecycle events, all currently collapsing
into `notification.create`. Separately, research db cmux-007 flags the
scalability trap cmux hit (PR #6807/#7182): rebuilding sidebar list models per
title/output update melts under many active workspaces.

## Goals

1. A per-workspace (and per-pane, internally) agent lifecycle state machine
   fed by hook events, rendered as a compact state indicator on sidebar rows.
2. Sidebar update path does targeted row updates — measured, not vibes.
3. State is queryable over the control socket for agents/scripts.

## State model (binding)

`unknown → running → needs-input → idle` transitions:

| Event source | Transition |
|---|---|
| Hook event: agent started / prompt submitted / tool activity (per-agent vocabularies parsed by the shims — note (Codex-revised): there is NO `codex-hook` alias; Codex routes via `limux hooks codex <event>`, dispatch main.rs:5855) | → `running` |
| Hook event: awaiting user input / permission request / notification-class "needs attention" | → `needs-input` |
| Hook event: stop/end-of-turn | → `idle` |
| No hook data for the surface (non-agent pane, hooks not installed) | `unknown` (never guessed) |
| Workspace focused + operator interacts (existing hover/focus-clear analog) | `needs-input` → `running` or `idle` per next hook event; the *visual urgency* clears like unread does today |

State is per-surface, aggregated to workspace as: any `needs-input` →
needs-input; else any `running` → running; else any `idle` → idle; else
unknown. Unread semantics are untouched — state is a separate field, not a
re-skin of unread. Closing a pane/tab/surface REMOVES its state from the
machine (Codex-required — otherwise a closed surface stuck in `needs-input`
pins the workspace aggregate forever; eviction test required). Operator
interaction does NOT mutate state (state changes only on hook events); the
visual urgency uses a separate `acknowledged` bit that clears on focus,
mirroring unread — so socket-reported state and sidebar never disagree.

**(Codex-required) Per-family reachable-state matrix — the installed hook
wiring, not the parser, determines what is reachable, and today's installers
break the model for 4 of 5 families:**

| Family | Installed events today | Reachable states | In-scope fix (FR-2) |
|---|---|---|---|
| hermes | `pre_llm_call` / `pre_approval_request` / `post_llm_call` (main.rs:1170-1187) | ALL | none — reference family |
| claude | `Notification` hook is installed mapped to positional event `stop` (`install_hook_target` main.rs:1709), and the positional event WINS over JSON `hook_event_name` (`parse_hook_event` main.rs:1012-1017) → needs-input arrives as `stop` = INVERTED to idle | broken as-is | remap installer to `("Notification", "notification")` (side effect is a bug fix: "Claude finished" toast on Notifications becomes "Claude needs you"); delivered via `limux hooks setup` re-run |
| codex | SessionStart/UserPromptSubmit/Stop only (main.rs:1693-1701) — no needs-input source | running/idle/unknown | wire the Codex notification-class event if its hook vocabulary offers one; else document needs-input as unreachable for codex v1 |
| gemini | SessionStart/BeforeAgent/AfterAgent/SessionEnd (main.rs:1714-1723) | running/idle/unknown | same treatment as codex |
| opencode | plugin maps `session.idle/updated/status/compacted` → `prompt-submit`, NO stop-class event (main.rs:2210-2213) → idle unreachable, idle agents look `running` | broken as-is | rework plugin mapping: `session.idle` → stop-class |

US-1's "works for every hooked agent family" acceptance is scoped BY THIS
MATRIX: full three-state coverage is required only where the family's hook
vocabulary provides the events; unreachable states per family are documented
in the checklist doc, never silently wrong.

**Running-state keepalive:** no family installs tool-activity hooks today
(only the parser recognizes them, main.rs:1183-1185). FR-2 wires
tool-activity (PreToolUse/PostToolUse-class) events for families that support
them (claude does) as the `running` keepalive; for families without, the
decay default is raised to 30 min and the mid-turn decay behavior is
documented.

## User Stories

### US-1: As the operator, I can see who needs me across 15 workspaces
- [ ] Sidebar rows render a state indicator (icon or colored glyph +
      accessible tooltip naming the state) for the aggregated workspace state.
- [ ] `needs-input` is visually dominant (distinct from unread dot AND from
      PRD-D attention border). The distinctness matrix's axes are named
      (Codex-revised): {needs-input indicator on|off} × {unread dot on|off} ×
      {attention border on|off} = 8 combinations, asserted structurally
      (widget/CSS-class presence) in the Xvfb suite; PLUS one rendering
      assertion each for the `running`, `idle`, and `unknown` indicator
      variants.
- [ ] State transitions arrive without focus changes (background workspace
      row updates when its agent stops — Xvfb fake-agent test).
- [ ] Stale-state decay: `running` with no hook events for a configurable
      period degrades to `unknown` (never silently forever-running).
      Default 30 min (Codex-revised — long agent turns emit no events for
      families without tool-activity hooks; see keepalive note above).
- [ ] Works for every hooked agent family: claude, codex, gemini, opencode,
      hermes (fixture hook payloads per family in tests — reuse the payload
      shapes the hook shims already parse).

### US-2: As an agent/script, I can query fleet state over the socket
- [ ] `workspace.list` and `pane.surfaces` responses gain additive
      `agent_state` fields (per PRD-E registry conventions if landed; plain
      additive fields if PRD-G ships first).
- [ ] `sidebar-state --workspace <id>` includes per-surface + aggregated
      state.
- [ ] New CLI convenience `limux agents-status [--json]`: one table — workspace,
      pane/surface, agent family (from hook metadata when known), state, age
      of last event.
- [ ] Hook shims stamp events with surface identity exactly as today
      (`LIMUX_SURFACE_ID` env contract) — no new env vars required.

### US-3: As a maintainer, the sidebar scales
- [ ] (Codex-revised — ground truth: the sidebar ALREADY does targeted
      per-row widget updates via retained `gtk::ListBoxRow` + label/CSS
      mutations, window.rs:3506-3521/4810/6390; there is no per-update
      rebuild to fix) US-3 is therefore a REGRESSION GUARD-RAIL, not a fix:
      the new agent-state rendering must preserve the targeted-update
      pattern. Instrumentation counter (existing `debug.*` count/reset
      precedent, e.g. `debug.flash.count`) increments on row
      RE-CREATION and on full `sidebar_list` repopulation — those are the
      named increment sites; a test pushing 100 state updates across 20
      workspaces asserts counter == 0.
- [ ] Batching: bursts of hook events within one frame coalesce to one row
      update (existing GTK idle/tick pattern; test with 50-event burst).
- [ ] Wall-time is the PRIMARY metric: the harness records the 100-update
      scenario duration; merged PR description includes the numbers
      (target: no worse than 2× single-update cost).

## Functional Requirements

1. State machine in a pure module (`rust/limux-host-linux/src/agent_state.rs`
   or `limux-core` if the socket-visible fields land there) — unit-testable
   without GTK; GTK wiring separate (`docs/maintainability.md`). Data model
   records `AgentKind` per surface (feeds `agents-status` family column;
   precedent `AgentHookSessionRecord`, agent_hooks.rs:85) and evicts on
   surface/pane/tab close.
2. Hook path: extend the existing `hooks <agent> <event>` translation layer
   in `rust/limux-cli/src/main.rs` to ALSO emit a state-transition control
   call (new method `surface.agent_event` — additive, classified in the
   PRD-E registry as mutation). Call structure (Codex-revised, bound): the
   shim makes it a SECOND fire-and-forget call after the existing awaited
   `notification.create` (main.rs:1125-1131), connect+write timeout 2 s,
   failures silent — hook latency budget stays within the existing 5 s
   hook-side budget. Hook shims remain drop-in compatible EXCEPT the
   installer mapping fixes in the per-family matrix (Claude Notification
   remap, OpenCode plugin mapping, optional codex/gemini/tool-activity
   wiring) — all delivered via `limux hooks setup` re-run; no manual user
   config edits.
3. Sidebar rendering in `window.rs` sidebar row builder; row-update
   instrumentation counter behind `debug.*`.
4. Persistence: agent_state is runtime-only (NOT persisted to session.json —
   states are meaningless across restarts; restored workspaces start
   `unknown`).
5. All new socket fields additive; no renames.

## Non-Goals

- No PR/CI status in sidebar (research db cmux-011 — future PRD, needs GitHub
  auth design).
- No listening-ports / cwd display additions (separate small candidates).
- No agent auto-naming, no brand icons (cmux #7449 territory — later).
- No changes to notification/toast behavior or unread semantics.
- No detachable notification sidebar (TaskMaster #17 research stays parked).

## Technical Considerations

- Hook latency: hook shims run as short-lived CLI processes; the added
  `surface.agent_event` call must not slow agent hooks noticeably —
  fire-and-forget with short timeout (cmux #7410 precedent: fire-and-forget
  stop hooks).
- Event authenticity: any socket peer can emit `surface.agent_event` (same
  trust model as `notification.create` today — LocalUser mode). Acceptable
  for v1; note in the registry's restricted-surface review (Cursor lane) that
  `surface.agent_event` should NOT be in the Cursor-restricted allowlist.
- Aggregation lives with the state machine, not scattered in UI code.
- The 10-min decay timer must be testable with injected clock (no sleeps in
  tests).

## Success Metrics

- Operator confirms (PRD-C checklist addendum): with 3 concurrent agents, the
  sidebar correctly shows which one is waiting on input within ~2 s of the
  agent stopping.
- Rebuild-count assertion holds at 0 for the 20-workspace update scenario.

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-host-linux agent_state -- --nocapture
cargo test -p limux-cli hooks_agent_event -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended: fake-agent lifecycle + rebuild-counter + 8-combination visual matrix
```

## Rollback Plan

`git revert` feature commits; `surface.agent_event` unclassified → `-32601`;
sidebar falls back to current rendering. No persisted state to migrate.

## Open Questions

1. Indicator design: colored dot-with-glyph vs mini-icon set — visual call at
   implementation; acceptance only requires the distinctness matrix to pass.
2. Should `agents-status` also fold in `surface.health`? Default: yes if
   trivially available from the same call path, else defer.
