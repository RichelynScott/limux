# PRD-E: Live-Bridge Parity Core — ControlState Fall-Through

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:30 UTC
**Purpose:** Close the 18-vs-100+ method gap between the live GTK bridge and the
standalone dispatcher so the full Limux control vocabulary works against the
running GUI — the single architectural unlock the original parity plan named
("fixing this once, properly, makes phases 3–5 small") and the dependency for
live browser panes (PRD-F).

- **Priority:** P1 (Wave 1 — roadmap W1.1)
- **Dependencies:** SEQUENCING GATE — lifo's Cursor PR #15 (task #6 server-side
  restricted `cursor.*` surface) lands FIRST; the method-registry shape is then
  co-designed once (lifo agreement, hcom 2026-07-06 #302520). PRD-F depends on this PRD.
- **Effort:** M/L — **Tier 3 / high-stakes** per prd-generation-workflow
  (touches 2+ core files: `control_bridge.rs`, `window.rs`, `limux-core`);
  execution must be decomposed into small reviewed commits.
- **Channel targeting:** preview channel until PRD-C checklist passes on it

## Problem Statement

Limux has two control servers (`docs/cmux-parity-plan.md` §Architecture
discovery): the standalone `limux-control-server` speaks the full
`limux_core::Dispatcher` vocabulary (~100+ methods: full `workspace.*`,
`window.*`, `pane.*`, `surface.*`, `notification.*`, `tab.action`,
`browser.*`, `debug.*`), while the embedded GTK bridge
(`rust/limux-host-linux/src/control_bridge.rs`, `METHODS` table) routes exactly
18 methods and returns `-32601 unknown method` for everything else. In
practice, every tmux-compat verb (`swap-pane`, `break-pane`, `join-pane`,
`resize-pane`, `clear-history`, window navigation), `workspace.reorder/
move_to_window`, `surface.split/focus/close/move`, `notification.list/clear`,
and `tab.action` silently work headless but fail against the GUI the operator
actually runs. Agents driving Limux hit an arbitrary vocabulary wall.

## Goals

1. The live GUI answers the full read-only vocabulary via dispatcher
   fall-through on a live-synced `ControlState`.
2. A prioritized set of mutation methods gains real GTK side-effect routing.
3. One method registry (single source of truth) declares, per method:
   `bridge-native | fallthrough-read | gtk-mutation | deferred | restricted` —
   co-designed with the Cursor restricted surface so `cursor.*` and parity
   routing share one registry.

## Architecture (binding for implementation)

Per the parity plan's designed-but-unbuilt approach, refined:

- (Codex-required — the load-bearing unknown) **limux-core must grow a public
  API first; none of this exists today.** Verified: the only `pub fn`s in
  `rust/limux-core/src/lib.rs` are `Dispatcher::new` (:2757) and
  `Dispatcher::with_state` (:2763, consumes state by value into a NEW
  Arc<Mutex>); `ControlState` fields/methods are private; `dispatch_request`
  is private; and `Dispatcher::dispatch` is `async` (:2769, body never
  awaits) while bridge connection threads are sync. Therefore commit 2a of
  this PRD is a **limux-core FR**: (i) a pub shared-state constructor
  (`Dispatcher::with_shared_state(Arc<Mutex<ControlState>>)`), (ii) a pub
  SYNC dispatch entry, (iii) a pub snapshot-import/sync surface on
  `ControlState` so GTK state can be written into the mirror. This commit is
  reviewed by lifo before anything builds on it.
- The GTK app owns an `Arc<Mutex<ControlState>>` **mirror** kept in sync with
  live workspace/pane/surface/tab state at mutation points (create/close/
  rename/focus/reorder/split — the same code paths that already persist
  `layout_state`, e.g. window.rs:961 call sites).
- **Read-only methods** (`*.list`, `*.current`, `notification.list`; NOT
  `workspace.next/previous/last` — see mutation set; `surface.read_text`
  stays bridge-native) fall through against the mirror. CRITICAL INVARIANT:
  fall-through must be read-only — the bridge MUST NOT let a mutating method
  reach the mirror dispatcher, or mirror and GUI diverge silently. Enforce
  via the registry classification, deny-by-default: unclassified methods
  stay `-32601`.
  (Codex-required — read-path mutation contamination is REAL and broad:
  `with_workspace_scope` (lib.rs:3118-3134), used by `surface.current`,
  `surface.list`, and every workspace-scoped read, calls `select_workspace`
  on entry AND restore, unconditionally clobbering `last_workspace_id` and
  marking notifications read when app-active.) **Binding mitigation:
  fall-through reads dispatch against a per-request CLONE of the mirror**
  (`ControlState: Clone`, lib.rs:630) — correctness-safe by construction;
  the cost (cloning surface text buffers per control-plane request) is
  accepted and documented. A core read-path refactor (`&ControlState` reads)
  is a later optimization, not v1.
- **Mutation methods** get explicit `ControlCommand` variants executed on the
  GTK main loop (the existing pattern for the 18 natives), updating both the
  GUI and the mirror.
- **ID mapping layer:** limux-core uses `u64` ids; host-linux uses `String`
  workspace ids, `u32` pane ids, uuid-`String` tab ids (repo CLAUDE.md
  pitfall). A boundary map alone is NOT sufficient (Codex-required):
  dispatcher RESPONSES embed ids in three non-generic shapes — synthetic
  uuids via `encode_handle_id` (`"00000000-…-{id:012x}"`, lib.rs:2847),
  `workspace:N`/`pane:N`/`surface:N` refs, and raw `u64` fields nested in
  serialized Info structs (`WorkspaceInfo`/`SurfaceInfo`, lib.rs:277-312,
  where a raw `id: 7` is indistinguishable from `flash_count: 7`
  generically). Therefore: a **schema-aware, per-method-family response
  id-rewrite** is a named FR — each fall-through family declares which
  response fields are ids and maps them to host-native id forms so agents
  see ONE coherent id vocabulary across native and fall-through methods.
  Wire-visible ids remain EXACTLY what today's bridge emits (no breaking
  change to `refs`/`uuids` id-format behavior).
- **`SurfaceInfo.text` divergence:** mirror-backed surface reads emit `text`
  as empty/omitted (the mirror does not sync terminal text — a hot path
  deliberately kept bridge-native via `surface.read_text`); documented +
  tested, so headless-vs-live divergence is explicit, never silent.
- **`window.list` semantics:** the mirror exposes exactly one window per
  workspace, id = mapped host window (host-linux has no multi-window
  structure; core models windows-within-workspaces, lib.rs:286-291).

## User Stories

### US-1: As an agent in a pane, read introspection works identically live and headless
- [ ] Every read-only method in the registry returns live-truth data against
      the running GUI: at minimum `workspace.list/current`, `window.list/current`,
      `pane.list/surfaces`, `surface.list/current/health`,
      `notification.list`, plus the read legs of tmux-compat verbs
      (`find-window`, `list-buffers`, `display-message` resolution).
- [ ] For each read family, an Xvfb smoke assertion compares bridge output vs
      mirror-dispatcher output for structural equality **modulo the declared
      id-mapping and the documented `text`-field divergence** (Codex-revised
      — raw equality is not executable across the two id vocabularies).
- [ ] Unknown/unclassified methods still return `-32601` (deny-by-default
      regression test).
- [ ] `system.capabilities` reports the registry: method name → routing class,
      so agents can feature-detect instead of trial-and-erroring.

### US-2: As an agent, the priority mutation set works against the live GUI
- [ ] Wave-1 mutation set, each as a `ControlCommand` with a focused test:
      `pane.focus`, `pane.resize` (+ `resize-pane`), `surface.split`
      (+ split-direction args), `surface.focus`, `surface.close`,
      `workspace.reorder`, `workspace.next/previous/last` (Codex-confirmed
      MUTATION class — `select_workspace_relative`/`select_last_workspace`
      write selection state + mark notifications read, lib.rs:882-909; OQ1
      is hereby CLOSED as mutation), `tab.action` (all existing actions),
      `notification.clear`.
- [ ] Each mutation updates GUI AND mirror atomically (single main-loop hop);
      a follow-up read via fall-through reflects the mutation (test per method).
- [ ] Deferred mutations (`pane.swap/break/join`, `surface.move/reorder/
      drag_to_split`, `workspace.move_to_window`, `window.create/close/focus`)
      are explicitly classified `deferred` in the registry and still `-32601`
      — documented, not silently broken.
- [ ] Typed-PTY control-character guard and `send_text` readiness-conflict
      semantics unchanged (regression tests must stay green).

### US-3: As the Cursor lane, the restricted surface and parity share one registry
- [ ] The method registry is one Rust source of truth that BOTH the parity
      routing and the Cursor-restricted allowlist consume
      (`integrations/cursor-limux/methods.json` is generated from or verified
      against it by a Rust check-in-sync test — no drift possible). NOTE:
      methods.json currently names methods that exist only in lifo's
      unmerged PR #15 (`window.present`, `cursor.*`) — the drift test lands
      only AFTER the PR-#15 sequencing gate clears.
- [ ] Registry shape is co-designed with lifo AFTER PR #15 merges (do not
      churn PR #15); the PRD execution's first commit is the registry
      extraction, reviewed by lifo before the fall-through lands.
- [ ] `SocketControlMode`/auth behavior unchanged for non-restricted callers.

## Functional Requirements

1. New module `rust/limux-host-linux/src/control_registry.rs` (or
   `limux-control` crate location if lifo's PR #15 already created a registry
   home — reuse, don't duplicate) declaring the classification table.
2. Mirror sync in `window.rs` at the same call sites that mutate pane/
   workspace/tab state; keep sync logic in its own module
   (`state_mirror.rs`), pure-testable without GTK where feasible
   (`docs/maintainability.md` discipline).
3. Bridge fall-through in `control_bridge.rs`: registry lookup → native |
   fallthrough (read) | ControlCommand (mutation) | -32601.
4. No changes to socket framing, error codes, or existing 18 methods' wire
   shapes.
5. Commit decomposition (execution order, Codex-revised): (1) registry
   extraction + deny-by-default (no behavior change), (2a) **limux-core
   public API** (shared-state constructor, pub sync dispatch, snapshot-
   import/sync surface) — reviewed by lifo before 2b, (2b) mirror + sync +
   clone-per-read fall-through + id-rewrite layer, (3) mutation set in 2–3
   commits by family, (4) capabilities reporting + docs. Each commit passes
   `./scripts/check.sh` (no NEW failures vs baseline) + Xvfb smoke.

## Non-Goals

- No `browser.*` routing (PRD-F owns it, on top of this registry).
- No `debug.*` exposure on the live bridge beyond what exists (test surface
  stays standalone/Xvfb).
- No new socket auth model (Cursor restricted surface is lifo's lane).
- No wire-format/id-format changes.
- No `window.*` multi-window mutation support in Wave 1 (deferred class).

## Technical Considerations

- Mirror-divergence is THE failure mode: add a `debug`-surface invariant check
  (mirror vs GUI walk) run in Xvfb smoke after every mutation family test.
- Lock discipline: never hold the mirror mutex across a GTK main-loop dispatch
  (deadlock risk — bridge thread vs main loop); document the ordering rule in
  `state_mirror.rs` header.
- `Dispatcher::dispatch` may have side effects even for "read" paths (e.g.
  `workspace.next` semantics in core select a workspace) — the registry
  classification must be decided by READING each core method's implementation,
  not by name; the classification table in the PRD execution's first commit
  requires a reviewer sign-off (lifo or subagent lens review).

## Success Metrics

- Method-coverage table (generated from the registry) shows 0 read-only
  methods unclassified; all Wave-1 mutation set live-routed.
- tmux-compat verbs listed in US-2 work from inside a live pane on the
  operator's machine (PRD-C checklist addendum item).

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-host-linux registry -- --nocapture
cargo test -p limux-host-linux state_mirror -- --nocapture
cargo test -p limux-host-linux bridge_fallthrough -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended: per-family live assertions + mirror-invariant check
```

## Rollback Plan

Registry is deny-by-default: reverting the fall-through/mutation commits
returns exactly today's 18-method behavior. Ship behind an env kill-switch
with precise semantics (Codex-revised): `LIMUX_BRIDGE_PARITY=0` ⇒ all
fall-through + new mutation methods return `-32601` byte-identical to today,
the 18 natives unaffected — with a regression test asserting exactly that.
Checked once at bridge init, so disabling requires a GUI restart (accepted,
documented).

## Open Questions

1. ~~`workspace.next/previous/last` classification~~ — CLOSED: mutation class
   (code-verified, lib.rs:882-909).
2. Registry home: host-linux module vs `limux-control` crate — decide with
   lifo at co-design (whichever PR #15 chose wins).
