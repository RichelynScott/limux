# Limux TaskMaster Refinement Map

Date: 2026-07-13
Owner/writer: lifo / LIMUX_MGR
Status: In progress
Program: `taskmaster-refinement-20260713`
Method: Sage guidance SHA-256 `66ca155244926e238238e8f8f9b1893876c3747176a97ea05540195462770ec5`

## Frozen Baseline

| Evidence | Value |
|---|---|
| Git base | `f1db1d5a6005cfb2b2efe60422725728dccce48a` |
| Working branch | `lifo/taskmaster-doc-parity-refinement-20260713` |
| TaskMaster runtime source | `98217f84de4842b3a4257445c0490f1513a4dc86` |
| Task store SHA-256 | `f9c3609a682f5f5921c57b832def7bb308d53b114f2020dd995a1f056d975328` |
| State SHA-256 | `4e3477c8aaf3978c45dd38ce6a8111f06fd20fa60bd1528abef1f135a27a180f` |
| Config SHA-256 | `b0ac58564828d26f0eac84c504fdbfbf11c21ad37a7a059e1c0c071cdbc09df8` |
| Existing tags | `master` (25), `cmux-parity-20260707` (10), `product-hygiene` (3) |
| Existing parents | 38 total; 19 not done |
| Field gaps | 14 parents without `testStrategy`; 13 active/review parents without subtasks |
| Dependency check | `master` valid before mutation; other tags pending explicit validation |

## Current Mutation-Hold Snapshot

Read-only evidence after the provider-backed operations and before any supported
repair:

| Evidence | Current value |
|---|---|
| Task store SHA-256 | `219de423b704e6d599095372eb9f4b00632cd60582a0b27ac6b61e231446eaf9` |
| Config SHA-256 | `844313ec907fc9e6c3d8cc32aae2e822b312c3ec70fa5b3299fa4007e6dc768b` |
| State SHA-256 | `cf38828188863f48bff9c473875daed1f9166ea31eeb87f7a92e73e8464b356e` |
| Structural ID check | Invalid string parent IDs: `master/15`, `taskmaster-refinement-20260713/5`; no non-numeric subtask IDs found |
| Dependency-edge check | All four tags pass `validate-dependencies`; this does not prove structural ID validity |
| Active parent test gaps | Six: `master/16,17,20,23,24,25` |
| Active/review parents without subtasks | Seventeen, including five refinement-program parents; expansion remains blocked |

All TaskMaster mutation remains stopped. These counts describe the live hold
state and do not supersede the frozen before-state baseline above.

## Disposition Legend

- `existing`: represented by the named tag/task.
- `new-prd`: implementation work requires a companion PRD and parse.
- `historical`: completed evidence; no new task unless a residual is named.
- `requirement`: standing contributor/runtime requirement applied to related tasks.
- `decision`: records a gate or selected direction; task points to it.
- `evidence`: supports status/verification but does not independently create work.
- `handoff`: resume/routing surface; not authoritative when stale or peer-owned.
- `non-task`: external, superseded, generated, or reference-only material.

Complexity and expansion cells are updated after the tag-specific reports. A
blank task assignment is not allowed at final closeout.

## Root And Coordination Surfaces

| Source | Class | Disposition / TaskMaster | Order / dependency | Complexity / expansion | Evidence / owner |
|---|---|---|---|---|---|
| `AGENTS.md` | requirement | Applies to every active task; no separate task | Verification and ownership guard | Non-task | Repo canonical / lifo |
| `CLAUDE.md` | requirement | Applies to implementation tasks; no separate task | Contributor/runtime guard | Non-task | Repo canonical / repo owners |
| `README.md` | requirement | `product-hygiene/1-2`, `cmux-parity-20260707/1-3`, `master/1-13` | Updated after shipped behavior | Pending report | Git/PR evidence / lifo |
| `CHANGELOG.md` | release evidence | `product-hygiene/1-2` | Follows merged release work | Pending report | PR #43 and #56 / lifo |
| `FYI.md` | evidence journal | Task-specific evidence only; no parse | Append-only; not roadmap authority | Non-task | Multi-owner journal |
| `HANDOFF.md` | handoff | Peer-owned and stale; explicit non-task | Do not edit; current state lives in owned handoff/map | Non-task | halo-owned route-only |
| `LIFO_HANDOFF.md` | handoff | `master/23-25` plus this refinement program | Refresh at closeout only | Non-task | lifo-owned |
| `HALO_HANDOFF.md` | handoff | Historical peer state | No active scheduling authority | Non-task | halo-owned |
| `NATO_HANDOFF.md` | handoff | Historical PRD A-H handoff | Superseded by merged PRs and live tasks | Non-task | nato-owned |
| `LIFO_CL_MGR_HANDOFF.md` | handoff | Historical product-hygiene lane | Superseded by PR #43/#45/#56 | Non-task | lifo historical |
| `LIFO_INBOX/TASK_FROM_kuma_2026-07-12_visibility-restart-design-commission.md` | requirement | `master/23`, `.taskmaster/docs/limux-prd-i-*` | G1-G5 contract gates | Report under `master/23` | Kuma intake / lifo+dino |
| `LIMUX_SECURITY_DEPENDENCY_REVIEW_2026-06-17.md` | evidence | `cmux-parity-20260707/2`; historical security input | Precedes packaging/install changes | Atomic evidence | Security review / lifo |
| `hooks/README.md` | requirement | `cmux-parity-20260707/7`, `master/23-25` | Hook behavior precedes sidebar truth | Pending report | Hook docs / lifo |

## TaskMaster PRDs And Seed Records

| Source | Class | Disposition / TaskMaster | Order / dependency | Complexity / expansion | Evidence / owner |
|---|---|---|---|---|---|
| `.taskmaster/docs/hermes-workspace-highlight-resize-20260627.md` | historical plan | PR #6; residual verification in `cmux-parity-20260707/3`; pane attention canonical is `cmux-parity-20260707/4` | Verification after current build identity | Pending report | PR #6 / lifo |
| `.taskmaster/docs/limux-copy-paste-defect-20260622.md` | historical plan | PR #4; no active residual currently proven | Historical completion | Atomic historical | PR #4 / lifo |
| `.taskmaster/docs/limux-cursor-ide-integration-prd-20260630.md` | authoritative plan | `master/1-13` | Existing graph; acceptance task 12 last | Report `master`; expand active 7-13 selectively | PR #8-#17 / lifo |
| `.taskmaster/docs/limux-prd-a-runtime-trust-20260706.md` | authoritative plan | `cmux-parity-20260707/1` | Unblocks task 3 | Done; no expansion | PR #20 |
| `.taskmaster/docs/limux-prd-b-ghostty-packaging-20260706.md` | authoritative plan | `cmux-parity-20260707/2` | Unblocks task 3 | Done; no expansion | PR #21 |
| `.taskmaster/docs/limux-prd-c-verify-loop-20260706.md` | authoritative plan | `cmux-parity-20260707/3` | Depends on 1 and 2 | Pending report; expand live-run closeout | PR #23 + run evidence pending |
| `.taskmaster/docs/limux-prd-d-pane-attention-20260706.md` | authoritative plan | `cmux-parity-20260707/4`; `master/20` is duplicate scheduling to reconcile | Verify through task 3 | Pending report | PR #22/#57 + live check |
| `.taskmaster/docs/limux-prd-e-bridge-parity-20260706.md` | authoritative plan | `cmux-parity-20260707/5` | Browser F2 depends on completion | Pending report; task already partly expanded | PR #24/#25/#39-#44 |
| `.taskmaster/docs/limux-prd-f-browser-live-20260706.md` | authoritative plan | `cmux-parity-20260707/6` for F1; F2 remains gated until decision | Task 5 before F2 | Pending report; expand F1 evidence | Decision skeleton incomplete |
| `.taskmaster/docs/limux-prd-g-agent-sidebar-20260706.md` | authoritative plan | `cmux-parity-20260707/7` | Soft order after task 4; coordinate with task 5 | Pending report; expand remaining families/UI | PR #27 partial |
| `.taskmaster/docs/limux-prd-h-restore-pack-20260706.md` | authoritative plan | `cmux-parity-20260707/8` | Independent; done slices 8.1-8.3 | Done; no expansion | PR #28/#36-#38 |
| `.taskmaster/docs/limux-prd-i-hcom-visibility-restart-integration-20260713.md` | authoritative plan | `master/23`, subtasks 23.1-23.11 | G3 HCOM dependency; 23.11 operator-gated | Existing expansion sufficient; report only | PR #57 + canonical contract |
| `.taskmaster/docs/product-hygiene-version-and-tab-rename-20260707.md` | authoritative plan | `product-hygiene/1-3` | Release verification closes task 1 | Pending report; reconcile 1/1.1 status | PR #43/#45/#56 |
| `.taskmaster/docs/workspaces-sidebar-notifications-20260620.md` | historical plan | PR #3; active residuals map to `master/17` and `cmux-parity-20260707/4,7` | Historical base before active tasks | Pending report | PR #3 / lifo |
| `.taskmaster/docs/limux-taskmaster-refinement-prd-20260713.md` | authoritative plan | `taskmaster-refinement-20260713/1-6` | Governs this pass only | Analyze then expand as needed | Operator + Sage guidance |
| `.taskmaster/docs/limux-cmux-continuation-intake-prd-20260713.md` | authoritative continuation intake | Parse six tasks into `cmux-parity-20260707` after TaskMaster repair | W1.5 -> Wave-1 closure -> upstream refresh -> Wave-2/3 PRDs | Parse/complexity blocked on source repair | lifo |
| `.taskmaster/docs/TASKMASTER_REFINEMENT_MAP.md` | authoritative parity map | `taskmaster-refinement-20260713/1-6` | Updated at every refinement gate | Non-product tracker | lifo-owned |
| `.taskmaster/docs/TASKMASTER_REFINEMENT_ACTION_PLAN.md` | execution queue | Exact post-repair status/dependency/expansion sequence | Apply only after TaskMaster repair | Non-product tracker | lifo-owned |
| `.taskmaster/reports/TASKMASTER_RUNTIME_DIVERGENCE_20260713.md` | runtime evidence | TaskMaster manager triage; refinement task `5` mutation hold | Preserve before further task-5 operations | Non-product evidence | lifo -> Sage |
| `.taskmaster/reports/TASKMASTER_UPDATE_REVIEW_STATUS_FAILURE_20260713.md` | runtime defect evidence | TaskMaster manager triage; `master/15` mutation hold | Truncated terminal result; later write stringified ID | Non-product evidence | lifo -> Sage |

## Product Plans, Decisions, And Future Work

| Source | Class | Disposition / TaskMaster | Order / dependency | Complexity / expansion | Evidence / owner |
|---|---|---|---|---|---|
| `docs/cmux-parity-plan.md` | roadmap | `cmux-parity-20260707/1-8`; later candidates through new continuation PRD | Stability before feature parity | Pending continuation report | Repo roadmap / lifo |
| `docs/cmux-parity-roadmap-20260706.md` | roadmap | Tasks 1-8 cover W0/W1.1-W1.4; continuation-intake PRD covers W1.5 and gated W2/W3 PRD promotion | Append to coherent cmux tag after repair | Six-task parse pending source repair | Nato roadmap / lifo |
| `docs/decisions/browser-pane-architecture-20260707.md` | decision skeleton | `cmux-parity-20260707/6`; no F2 task until ratified | Hard gate after task 5 | Expand F1 evidence work | Evidence cells still TBD |
| `docs/future-improvements/hcom-limux-session-pane-visibility-restart-design-20260712.md` | authoritative contract | `master/23` | G1-G6 gated sequence | Existing 23.1-23.11 | Canonical hash in task 23 |
| `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md` | authoritative plan | `master/1-13` | Existing Cursor graph | Report `master`; expand active parents | Cursor review bundle |
| `docs/future-improvements/limux-cursor-integration-options-after-pr-greenlight.md` | historical options | Superseded by Cursor PRD and `master/1-13` | No separate parse | Historical | PR #5 + Cursor PRD |
| `docs/future-improvements/limux-dual-runtime-runbook-20260702.md` | runbook | `master/19` | Operational use after channel implementation | Done; no expansion | PR #7 |
| `docs/future-improvements/limux-global-use-guide-and-hcom-routing-20260707.md` | requirement/plan | `master/22-25`, `cmux-parity-20260707/5,7` | Mechanics track shipped behavior | Pending active task reports | PR #30/#50/#55/#57 |
| `docs/future-improvements/limux-lifecycle-events-and-agent-team-staleness-20260707.md` | requirement plan | `cmux-parity-20260707/5.1,7.1,7.2` | 5.1 can feed 7.1; 7.2 done | Expand 5.1/7.1 under parents | PR #31/#52/#53 |
| `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md` | requirement plan | Canonical `cmux-parity-20260707/4`; duplicate `master/20` to reconcile | Verify through task 3 | Pending report | Screenshot + PR #22/#57 |
| `docs/future-improvements/limux-runtime-channel-contract-20260702.md` | authoritative contract | `master/19` | Precedes isolated preview work in task 23 | Done; no expansion | PR #7 |
| `docs/future-improvements/limux-runtime-isolation-and-window-ui-plan-20260701.md` | authoritative plan | `master/15-19` | 15/19 before future 16/17 | Report `master`; expand 16/17 | PR #6/#7/#33 |
| `docs/future-improvements/limux-runtime-isolation-surface-audit-20260702.md` | evidence/requirements | `master/19` | Before channel implementation | Done; no expansion | PR #7 |
| `docs/shortcut-remap-testing.md` | requirement | `cmux-parity-20260707/8`; future W2.3 in continuation PRD | Applies to shortcut changes | Pending continuation report | Test doc / lifo |
| `docs/terminal-input-regression-20260701.md` | incident/evidence | `master/14`, `cmux-parity-20260707/2` | Packaging/input regression lineage | Historical done | PR #21 and terminal fixes |
| `docs/taskmaster-tag-reconciliation-20260707.md` | historical process record | This refinement program supersedes its worktree-specific inspection rule | No product task | Atomic historical | TaskMaster reconciliation evidence |
| `docs/project-isolation-lab-goal.md` | superseded external handoff | `non-task`; owned by SUPPLY_CHAIN_SECURITY | Explicitly outside Limux | Non-task | External owner gumo |
| `docs/limux-vs-multica-decision-guide.md` | decision/historical | `non-task`; Limux+hcom selected, Multica deferred | Revisit only by operator decision | Non-task | Operator decision |

## Verification, Incident, Research, And Review Evidence

| Source | Class | Disposition / TaskMaster | Order / dependency | Complexity / expansion | Evidence / owner |
|---|---|---|---|---|---|
| `docs/verification/post-install-checklist-v1.md` | verification requirement | `cmux-parity-20260707/3` | After tasks 1 and 2 | Expand task 3 closeout | Operator run pending/current evidence review |
| `docs/verification/run-template.md` | generated-run template | `cmux-parity-20260707/3` | Used per release candidate | Non-task template | Verification owner |
| `docs/verification/wave1-morning-summary-20260707.md` | evidence | `cmux-parity-20260707/5-8` | Historical merged-wave snapshot | Non-task evidence | PR #26 |
| `docs/reviews/limux-crash-restore-logflood-20260710.md` | incident/evidence | `master/21` | Completed before later restore work | Done | PR #49 |
| `docs/reviews/prd-e-core-api-2a-review-20260707.md` | review evidence | `cmux-parity-20260707/5` | Precedes bridge fallthrough commits | Non-task evidence | PR #24/#25 |
| `docs/reviews/limux-cursor-ide-integration-20260630/REVIEW_BRIEF.md` | review input | `master/1-13` | Before Cursor implementation | Non-task evidence | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/MANAGER_SYNTHESIS.md` | review synthesis | `master/1-13` | Shapes PRD and graph | Non-task evidence | lifo synthesis |
| `docs/reviews/limux-cursor-ide-integration-20260630/MINIMAX_SCRIM_WRAPPER.md` | historical tooling note | `master/1-13`; no current execution authority | Historical only | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-architecture.md` | review evidence | `master/1-13` | PRD input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-control-trust.md` | review evidence | `master/1-13` | PRD input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-cursor-api.md` | review evidence | `master/1-13` | Superseded by rerun where applicable | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-cursor-api-rerun.md` | review evidence | `master/1-13` | PRD input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-glm-runtime.md` | review evidence | `master/1-13` | PRD input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-runtime.md` | review evidence | `master/1-13` | PRD input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-security.md` | review evidence | `master/1-13` | PRD trust-boundary input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-sequencing.md` | review evidence | `master/1-13` | Graph input | Non-task | Review bundle |
| `docs/reviews/limux-cursor-ide-integration-20260630/hcom-minimax-tests.md` | review evidence | `master/1-13` | Test strategy input | Non-task | Review bundle |
| `docs/research/cmux-upstream/README.md` | research method | `master/18`; future continuation PRD | Refresh at wave boundaries | New PRD references | Research database |
| `docs/research/cmux-upstream/items.md` | research backlog | `master/18`; unpromoted items flow through continuation PRD | Score before implementation | New PRD references | Research database |
| `docs/research/cmux-upstream/sources.md` | evidence index | `master/18`; future continuation PRD | Source provenance | Non-task evidence | Research database |
| `docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/README.md` | evidence | `cmux-parity-20260707/2` | Historical packaging evidence | Non-task | Ghostty lane |
| `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md` | decision/gate | `cmux-parity-20260707/2` | Applies only if full Zig lane reopens | Non-task gate | Historical review |
| `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_REVIEW_REQUEST_2026-05-29.md` | review input | `cmux-parity-20260707/2` | Historical | Non-task | Historical review |
| `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md` | review evidence | `cmux-parity-20260707/2` | Historical mutation gate | Non-task | Historical review |
| `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | review evidence | `cmux-parity-20260707/2` | Historical install gate | Non-task | Historical review |
| `docs/LIMUX_PHASE5C_NEXT_STEPS_DECISION_PACKET_2026-05-29.md` | decision/historical | `non-task`; shipped agent-team lineage documented elsewhere | Superseded | Non-task | Historical decision |
| `docs/install-security-report-2026-05-29.md` | security evidence | `cmux-parity-20260707/2` | Packaging/install input | Non-task | Historical security review |
| `docs/maintainability.md` | requirement | Applies to every implementation task | Standing code-quality gate | Non-task | Repo canonical |
| `docs/limux-hcom-workflow.md` | requirement/runbook | `master/22-25`, `cmux-parity-20260707/7` | Runtime coordination boundary | Pending active reports | Repo workflow guide |

## Staged Skills

| Source | Class | Disposition / TaskMaster | Order / dependency | Complexity / expansion | Evidence / owner |
|---|---|---|---|---|---|
| `skills/limux-a2a/SKILL.md` | requirement/tooling | `master/23-25` | Must follow exact Surface/Pane targeting | Pending active reports | PR #57 / lifo |
| `skills/limux-use-guide/SKILL.md` | requirement/tooling | `master/22-25` | Mirrors verified Limux mechanics | Pending active reports | PR #30/#57 / lifo |
| `skills/limux-use-guide/README.md` | skill metadata | `master/22` | Promotion metadata only | Non-task | Repo-staged skill |
| `skills/reconcile-via-limux/SKILL.md` | tooling | `master/22` | Completed reconciliation method | Done | PR #50 |
| `skills/reconcile-via-limux/assets/cleanup-brief-template.md` | skill asset | `master/22` | Used by skill | Non-task asset | PR #50 |
| `skills/reconcile-via-limux/references/command-surface.md` | skill reference | `master/22` | Used by skill | Non-task reference | PR #50 |
| `docs/skills/reconcile-via-limux-prd-lite-20260710.md` | authoritative skill plan | `master/22` | Completed | Done | PR #50 |
| `skills/taskmaster-refining/SKILL.md` | skill pilot | This refinement program; mechanics candidate for TaskMaster canonical | Sage-approved project-local candidate; repaired-runtime G0 before global promotion | Runtime dogfood incomplete | lifo pilot |
| `skills/taskmaster-refining/README.md` | skill metadata | Public classification, canonical/promotion contract, forbidden content | Reviewed with skill | Non-task metadata | lifo pilot |

## Known Reconciliation Decisions

1. `master/20` and `cmux-parity-20260707/4` overlap. The cmux-tag task owns
   PRD-D and implementation evidence; the master task must become an explicit
   duplicate pointer rather than remain a second pending implementation lane.
2. `product-hygiene/1` is not automatically done merely because PR #56 merged;
   its release/preview acceptance evidence and subtask `1.1` must be reviewed.
3. `cmux-parity-20260707/6` remains F1 only. The decision document is still a
   provisional skeleton; no F2 implementation task may be generated until its
   evidence and ratification gate pass.
4. `master/23` is already sufficiently decomposed through `23.11`; complexity
   analysis may score it, but expansion must not duplicate the frozen ladder.
5. Completed historical documents do not need synthetic active tasks. They
   must retain explicit PR/task evidence in this map and in the related parent
   task where useful.
6. Peer-owned handoffs are not rewritten for parity. Current authority is the
   task store, this map, source PRDs, merged Git evidence, and the owned Lifo
   handoff refreshed at closeout.

## Active Task To Source Index

This is the inverse of the source tables above. It gives every non-done parent
an exact authoritative source set and post-repair disposition.

| Active task(s) | Authoritative source set | Post-repair disposition |
|---|---|---|
| `master/7,8,10,11,12,13` | `.taskmaster/docs/limux-cursor-ide-integration-prd-20260630.md`; `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`; `docs/reviews/limux-cursor-ide-integration-20260630/` | Preserve graph; expand 7, 8, 10, 11, 12 selectively; finish 13's existing boundary subtask before residual expansion |
| `master/15` | `docs/future-improvements/limux-runtime-isolation-and-window-ui-plan-20260701.md`; PR #33 evidence | Repair string ID; retain review until live edge/control acceptance |
| `master/16,17` | `docs/future-improvements/limux-runtime-isolation-and-window-ui-plan-20260701.md`; `.taskmaster/docs/workspaces-sidebar-notifications-20260620.md` for task 17's inherited sidebar behavior | Add dependency on 15; add test strategy; expand design/research gates |
| `master/20` | `.taskmaster/docs/limux-prd-d-pane-attention-20260706.md`; `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md` | Defer as duplicate pointer to canonical cmux task 4; do not expand |
| `master/23` | `.taskmaster/docs/limux-prd-i-hcom-visibility-restart-integration-20260713.md`; `docs/future-improvements/hcom-limux-session-pane-visibility-restart-design-20260712.md`; `LIFO_INBOX/TASK_FROM_kuma_2026-07-12_visibility-restart-design-commission.md` | Preserve 23.1-23.11; add parent test strategy from PRD-I verification matrix |
| `master/24` | `docs/future-improvements/limux-global-use-guide-and-hcom-routing-20260707.md`; `skills/limux-a2a/SKILL.md`; `skills/limux-use-guide/SKILL.md` | Add test strategy; expand exact identity/elevation/restoration proof only after complexity review |
| `master/25` | `docs/future-improvements/limux-global-use-guide-and-hcom-routing-20260707.md`; `skills/limux-use-guide/SKILL.md`; `skills/limux-a2a/SKILL.md`; hcom events `434032`, `434092`, `434101` | Add test strategy; expand parser, byte transport, integration fixtures, and first-use docs |
| `cmux-parity-20260707/3` | `.taskmaster/docs/limux-prd-c-verify-loop-20260706.md`; `docs/verification/post-install-checklist-v1.md`; `docs/verification/run-template.md` | Expand only live-run/write-back residuals; retain in progress |
| `cmux-parity-20260707/4` | `.taskmaster/docs/limux-prd-d-pane-attention-20260706.md`; `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md`; PR #22/#57 evidence | Retain review; expand only residual layering/clear-mode live proof |
| `cmux-parity-20260707/5` | `.taskmaster/docs/limux-prd-e-bridge-parity-20260706.md`; `docs/cmux-parity-plan.md` | Preserve existing subtask; expand remaining registry/mirror/mutation routes after complexity review |
| `cmux-parity-20260707/6` | `.taskmaster/docs/limux-prd-f-browser-live-20260706.md`; `docs/decisions/browser-pane-architecture-20260707.md` | Remain F1 only; no F2 promotion until decision evidence and task 5 gate pass |
| `cmux-parity-20260707/7` | `.taskmaster/docs/limux-prd-g-agent-sidebar-20260706.md`; `docs/future-improvements/limux-lifecycle-events-and-agent-team-staleness-20260707.md`; `hooks/README.md` | Preserve existing subtasks; expand remaining agent-family/UI/scale proof |
| `product-hygiene/1` | `.taskmaster/docs/product-hygiene-version-and-tab-rename-20260707.md`; `CHANGELOG.md`; PR #56 evidence | Retain in progress until merged-main preview checklist/promotion evidence |
| `taskmaster-refinement-20260713/2-6` | `.taskmaster/docs/limux-taskmaster-refinement-prd-20260713.md`; `.taskmaster/docs/TASKMASTER_REFINEMENT_ACTION_PLAN.md`; `skills/taskmaster-refining/SKILL.md`; continuation PRD for tasks 3-4 | Apply the self-reconciliation table after supported runtime repair; task 5 string ID remains parked |

## TaskMaster Runtime Divergences

1. The reviewed AI front door selected provider `ollama` and model `glm-5.2`
   for the initial parse, despite the manager guidance expecting the current
   fresh-init MiniMax default. No credential values were read or exposed. Sage
   traced this to per-project config drift; the supported model commands have
   now restored main `minimax/MiniMax-M3` with Ollama GLM 5.2 as research and
   fallback. Current config hash:
   `844313ec907fc9e6c3d8cc32aae2e822b312c3ec70fa5b3299fa4007e6dc768b`.
2. The provider emitted malformed JSON that the reviewed wrapper repaired. It
   also tried to change generated task IDs during two updates; the wrapper
   restored those IDs. Task `5` remains serialized as string `"5"`, while
   dependency validation still resolves all six tasks successfully.
3. Provider-generated command examples used unreviewed/raw TaskMaster command
   forms. Tasks `4` and `5` were corrected through the reviewed AI front door,
   and exact provenance was appended through non-AI `append-note` operations.
4. `set-status` is current-tag scoped and does not accept `--tag`; this pass
   explicitly switches tags with `task-master-reviewed tags use <tag>` before
   status changes and restores the intended tag after each bounded lane.
5. A subsequent `master/15` AI update capture shows the provider schema rejecting
   valid status `review` and strict `updatedAt`, then truncates while fallback is
   still active. The store later contains provider-derived details, test
   strategy, five subtasks, and string ID `"15"`. The terminal result is
   ambiguous, not a proven write-after-failure. Task `master/15` is parked with
   refinement task `5`; exact evidence is in the dedicated report.
6. Sage froze the source-triage incident at
   `/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/INCIDENT_FROM_sage_20260713_limux_refinement_runtime_divergences.md`,
   authoritative SHA-256
   `52d437cfc94e1d9d6c56fe03f22ebe591c4a8feaaeb9e55c21ec0fe17bd0a2ff`.
   It is the canonical repair contract for the CLI-string ID, validator blind
   spot, bidirectionally inconsistent parent-status sets, and ambiguous terminal
   outcome observability defect. It also records the clean-lane launch root
   cause and temporary no-TaskMaster-mutation tracking exception caused by the
   confirmed tags-add defect. It does not classify the truncated command as a
   proven transaction failure.
7. Sage approved the project-local `taskmaster-refining` candidate at exact
   SHA-256
   `8eb3c2ede88a7c1021cc97d06b89db4895730b3883748650bd9a1937169f0b56`.
   Promotion remains blocked until a repaired reviewed runtime passes the full
   G0 invariants.
8. The numeric-parent-ID source repair is frozen as an apply-clean patch at
   `/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/PATCH_FROM_Banach_20260713_update_task_numeric_parent_id.patch`,
   SHA-256
   `28ef4a65b8bc019081cfea2cf6a46c4354f1266b50cfaca700af3bd5e445ca5b`.
   Independent evidence is at
   `/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/EVIDENCE_FROM_sage_20260713_update_task_numeric_id_patch.md`,
   SHA-256
   `78670858b3c0c7bacd089f2f45a92926e72e1487be351dc77c61c3fb238a4ec2`.
   Static/Node checks pass, but focused Jest lacks a reviewed dependency
   runtime. The patch is not integrated, pushed, installed, or sufficient to
   clear the Limux mutation hold.
9. Supplemental source analysis is frozen at
   `/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/EVIDENCE_FROM_sage_20260713_update_task_schema_fallback_status_analysis.md`,
   SHA-256
   `78ec8eeb1b53604e7fca9edfd3e1c12ef7a0757aa52dda644a1594d592d1e92e`.
   It confirms later-role success/write after earlier schema failures, canonical
   seven-status drift, `updatedAt`/full-task projection conflict, malformed-JSON
   repair without schema revalidation, and final text-fallback provider-order
   suppression. The missing tests/source tasks remain part of the repair gate;
   this evidence does not authorize Limux TaskMaster writes.
10. The current source-worker execution contract is
   `/home/riche/MCPs/claude-task-master/TASKMASTER_MGR_INBOX/BRIEF_TO_tmidfix_20260713_update_task_parent_id_type.md`,
   SHA-256
   `1bda528132727bfa6a253b99a477b9706eb9dc570d8eb5da24a06969c4c850d1`.
   It remains a TaskMaster source-lane artifact, not authorization for Limux
   task-store repair or mutation.

## Pending Outputs

- Parse companion cmux continuation PRD after TaskMaster source repair.
- Source/detail/test-strategy reconciliation notes for active parents.
- Apply frozen action-plan mutations after TaskMaster source repair.
- Tag-specific complexity reports and hashes.
- Expansion evidence: aggregate `failedCount == 0`/expected `expandedCount` for
  `expand --all` when exposed, or parent-specific before/after hash and subtask
  inspection for targeted `expand --id`.
- Final dependency/order table and `next --json` evidence per affected tag.
- TaskMaster owner review and later global-config promotion of skill pilot.
- Final task-store hash, changed paths, PR, and exact-head review evidence.
