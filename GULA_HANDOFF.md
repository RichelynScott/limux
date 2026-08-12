# Limux Gula Manager Handoff

Checkpoint date: 2026-08-12
Owner: `gula`
HCOM session: `019ff3bd-7989-7fc1-aaef-c27e86de32d3`
Scope: all Limux work, per operator confirmation relayed by `nafo` in hcom request `#165`
Checkpoint branch: `gula/checkpoint-anti-refill-20260812`

## Succession State

The operator requested a fresh session takeover. Use `GULA_SUCCESSOR_PROMPT.md` as the copy-ready prompt. This file remains the predecessor record; the successor must create its own `<SUCCESSOR_NAME>_HANDOFF.md` after assuming ownership.

- `gula` is now read-only/standby. The successor owns all new Limux writes after it posts the required intake acknowledgement.
- Do not resume or reclaim the `gula` identity. Initial native-resume preflight showed a missing hcom session/transcript binding; the later metadata refresh restored session `019ff3bd-7989-7fc1-aaef-c27e86de32d3`, transcript and resume availability, process binding, and live delivery. Use a fresh identity to prevent collision even though `gula` is now recoverable.
- This is an orderly single-writer cutover. The successor may use the primary checkout and must create a fresh owned implementation branch from `origin/main` before editing. A new worktree is unnecessary unless two sessions later need to write concurrently.
- The checkpoint branch contains predecessor-only handoff commits on top of `origin/main`. No PR exists for the checkpoint branch.

## Immediate Next Action

Resume the anti-refill lane before any other new Limux work:

1. Fetch `origin/main`, reconcile this checkpoint branch, and create a fresh owned implementation branch from the current base in the primary checkout. Do not create a worktree and do not use `/tmp`.
2. Re-measure the current Limux `target/` allocation and build-process history before treating the incident numbers below as current facts.
3. Inspect the existing Limux build/install/retention mechanisms and coordinate with the global fleet-policy owner and `zori`'s hcom shared-Cargo work before selecting a mechanism.
4. Implement one shared Cargo target strategy (`CARGO_TARGET_DIR` and/or `sccache`) that works for the primary checkout and any explicitly approved exceptional worktree.
5. Define and implement a reviewed retention boundary for incremental build artifacts plus build-wave discipline for constrained disk windows. Do not delete or destructively clean artifacts; use the archive/no-loss and operator gates.
6. Verify the mechanism with measured disk/build evidence, then send `nafo` a short hcom result pointing to the durable artifact.

The incident figures in `nafo` request `#165` are claims from that message, not revalidated by `gula` during this checkpoint: Limux `target/` at about 4.3 GiB, 42 Cargo calls before the crash, about 1.5 GiB attributed to the incident window, and ext4 refill from 199 GiB to 305 GiB in about two weeks.

## Completed Before This Checkpoint

- PR `#136` merged to `main` at `204a3b6eb2cf955373f26df5e1d04a644fd0ccb7`.
- The merged change contains the fail-closed renderer probe supervisor and the Orca/Limux repository evaluation.
- Renderer implementation review passed after ownership/reaping fixes. Focused renderer tests passed: 10 CLI tests and 1 host test.
- Post-merge broader verification passed: 165 CLI unit tests, 5 launcher-route tests, and 496 host tests. Scoped strict clippy and direct rustfmt checks for the new renderer files passed. Repository-wide formatting/clippy still has unrelated pre-existing drift.
- The Orca decision is: keep Limux and selectively extract useful designs. Do not switch wholesale unless cross-platform, mobile, or SSH becomes the controlling product requirement. The evaluation pinned Orca commit `09ec516ae50b7b83fa65343d9ad96159e3fe71fc`.
- The branch was aligned with `origin/main` and GitHub reported no open PRs immediately before this checkpoint.

## Current Repository And Task State

- Product base: `origin/main` at `204a3b6eb2cf955373f26df5e1d04a644fd0ccb7`.
- Pre-succession checkpoint: `911000d12f31f3d35224c27f6be15512601b976d` on `origin/gula/checkpoint-anti-refill-20260812`.
- GitHub intake on 2026-08-12: no open PRs and no PR for the checkpoint branch.
- TaskMaster tag `limux-resource-crash-20260716`: tasks 1-3 done, task 4 in progress, tasks 5-6 pending, task 7 blocked. Task 4 is existing work and must not be repurposed for the anti-refill request.
- The anti-refill request is not represented by a dedicated task in that tag. The successor must inspect the current store and use the reviewed TaskMaster workflow to create or route an appropriate task before multi-step implementation; do not invent an ID or hand-edit `tasks.json`.
- Installed TaskMaster front doors `task-master-reviewed`, `task-master`, and `taskmaster` hash-match at `a9fb11ffa6e4a0e560bf4996ccb11292df53d5c861732f281939e342b58c962f`. Doctor reports reviewed source `0768c2ae0c429277209f35a5aa9652f26f71a850`.
- Six historical `/tmp` worktree records are prunable administrative residue whose checkout directories are already missing. Do not prune or recreate them as part of succession. Current policy forbids new `/tmp` worktrees.

## Renderer Blocker

Renderer auto-selection remains deliberately inactive because `child_env_removal_supported()` is false. Activation is blocked until an owned/upstream Ghostty C API can remove Limux-injected renderer variables from terminal child environments. Vendored `ghostty/` is read-only from the Limux layer.

- TaskMaster tag: `limux-resource-crash-20260716`
- Task: `#7`, blocked
- Primary source checkpoint: `docs/verification/renderer-probe-supervisor-20260812.md`
- Matched evidence: `GULA_EVIDENCE/2026-08-12/renderer-probe-matched-r1/summary.json`
- Final review: `GULA_EVIDENCE/2026-08-12/RENDERER_FINAL_REVIEW_SUBAGENT.md`
- Ghostty seam report: `GULA_EVIDENCE/2026-08-12/RENDERER_OWNED_ENV_SEAM_SUBAGENT.md`

No daily-driver install, restart, promotion, or live renderer activation occurred. The installed Limux resource drain therefore remains unresolved in the live runtime.

## Preserved Untracked And Peer-Owned State

Do not stage, archive, rewrite, or clean the existing untracked material without exact ownership review.

At the 2026-08-12 succession snapshot, `git status --porcelain=v1 -uall` reported 1,835 untracked entries: 1,831 under `GULA_EVIDENCE/`, one `AUTOPILOT_LOG.md`, one `LIMU_INBOX/` alert, and two `docs/research/` files. There were no tracked or staged changes before the succession documents were authored.

Peer-owned source files left untouched:

- `AUTOPILOT_LOG.md`
- `LIMU_INBOX/ALERT_FROM_bora_2026-07-31_omp-shell-keeps-dying.md`
- `docs/research/LIMUX_CONTROL_SCOUT_2026-07-31.md`
- `docs/research/T3CODE_MOBILE_SCOUT_2026-07-31.md`

A subagent-only inventory of those files is preserved at `GULA_EVIDENCE/2026-08-12/PEER_OWNED_FILES_INVENTORY.md`, SHA-256 `27d6481cfa47acd1cfacd9129098236a12e4526b1eb48b0fcf8a7d6a83bb6dd7`. It remains untracked because this is a public repository and the report identifies private coordination/research content.

Numerous older `GULA_EVIDENCE/2026-08-12/` captures and `renderer-supervisor-orca-pr-body.md` also remain untracked. Preserve them in place pending an exact owner/publication review. In particular, do not treat generated-looking runtime directories as authorized cleanup targets.

## HCOM And Delivery State

- The initial native-resume preflight temporarily showed `session_id=null` and no transcript/resume availability. Refreshing the owned hcom metadata restored session `019ff3bd-7989-7fc1-aaef-c27e86de32d3`; the final row reports process binding, transcript/resume availability, hook delivery, and live delivery. The successor still uses a fresh identity for collision prevention.
- The acknowledgement to `nafo` request `#165` was stored in the inbox because `nafo` was not live; it was not a live wakeup.
- `momo` was unavailable and had no live/resumable hcom identity when the resource findings were delivered. The durable renderer evidence and merged source are the recovery record.

## Key Files For The Successor

- `/home/riche/MCPs/limux/AGENTS.md` — active project contributor rules.
- `/home/riche/MCPs/limux/GULA_SUCCESSOR_PROMPT.md` — copy-ready takeover prompt and acknowledgement contract.
- `/home/riche/MCPs/limux/GULA_HANDOFF.md` — this zero-context predecessor record.
- `/home/riche/MCPs/limux/docs/verification/renderer-probe-supervisor-20260812.md` — renderer supervisor implementation and activation boundary.
- `/home/riche/MCPs/limux/docs/REPO_AUDIT_ORCA_2026-08-12.md` — Orca/Limux evaluation and selective-extraction recommendation.
- `/home/riche/MCPs/limux/GULA_EVIDENCE/2026-08-12/PEER_OWNED_FILES_INVENTORY.md` — private, untracked inventory; read only and never commit to the public repository.

## Critical Successor Rules

- Use a fresh hcom identity; do not reclaim `gula`.
- Treat this session as read-only after succession. The successor owns the primary checkout and all new writes.
- Preserve all 1,835 pre-existing untracked entries. Exact-stage successor-owned paths only; never use `git add .`.
- Do not use `/tmp` for worktrees, evidence, handoffs, or delegated outputs.
- Do not prune historical worktree records, clean build artifacts, install/restart the live runtime, activate renderer policy, or patch vendored Ghostty without the applicable authorization.
- Keep the renderer/Ghostty blocker separate from anti-refill storage work.

## Resume Checks

```bash
cd /home/riche/MCPs/limux
git fetch origin
git status --short --branch
git rev-parse HEAD origin/main
task-master-reviewed show 7 --tag limux-resource-crash-20260716
hcom list -v --name gula
```

Create the anti-refill implementation branch only after these checks and the successor acknowledgement. Preserve all unrelated dirt, exact-stage owned paths only, and do not install/restart the live runtime without the normal gate.
