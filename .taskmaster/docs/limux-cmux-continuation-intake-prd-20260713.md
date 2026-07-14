# PRD: Limux cmux Continuation Intake And Promotion Gates

Date: 2026-07-13
Owner: Limux project manager
Target TaskMaster tag: `cmux-parity-20260707`
Parse mode: append to the existing non-empty tag only
Requested generated tasks: 6
Status: Drafted; parsing blocked by TaskMaster transactional-update incident

## 1. Source Authority

This PRD converts currently untracked work from these sources into a durable,
ordered TaskMaster intake program:

- `docs/cmux-parity-roadmap-20260706.md`, especially W1.5, Wave 2, Wave 3,
  sequencing, proposed PRDs, and verification discipline.
- `docs/cmux-parity-plan.md`, including the upstream-watch policy, current live
  bridge boundary, and deferred browser/phase work.
- `docs/research/cmux-upstream/README.md`
- `docs/research/cmux-upstream/items.md`
- `docs/research/cmux-upstream/sources.md`
- `docs/shortcut-remap-testing.md`

Existing TaskMaster tasks and PRDs remain authoritative for Wave 0 and Wave 1.
This PRD must not duplicate them.

## 2. Problem

The current `cmux-parity-20260707` tag represents PRD-A through PRD-H, but the
roadmap also names one untracked Wave-1 research deliverable and twelve Wave-2
or Wave-3 candidates. Creating implementation tasks for all twelve now would
contradict the roadmap: Wave-2/3 PRDs may be cut only after Wave 1 lands and a
fresh cmux/upstream Limux review is complete.

The durable task graph therefore needs an intake and promotion lane that:

1. captures W1.5 now;
2. proves Wave-1 closure before promotion;
3. refreshes volatile upstream evidence;
4. groups candidates into coherent execution PRDs without duplicating existing
   Cursor, pane-attention, agent-sidebar, notification, or hcom lifecycle work;
5. preserves explicit external-owner and operator gates.

## 3. Outcomes

### O1 - W1.5 decision evidence

Produce a timeboxed mux contract-alignment decision document that compares
Limux's GTK/core identifiers, method vocabulary, JSON framing, state ownership,
and bridge behavior with the current mux contract. The output states where
convergence is useful and whether a future mux backend remains architecturally
possible. This is research only; it does not implement a backend.

### O2 - Wave-1 closure packet

Establish an exact closure matrix for existing cmux-tag tasks 3-8. Each row must
name task status, source PRD, merged PRs, automated evidence, required live
evidence, residuals, and whether the row blocks Wave-2/3 promotion.

### O3 - Fresh upstream intake

Refresh cmux releases, open/recent PRs, issues, and upstream Limux sources after
Wave-1 closure. Update the research database with source dates and links. Score
each retained candidate for user impact, Linux/GTK fit, risk, overlap, and
verification cost.

### O4 - Wave-2 execution PRDs

After O2 and O3 pass, author focused execution PRDs for retained Wave-2 work.
The default grouping is:

1. terminal fidelity: W2.1 render/fractional scale, W2.2 IME/dead keys, W2.3
   shortcut contract, and W2.8 per-terminal font size;
2. runtime efficiency: W2.4 occluded-surface throttling;
3. workspace organization: W2.5 groups/saved layouts and W2.7 sidebar git/PR
   metadata;
4. command extensibility: W2.6 command palette and project-scoped
   `limux.json` commands.

Split or combine only when refreshed evidence proves a different ownership or
test boundary. Each PRD must identify overlap with existing TaskMaster tasks and
must not create a duplicate implementation lane.

### O5 - Wave-3 decision PRDs

After O2 and O3 pass, create decision-first PRDs for retained Wave-3 work:

- W3.1 Limux session lifecycle UX, with hcom remaining the resume/fork engine;
  hibernation has no Limux implementation authority until an hcom-side PRD and
  primitive exist.
- W3.2 SSH remote workspaces and detachable PTY daemon, requiring a security,
  authentication, reconnect, and resource-lifecycle design gate.
- W3.3 notification panel/unread jump/category gating, reconciled with existing
  pane-attention, workspace-sidebar, and agent-sidebar work.
- W3.4 extension/custom sidebar discovery only after a concrete internal need
  is recorded; otherwise defer with rationale.

### O6 - Parse-ready provenance and ordering

Every produced PRD must identify its target tag, source paths/sections, expected
task count, dependency chain, channel target, rollback, and acceptance tests.
Parsing uses the reviewed AI front door only after the active TaskMaster source
incident is cleared.

## 4. Generated Task Contract

When this PRD is parsed, generate exactly six parent tasks in this order:

1. **Run the W1.5 mux contract-alignment research spike.**
   Dependency: none or only already-done runtime-trust prerequisites. Deliver a
   decision document; no product code.
2. **Close and classify Wave-1 evidence.**
   Dependencies: existing cmux tasks 3, 4, 5, 6, 7, and 8 plus generated task
   1. Do not mark existing work done; produce the promotion matrix.
3. **Refresh cmux and upstream Limux research after Wave 1.**
   Dependency: generated task 2. Update research database and source dates.
4. **Author Wave-2 terminal-fidelity and runtime-efficiency PRDs.**
   Dependencies: generated tasks 2 and 3. Cover W2.1-W2.4 and W2.8 without
   implementation.
5. **Author Wave-2 organization and command-extensibility PRDs.**
   Dependencies: generated tasks 2 and 3. Cover W2.5-W2.7 without duplicating
   Cursor/sidebar tasks.
6. **Author Wave-3 decision PRDs and gate dispositions.**
   Dependencies: generated tasks 2 and 3. Cover W3.1-W3.4, preserving hcom,
   security, operator, and demonstrated-need gates.

The append parser is expected to allocate IDs after existing task 10, but the
actual returned range is authoritative. After parsing, literal provenance notes
must be appended to every generated parent.

## 5. Non-Goals

- No Wave-2 or Wave-3 product implementation in this PRD.
- No browser F2 work while `docs/decisions/browser-pane-architecture-20260707.md`
  remains provisional or cmux task 5 is incomplete.
- No duplicate task for master task 20 / cmux task 4 pane attention.
- No new resume/fork engine in Limux for hcom-managed sessions.
- No hibernation implementation before an hcom primitive exists.
- No cmux source or asset copying; translate concepts into Rust/GTK designs.
- No iOS, cloud VM/billing, vault sync, or freeform-canvas scope.

## 6. Acceptance Criteria

1. W1.5 has a committed decision document with sources, date, comparison table,
   recommendation, rejected alternatives, and impact on existing PRD-E/PRD-F.
2. The Wave-1 matrix covers tasks 3-8 and identifies every live/operator gate.
3. Research sources are refreshed after Wave-1 closure, not reused solely from
   the 2026-07-02 snapshot.
4. Every Wave-2/3 candidate has one disposition: promoted to a named PRD,
   merged into an existing task/PRD, deferred with gate, or rejected with
   rationale.
5. Produced PRDs have measurable outcomes, non-goals, dependencies, test
   strategy, preview/stable channel, rollout, rollback, and TaskMaster parse
   constraints.
6. No duplicate implementation parent is introduced across `master`,
   `cmux-parity-20260707`, and the Cursor lane.
7. All resulting dependency graphs validate through the reviewed wrapper after
   the TaskMaster source repair is installed and proven.

## 7. Verification

- Compare the W1.5 decision against current mux/cmux primary sources.
- Run the repo's research database validation/update method and inspect changed
  source dates.
- Cross-check every Wave-2/3 roadmap item against the final disposition table.
- Run `task-master-reviewed list --all-tags --with-subtasks --json` read-only and
  check for semantic duplicate scopes.
- After parsing is reauthorized, run dependency validation and require no
  string-ID or transactional-write invariant failures.
- Run `git diff --check` on all produced docs.

## 8. Rollout And Rollback

This PRD changes planning/task metadata only. It does not alter the Limux
runtime. Land the decision and PRD artifacts on a normal review branch.

If refreshed evidence invalidates a candidate grouping, update the draft PRD
before parsing. If parsing partially fails, stop, preserve hashes/output, and
use only the TaskMaster manager's supported repair; never edit the task store
manually or rerun with `--force`.

## 9. Parse Gate

Parsing is blocked until the TaskMaster manager clears the 2026-07-13
CLI-string ID/status-schema/transactional-write incident and validates a
supported repair against the frozen Limux evidence. When cleared:

- use `task-master-ai-reviewed parse-prd`;
- target `cmux-parity-20260707`;
- include `--append` because the tag is non-empty;
- request six tasks;
- do not use `--force` or `--research` unless the manager explicitly revises
  the command contract;
- capture before/after hashes and exact generated IDs.
