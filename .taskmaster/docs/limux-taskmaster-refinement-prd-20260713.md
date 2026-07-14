# PRD: Limux TaskMaster And Documentation Refinement

Date: 2026-07-13
Owner: lifo / LIMUX_MGR
Program tag: `taskmaster-refinement-20260713`
Status: Approved by operator for execution
Manager guidance:
`/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/GUIDANCE_TO_lifo_20260713_limux_taskmaster_refinement.md`
Guidance SHA-256:
`66ca155244926e238238e8f8f9b1893876c3747176a97ea05540195462770ec5`

## Objective

Make the Limux TaskMaster store and the project documentation describe the same
work, status, evidence, ordering, and dependencies. Every material project
document must map to an existing task, a newly parsed PRD/task range, or an
explicit non-task disposition. Every active task must identify its source,
carry sufficient implementation and verification detail, and be decomposed
when it is broader than one independently verifiable unit.

## Current Baseline

- Git base: `origin/main` at
  `f1db1d5a6005cfb2b2efe60422725728dccce48a`.
- TaskMaster runtime source:
  `98217f84de4842b3a4257445c0490f1513a4dc86`.
- Existing tags: `master`, `cmux-parity-20260707`, and `product-hygiene`.
- Existing top-level tasks: 38 total; 19 are not `done`.
- Fourteen parent tasks have an empty `testStrategy`.
- Thirteen active or review parent tasks have no subtasks.
- Existing dependency validation passes for the current `master` tag, but the
  graph has not been reviewed across all tags for product ordering.
- Before-state task-store SHA-256:
  `f9c3609a682f5f5921c57b832def7bb308d53b114f2020dd995a1f056d975328`.

## Source Documents

- `.taskmaster/docs/*.md`
- `docs/cmux-parity-roadmap-20260706.md`
- `docs/cmux-parity-plan.md`
- `docs/future-improvements/*.md`
- `docs/decisions/*.md`
- `docs/reviews/*.md` and the Cursor review bundle
- `docs/verification/*.md`
- root contributor, release, decision, handoff, and evidence documents
- repo-staged Limux skills and their references
- GitHub PR history through merged PR #57

The durable bidirectional disposition map is
`.taskmaster/docs/TASKMASTER_REFINEMENT_MAP.md`.

## In Scope

1. Inventory and classify every material non-archive project document.
2. Map each source to tag-local task IDs or an explicit non-task disposition.
3. Repair broken, missing, stale, or ambiguous source pointers in tasks.
4. Add source-derived scope, acceptance, failure, rollout, and verification
   details where current tasks are incomplete.
5. Reconcile stale statuses only when Git, PR, test, or operator evidence
   proves the transition.
6. Remove duplicate scheduling by cancelling/defering only through supported
   TaskMaster commands and preserving the canonical task pointer.
7. Establish same-tag dependencies and a recommended execution order.
8. Create companion PRDs for material future implementation programs that are
   described in docs but not represented in TaskMaster.
9. Parse those new PRDs through `task-master-ai-reviewed`, with `--append` only
   for non-empty same-program tags and literal provenance notes afterward.
10. Run tag-specific complexity analysis before targeted expansion.
11. Expand broad active tasks; record why any active task remains atomic.
12. Stage and dogfood a `taskmaster-refining` skill proposal, with the
    TaskMaster repository declared as the intended mechanics canonical.

## Non-Goals

- No Limux feature implementation, runtime install, restart, or activation.
- No manual edit of `.taskmaster/tasks/tasks.json`.
- No raw TaskMaster package execution, MCP startup, `npx`, or `npm exec`.
- No destructive task removal, plan deletion, or archive mutation.
- No rewrite of peer-owned handoffs.
- No Claude contact or Claude-side skill/config work during the emergency hold.
- No global skill promotion from this repository. Niru owns the later Codex
  promotion gate; Claude parity is separately gated after operator release.

## Work Breakdown Contract

### Outcome 1: Frozen inventory and parity map

- Record Git, wrapper, tag, task-store, and document inventory evidence.
- Classify each material document and record exact task or non-task disposition.
- Identify broken task links, duplicate tasks, missing PRDs, and stale status
  claims.

### Outcome 2: Existing task contract reconciliation

- Add exact source paths and section pointers.
- Ensure non-empty details and test strategy for every active parent.
- Reconcile priorities, operator gates, and completion evidence. Record owner
  boundaries in task details/literal notes or the parity map because TaskMaster
  has no native owner field.
- Apply explicit same-tag dependency corrections and validate after each set.

### Outcome 3: Missing PRDs and parsed tasks

- Preserve original source plans.
- Create companion PRDs with measurable outcomes, non-goals, acceptance,
  tests, dependencies, rollout, rollback, and task-generation constraints.
- Parse into the coherent existing program tag or a justified new tag.
- Capture exact new ID ranges and append literal provenance notes.

### Outcome 4: Complexity and targeted expansion

- Produce one complexity report per affected product tag.
- Review score, recommendation, and current subtask coverage.
- Expand broad tasks individually; never use `--force` in this pass.
- For `expand --all`, require `failedCount == 0` and the expected
  `expandedCount` when those aggregate fields are exposed. For targeted
  `expand --id`, verify the selected parent's before/after hash and actual
  subtask count/content instead.

### Outcome 5: Reusable skill pilot

- Stage `skills/taskmaster-refining/SKILL.md` as a Limux dogfood copy.
- Declare `/home/riche/MCPs/claude-task-master/skills/taskmaster-refining/`
  as the intended mechanics canonical and Sage as its owner.
- Include wrapper doctor, inventory/map, PRD normalization, tag guards,
  parse provenance, dependency review, complexity-before-expansion, targeted
  expansion, failure reconciliation, hashes, and promotion gates.
- Send the pilot and runtime divergence to Sage for source-owner review. Route
  the approved canonical candidate to Niru only after the repaired reviewed
  runtime passes G0; do not treat project-local approval as global promotion.

### Outcome 6: Final validation and durable closeout

- Validate dependencies for every affected tag.
- Verify `list --with-subtasks`, focused `show`, `list --ready`, `next --json`,
  the complexity report, and a dry-run status packet through installed,
  help-verified reviewed surfaces.
- Record final task-store and complexity-report hashes in the map.
- Run `git diff --check` and exact-path review.
- Commit, push, open a PR, and complete exact-head review before merge.

## Acceptance Criteria

- Every material document has one row in the refinement map.
- Every row has a classification, task/PRD or non-task disposition, owner, and
  verification pointer.
- Every active parent task has source provenance, implementation details, a
  test strategy or explicit verification rationale, correct priority, and
  valid same-tag dependencies.
- Every material untracked implementation program has a companion PRD and
  parsed tag-local task range.
- Complexity reports cover all intended active tasks and account for skipped
  statuses.
- Most broad active tasks are expanded; every atomic skip has a recorded reason.
- All task-scoped TaskMaster commands use explicit tags and the reviewed wrapper
  family, except commands whose installed help proves they are current-tag-only.
- No `expand --all` result is accepted unless exposed aggregate fields report
  `failedCount == 0` and the expected `expandedCount`; targeted expansion is
  accepted only after parent-specific before/after inspection.
- No task status is upgraded without evidence.
- The project-local skill pilot is reviewed by Sage. Niru routing waits for the
  repaired-runtime G0 gate; no Claude contact occurs during the emergency hold.

## Failure Behavior

- Stop on task-store hash drift from another writer and re-inventory.
- After an explicit nonzero exit, thrown terminal error, machine-readable
  failure, or truncated/ambiguous terminal result, stop mutation and inspect the
  live store. Do not call ambiguous output a proven failure, and reconcile only
  through an owner-reviewed supported repair; never restore by editing JSON.
- Stop a parse if the target tag is non-empty and `--append` was not supplied.
- Stop on duplicate task scope, invalid dependencies, unresolved source paths,
  or provider output that invents completed work.
- Preserve original plans and all evidence when a refinement decision is
  uncertain; record the blocker rather than deleting or overwriting material.

## Verification

```text
task-master-reviewed validate-dependencies --tag <tag>
task-master-reviewed list --tag <tag> --with-subtasks --json
task-master-reviewed list --tag <tag> --ready --json
task-master-reviewed show <id> --tag <tag> --json
task-master-reviewed next --tag <tag> --json
task-master-ai-reviewed complexity-report --tag <tag>
task-master-reviewed status-packet --tag <tag> --dry-run
sha256sum .taskmaster/tasks/tasks.json .taskmaster/reports/*.json
git diff --check
git status --short --branch
```

No command in this PRD authorizes a Limux runtime mutation or global skill
installation.
