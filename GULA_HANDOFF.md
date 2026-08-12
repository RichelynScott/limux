# Limux Gula Manager Handoff

Checkpoint date: 2026-08-12
Owner: `gula`
HCOM session: `019ff3bd-7989-7fc1-aaef-c27e86de32d3`
Scope: all Limux work, per operator confirmation relayed by `nafo` in hcom request `#165`
Checkpoint branch: `gula/checkpoint-anti-refill-20260812`

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

Peer-owned source files left untouched:

- `AUTOPILOT_LOG.md`
- `LIMU_INBOX/ALERT_FROM_bora_2026-07-31_omp-shell-keeps-dying.md`
- `docs/research/LIMUX_CONTROL_SCOUT_2026-07-31.md`
- `docs/research/T3CODE_MOBILE_SCOUT_2026-07-31.md`

A subagent-only inventory of those files is preserved at `GULA_EVIDENCE/2026-08-12/PEER_OWNED_FILES_INVENTORY.md`, SHA-256 `27d6481cfa47acd1cfacd9129098236a12e4526b1eb48b0fcf8a7d6a83bb6dd7`. It remains untracked because this is a public repository and the report identifies private coordination/research content.

Numerous older `GULA_EVIDENCE/2026-08-12/` captures and `renderer-supervisor-orca-pr-body.md` also remain untracked. Preserve them in place pending an exact owner/publication review. In particular, do not treat generated-looking runtime directories as authorized cleanup targets.

## HCOM And Delivery State

- `gula` was rebound after the prior crash under session `019ff3bd-7989-7fc1-aaef-c27e86de32d3`.
- The acknowledgement to `nafo` request `#165` was stored in the inbox because `nafo` was not live; it was not a live wakeup.
- `momo` was unavailable and had no live/resumable hcom identity when the resource findings were delivered. The durable renderer evidence and merged source are the recovery record.

## Resume Checks

```bash
cd /home/riche/MCPs/limux
git fetch origin
git status --short --branch
git rev-parse HEAD origin/main
task-master-reviewed show 7 --tag limux-resource-crash-20260716
hcom list -v --name gula
```

Create the anti-refill implementation branch only after these checks. Preserve all unrelated dirt, exact-stage owned paths only, and do not install/restart the live runtime without the normal gate.
