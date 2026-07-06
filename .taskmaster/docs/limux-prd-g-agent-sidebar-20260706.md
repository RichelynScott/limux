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
| Hook event: agent started / prompt submitted / tool activity (per-agent vocabularies already parsed by `claude-hook`/`codex-hook`/etc.) | → `running` |
| Hook event: awaiting user input / permission request / notification-class "needs attention" | → `needs-input` |
| Hook event: stop/end-of-turn | → `idle` |
| No hook data for the surface (non-agent pane, hooks not installed) | `unknown` (never guessed) |
| Workspace focused + operator interacts (existing hover/focus-clear analog) | `needs-input` → `running` or `idle` per next hook event; the *visual urgency* clears like unread does today |

State is per-surface, aggregated to workspace as: any `needs-input` →
needs-input; else any `running` → running; else any `idle` → idle; else
unknown. Unread semantics are untouched — state is a separate field, not a
re-skin of unread.

## User Stories

### US-1: As the operator, I can see who needs me across 15 workspaces
- [ ] Sidebar rows render a state indicator (icon or colored glyph +
      accessible tooltip naming the state) for the aggregated workspace state.
- [ ] `needs-input` is visually dominant (distinct from unread dot AND from
      PRD-D attention border; the three coexist without ambiguity — a
      screenshot matrix in the Xvfb suite proves all 8 combinations render).
- [ ] State transitions arrive without focus changes (background workspace
      row updates when its agent stops — Xvfb fake-agent test).
- [ ] Stale-state decay: `running` with no hook events for a configurable
      period (default 10 min) degrades to `unknown` (never silently forever-
      running).
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
- [ ] Title/message/state updates mutate the affected row widget only —
      no full list-model rebuild per update (verified by instrumentation
      counter exposed via the existing `debug.*` surface, asserted in a test
      that pushes 100 updates across 20 workspaces and requires
      rebuild-count == 0).
- [ ] Batching: bursts of hook events within one frame coalesce to one row
      update (existing GTK idle/tick pattern; test with 50-event burst).
- [ ] Measured before/after: the test harness records wall-time for the
      100-update scenario; the PRD's merged result must include the numbers
      in the PR description (target: no worse than 2× single-update cost).

## Functional Requirements

1. State machine in a pure module (`rust/limux-host-linux/src/agent_state.rs`
   or `limux-core` if the socket-visible fields land there) — unit-testable
   without GTK; GTK wiring separate (`docs/maintainability.md`).
2. Hook path: extend the existing `hooks <agent> <event>` translation layer in
   `rust/limux-cli/src/main.rs` to ALSO emit a state-transition control call
   (new method `surface.agent_event` — additive, classified in the PRD-E
   registry as mutation; hook shims remain drop-in compatible: no user
   reconfiguration of `~/.claude/settings.json` etc. beyond `limux hooks
   setup` re-run).
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
