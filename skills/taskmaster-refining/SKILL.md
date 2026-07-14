---
name: taskmaster-refining
description: Reconcile a project's TaskMaster task graph with its PRDs, plans, decisions, evidence, and handoffs. Use when tasks are stale, under-specified, missing source links, unordered, duplicated, insufficiently expanded, or inconsistent with project documentation; also use when documented work is absent from TaskMaster and needs gated PRD intake.
metadata:
  classification: public-mechanics
  canonical_source: candidate-for-taskmaster-repo
---

# TaskMaster Refining

Bring TaskMaster and project documentation into evidence-backed parity without
manual task-store edits, duplicate work, or provider-driven drift.

Compose this skill with the active reviewed TaskMaster usage skill and the
project's own contributor rules. Project policy and package/runtime gates win
on conflict.

## Success Contract

At completion:

- every material non-archive project document maps to an exact task/PRD or an
  explicit non-task disposition;
- every active parent task names authoritative sources, acceptance evidence,
  priority, order, and same-tag dependencies; owner/gate is recorded in task
  details, a literal note, or the parity map because TaskMaster has no native
  owner field;
- material missing work has a reviewed PRD and tag-local parsed task range;
- complexity reports justify decomposition or an explicit atomic-task decision;
- duplicate and stale tasks have supported dispositions;
- dependency validation passes for every affected tag;
- before/after hashes and provider/runtime divergences are preserved.

## G0 - Freeze And Capability Gate

1. Establish one TaskMaster writer. Other sessions may inspect or draft docs,
   but must not mutate the task store.
2. Record branch, worktree, current tag, wrapper/runtime identity, and hashes of
   task store, state, and config.
3. Run the reviewed wrapper doctor and inspect the configured main, research,
   and fallback model roles without exposing credentials.
4. Inspect all tags and validate dependencies before mutation. Dependency
   validation is not structural validation: until the reviewed runtime ships a
   structural validator, require a manager-approved read-only JSON type check
   that every parent and subtask ID has the canonical type, or stop.
5. Confirm provider-backed mutation is safe on the installed runtime. Required
   invariants are numeric task IDs after CLI calls, one canonical parent-status
   set shared by CLI validation and AI schemas, and no task-store write after a
   proven failure. A proven failure is an explicit nonzero exit, a thrown
   terminal error, or a machine-readable failure result. Status parity must be
   bidirectional; for example, accepting `review` only in the CLI and `blocked`
   only in the AI schema violates the invariant.

If any invariant fails, or if terminal output is truncated or ambiguous, stop
all TaskMaster mutation. Truncated/ambiguous output is an independent stop
condition and is not proof that the command failed. Freeze command, output,
call IDs, exit evidence when available, before/after hashes, structural diff,
and affected IDs. Distinguish proven failure from ambiguous output. Continue
only read-only inventory and independent docs/skill drafting until the
TaskMaster owner ships and verifies a supported repair. Never normalize task
JSON manually.

## G1 - Build A Bidirectional Parity Map

Inventory:

- `.taskmaster/docs/` PRDs and seed records;
- product plans, roadmaps, decisions, incident reviews, verification docs, and
  research indexes;
- root release/contributor surfaces;
- project skills and operational runbooks;
- TaskMaster tags, parents, subtasks, statuses, priorities, dependencies, test
  strategies, and source pointers;
- merged/open PR evidence when the repository is GitHub-backed.

For each material source, record:

| Field | Required value |
|---|---|
| Source | Exact repository-relative path and useful section |
| Class | PRD, plan, decision, requirement, evidence, handoff, or non-task |
| Disposition | Existing task, new PRD, historical, deferred gate, or non-task |
| Order | Prerequisite, dependent, parallel, or terminal acceptance |
| Verification | Test, PR, live check, decision, or named gate |
| Owner | Task/project owner or explicit external-owner boundary |

Exclude generated/build/vendor/archive material deliberately and state the
rule. A zero-output coverage comparison between live source paths and map rows
is the inventory acceptance check.

## G2 - Reconcile Existing Tasks

For every active parent:

1. Preserve title/scope unless source evidence proves it stale.
2. Add exact source paths and section pointers.
3. Add observable acceptance and test strategy from those sources.
4. Reconcile status only from Git, test, live-runtime, or operator evidence.
5. Reconcile priority from impact, dependency criticality, risk, and current
   user pain.
6. Add same-tag dependencies only; use source pointers for cross-tag ordering.
7. Mark semantic duplicates through supported status/disposition commands and
   preserve the canonical task pointer.
8. Record why a small task remains atomic or flag it for complexity analysis.

Use literal non-AI note operations for factual evidence. Use provider-backed
updates only after G0 proves the transactional and ID invariants. Never edit
task JSON directly and never broad-stage unrelated files.

## G3 - Normalize Missing Work Into PRDs

Create a PRD only for material work that is not already represented. Each PRD
must include:

- source authority and affected existing tasks;
- measurable outcomes and non-goals;
- architecture/ownership boundaries;
- dependencies and promotion gates;
- executable acceptance and live/operator checks;
- rollout, rollback, and channel/runtime target;
- expected generated task count and target tag;
- duplicate-prevention and parse constraints.

Do not manufacture implementation PRDs when an authoritative roadmap requires
a research, decision, security, external-owner, or operator gate first. Create
the gate task/PRD and preserve that ordering.

## G4 - Parse With Provenance

Before each parse:

1. Recheck task-store hash and single-writer ownership.
2. Inspect whether the target tag is empty.
3. Use append mode only for a non-empty same-program tag.
4. Never use force mode during refinement.
5. Record the exact command and configured model role.

After each parse:

1. Capture the exact generated ID range and task-store hash.
2. Inspect every generated task for command, scope, status, dependency, and
   source drift.
3. Append literal source provenance to each generated parent.
4. Validate same-tag dependencies.
5. Stop on partial failure, malformed output, ID drift, or unexpected write;
   reconcile only through a supported owner-reviewed repair.

## G5 - Complexity Before Expansion

Generate one complexity report per affected product tag after contracts and
PRDs are stable. Review score, recommendation, current subtask coverage, and
cross-task dependencies.

Expand broad parents individually. Do not force expansion and do not use a
blind all-task expansion as the default. For `expand --all`, when the result
exposes aggregate acceptance fields, require:

- `failedCount == 0`;
- `expandedCount` equals the intended eligible count;
- completed subtasks remain intact;
- new subtasks have testable acceptance and useful sequence;
- every skipped active parent has an atomic-task rationale.

For targeted `expand --id`, record the task-store hash before and after, then
inspect the selected parent's actual subtask count, content, acceptance, and
sequence. Do not require aggregate fields from a targeted result that does not
expose them.

Re-run dependency validation and semantic duplicate review after expansion.

## G6 - Final Graph Review

For every affected tag, inspect:

- `list --with-subtasks` and focused `show` output;
- the computed `next` task;
- dependency validation;
- `complexity-report` and `status-packet` where those commands are installed
  and help-verified;
- source/test fields and remaining empty-field counts;
- complexity report and expansion outcomes;
- final task-store/config/state hashes.

Update the parity map with final task ranges, reports, hashes, blockers, and
owner-routed work. Keep TaskMaster status congruent with the live session
tracker. Commit and review the refinement as a normal repository change.

## Stop Rules

Stop TaskMaster mutation immediately when:

- another writer changes the task-store hash;
- a provider changes task identity or scope unexpectedly;
- a proven-failed command changes task-store bytes;
- CLI and AI surfaces do not share one canonical parent-status set;
- dependency validation masks a structural invariant failure;
- a non-empty tag would be parsed without append mode;
- expansion has any failed target;
- source ownership, operator authority, or duplicate scope is ambiguous.

Preserve evidence, route the defect, and continue only work that cannot further
corrupt the task graph.
