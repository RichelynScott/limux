# Limux TaskMaster Refinement Action Plan

Date: 2026-07-13
Status: Drafted from read-only evidence; TaskMaster mutation blocked on reviewed
runtime repair
Canonical map: `.taskmaster/docs/TASKMASTER_REFINEMENT_MAP.md`

## Execution Boundary

The live task store currently contains two confirmed string-ID parents created
through provider-backed updates: `master/15` and
`taskmaster-refinement-20260713/5`. All TaskMaster mutation is stopped until the
TaskMaster manager ships and verifies the supported repair. This file is the
exact reconciliation queue to apply afterward; it is not permission to edit
task JSON manually.

Current read-only hold evidence: task store
`219de423b704e6d599095372eb9f4b00632cd60582a0b27ac6b61e231446eaf9`;
all four tags pass dependency-edge validation, but direct structural inspection
still finds string parent IDs at `master/15` and
`taskmaster-refinement-20260713/5`. Six active product parents still have empty
test strategies: `master/16,17,20,23,24,25`.

Source-repair progress as of 2026-07-13: Sage has frozen an apply-clean numeric
parent-ID patch at
`/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/PATCH_FROM_Banach_20260713_update_task_numeric_parent_id.patch`
(SHA-256
`28ef4a65b8bc019081cfea2cf6a46c4354f1266b50cfaca700af3bd5e445ca5b`)
with independent evidence at
`/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/EVIDENCE_FROM_sage_20260713_update_task_numeric_id_patch.md`
(SHA-256
`78670858b3c0c7bacd089f2f45a92926e72e1487be351dc77c61c3fb238a4ec2`).
Static and Node checks pass, but focused Jest remains blocked because no reviewed
test dependency runtime is present. The patch is not integrated, pushed, or
installed, so this does not clear the mutation hold.

Supplemental root-cause evidence is frozen at
`/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/EVIDENCE_FROM_sage_20260713_update_task_schema_fallback_status_analysis.md`
(SHA-256
`78ec8eeb1b53604e7fca9edfd3e1c12ef7a0757aa52dda644a1594d592d1e92e`).
It confirms later-role success/write after earlier schema failures, drift from
one canonical seven-status set, `updatedAt`/full-task projection conflict,
malformed-JSON repair without schema revalidation, and final text-fallback
provider-order suppression. These defects require source fixes and focused
tests in addition to the numeric-ID patch; the evidence does not clear mutation.
The updated bounded source-worker brief is
`/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/BRIEF_TO_tmidfix_20260713_update_task_parent_id_type.md`
(SHA-256
`1bda528132727bfa6a253b99a477b9706eb9dc570d8eb5da24a06969c4c850d1`).
The canonical incident, now including the clean-lane launch diagnosis and
temporary tracking exception, is SHA-256
`52d437cfc94e1d9d6c56fe03f22ebe591c4a8feaaeb9e55c21ec0fe17bd0a2ff`.

## Recommended Cross-Tag Order

1. **Runtime/task-store repair gate.** Restore ID type invariants and prove
   complete terminal outcomes without losing the provider-derived task-15
   content or any later literal notes.
2. **Refinement-program self-reconciliation.** Align refinement tasks 2-6 with
   the Sage-approved skill and corrected source PRD before using them to mutate
   product tasks.
3. **Verification and active Wave 1.** Close evidence gaps in
   `cmux-parity-20260707/3-7` and `product-hygiene/1` before promoting new
   parity implementation.
4. **User-facing active fixes.** Continue `master/23-25`, with task 23's frozen
   PRD-I subtask ladder governing identity/restart work.
5. **Cursor v1 lane.** Execute `master/7,8,10,11`, then terminal acceptance task
   12 and boundary task 13.
6. **Future window/sidebar lane.** Review/close master 15, then design task 16
   and research task 17.
7. **cmux continuation intake.** Parse the six-task continuation PRD only after
   the task runtime repair. Run W1.5 research now; generate Wave-2/3 execution
   PRDs only after Wave-1 closure and a fresh upstream review.

Parallelism:

- Cursor tasks 7, 10, and 11 may proceed in parallel after their completed
  prerequisites; task 8 follows task 7; task 12 follows all Cursor feature
  tasks.
- master 23, 24, and 25 have distinct implementation surfaces but must share
  exact Surface/Pane, hcom, and shell-boundary terminology.
- cmux task 6 remains behind task 5. Tasks 3, 4, and 7 can close evidence in
  parallel where their source ownership does not overlap.

## Existing Task Reconciliation Queue

| Task | Current | Source/contract action | Dependency/order action | Status action after repair | Expansion decision |
|---|---|---|---|---|---|
| `master/7` | pending | Cursor PRD tree provider + review bundle now noted | Keep `4,5` | none | expand provider/tree/refresh/tests |
| `master/8` | pending | Cursor select/present + Wayland contract now noted | Keep `6,7` | none | expand Rust/extension/Xvfb/Wayland |
| `master/10` | pending | Cursor safe-path/security sources now noted | Keep `6` | none | expand metadata, validation, launch, tests |
| `master/11` | pending | Cursor restricted viewport boundary now noted | Keep `6` | none | small enough for 3-4 focused subtasks |
| `master/12` | pending | Cursor acceptance matrix now noted | Keep terminal dependency on `1-11` | none | expand by Node/Rust/Xvfb/acceptance script |
| `master/13` | in-progress | Clarify `docs/v2-boundary.md` is an output | Keep `5,6,9,11` | retain until file and scan exist | existing subtask first; expand only residuals |
| `master/15` | review, string ID | Source/test contract and 5 provider subtasks were later written | no dependency change while parked | retain review until PR #33 plus live edge/control acceptance | preserve current 5 subtasks after supported repair |
| `master/16` | pending | Window opacity/always-on-top source + verification noted | add hard dependency `15` | none | expand capability, persistence, UX, tests |
| `master/17` | pending | Detachable-sidebar source + research questions noted | add hard dependency `15`; soft coordination with `16` | none | expand decision/ownership/UX/test PRD |
| `master/20` | pending duplicate | Canonical source/task is cmux `4` | no new work | set `deferred` with canonical pointer | do not expand |
| `master/23` | in-progress | PRD-I/canonical contract and external HCOM PR pointer noted | existing 23.1-23.11 ladder; completed master `19,22` are provenance, not new blockers | retain | do not re-expand |
| `master/24` | in-progress | Recovery skill/source and missing restoration proof noted | soft completion gate on task 23 identity, no hard blocker on doc work | retain | expand identity, execution, proof, restoration after report |
| `master/25` | pending | Shell-boundary sources/events and byte fixtures noted | soft relationship to completed task 22 | none | expand parser/help, transport, integration, docs |
| `cmux/1` | done | replace nonexistent `MANIFEST.md` pointer with PRD-A, install manifest contract, PR #20 | none | preserve done | atomic historical |
| `cmux/2` | done | replace nonexistent `MANIFEST.md` pointer with PRD-B, installer-generated manifest contract, PR #21 | none | preserve done | atomic historical |
| `cmux/3` | in-progress | PRD-C + verification checklist/template | keep `1,2` | close only after exact preview run/write-back | expand live-run and write-back residuals |
| `cmux/4` | review | PRD-D + pane-attention plan + PR #22/#57 | keep `1,2` | retain until live border layering/clear-mode acceptance | expand only residual UI/live proof |
| `cmux/5` | in-progress | PRD-E + bridge review evidence | keep `4` | retain partial status | use existing subtask; expand remaining registry/mirror routes only |
| `cmux/6` | in-progress | PRD-F F1 + provisional decision skeleton | keep `5` | no F2 task/status promotion | expand decision evidence and ratification only |
| `cmux/7` | in-progress | PRD-G + lifecycle/staleness plan + hook docs | keep `5`; task 4 is a visual-composition soft gate | retain partial status | existing subtasks plus remaining families/UI/scale proof |
| `product-hygiene/1` | in-progress, subtask review | Product PRD + `0.2.1` PR #56 and changelog | no dependency | retain until isolated preview checklist/promotion evidence | existing release subtask sufficient |

## Refinement Program Self-Reconciliation

The parsed refinement tasks must be corrected through supported TaskMaster
commands after the source repair. The corrected source of truth is
`.taskmaster/docs/limux-taskmaster-refinement-prd-20260713.md` plus the
Sage-approved project-local skill SHA-256
`8eb3c2ede88a7c1021cc97d06b89db4895730b3883748650bd9a1937169f0b56`.

| Task | Current issue | Supported post-repair correction | Status/acceptance |
|---|---|---|---|
| `taskmaster-refinement-20260713/2` | Implies ownership reconciliation without stating where ownership lives | State that owner/gates are recorded in task details/literal notes or the parity map; add structural ID inspection alongside dependency validation | Keep in progress until all product-parent contracts are applied and verified |
| `taskmaster-refinement-20260713/3` | Artifact work is complete but task remains pending under mutation hold | Point to the continuation PRD and zero-output bidirectional coverage result | Mark done only after supported update and exact source review |
| `taskmaster-refinement-20260713/4` | Uses older ambiguous/partial-failure wording | Adopt explicit failure versus truncated/ambiguous outcome classification and the supported-repair-only rule | Remain pending until the continuation PRD is parsed with append mode and provenance |
| `taskmaster-refinement-20260713/5` | String parent ID; requires aggregate counts for targeted expansion and may name unsupported flags | Repair ID through the manager-supported path; use aggregate fields only for `expand --all`, and parent-specific hash/subtask inspection for `expand --id` | Park until repair; then run tag complexity reports and targeted expansion |
| `taskmaster-refinement-20260713/6` | Names nonexistent `roadmap`, omits status-packet dry-run, and prematurely routes to Niru | Use help-verified `list --with-subtasks`, `show`, `list --ready`, `next`, `complexity-report`, and `status-packet --dry-run`; keep skill local until G0 passes | Sage project-local review is complete; final validation/PR remains pending |

## Status Evidence Decisions

- PR #56 merged at `57347774852447032406eb9a350d16ac259fc401` with
  exact-head Codex clean review on `f79485a67afa8e513ae86d98ab578806bce29ea9`.
  It proves source/build gates, but its own promotion section requires an exact
  merged-main preview install and post-install checklist. Therefore
  `product-hygiene/1` remains in progress/review.
- PR #33 merged at `86c8b96e8ffa67b9cf0d6c9eee3f2bdd1c37dcfc`.
  It proves source landing for window chrome, not the remaining live edge/corner
  hitbox acceptance; `master/15` remains review.
- PR #22 merged at `67508d63c91aaef859e01f10210a57fa4eb69380`
  after a clean Codex review at `b304ce2b24dca6d646fec3418087269a107e87b1`.
  Operator evidence still reports pane-border visibility/behavior residuals, so
  cmux task 4 remains review.
- PR #57 merged at `f1db1d5a6005cfb2b2efe60422725728dccce48a`
  after a clean Codex review at `75d4960c2038c490e3b6f469bd2aa1c228cbaca6`.
  It advances identity/recovery but does not close PRD-I G4-G6; master task 23
  remains in progress.

## Missing Work Intake

`.taskmaster/docs/limux-cmux-continuation-intake-prd-20260713.md` is the only
new parse candidate found in this pass. It deliberately produces six planning
and research parents rather than premature Wave-2/3 implementation tasks:

1. W1.5 mux contract spike;
2. Wave-1 closure matrix;
3. refreshed upstream intake;
4. Wave-2 terminal fidelity/efficiency PRDs;
5. Wave-2 organization/command PRDs;
6. Wave-3 decision PRDs and gate dispositions.

Browser F2 is not a parse candidate while the decision skeleton remains
provisional. Existing master 16, 17, 24, and 25 already represent their source
plans and should be expanded, not duplicated by new PRDs.

## Post-Repair Mutation Sequence

1. Verify installed TaskMaster source commit and run the manager's ID/status/
   terminal-result regression smoke against a disposable fixture.
2. Freeze the Limux task-store hash and use the supported repair for tasks 15
   and refinement 5. Verify no other task changed.
3. Validate all four existing tags and inspect all parent ID types.
4. Apply exact source/test/status/dependency changes from this table through
   reviewed commands, validating and hashing after each tag.
5. Parse the continuation PRD with append mode into the cmux tag and attach
   literal provenance.
6. Run tag-specific complexity reports, targeted expansion, and the final
   dependency/duplicate review.
7. Restore the prior intended current tag, record final hashes, and update the
   canonical map.
