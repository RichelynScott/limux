# Limux Limu Manager Handoff

**Owner:** `limu` (`LIMUX_MGR`)  
**Updated:** 2026-07-21 12:05 EDT  
**Hcom session:** `019f851d-2892-7083-b72a-8fded7473c3d`  
**Manager claim:** `mgr-322175f796efe952`  
**Scope:** `/home/riche/MCPs/limux`

## Read This First

This is the active manager-owned resume surface. Historical handoffs remain
preserved under their original names and should be read only for lane history.
The root `HANDOFF.md` is Halo-owned and stale as of 2026-06-20; it was not
rewritten during succession.

## Current Status

- Active manager branch: `limu/repo-reconciliation-20260721`, based on
  `origin/main` at `31a9431`.
- Stable runtime: Limux `0.2.3`, source `1a26bda0`, stable channel. Live
  `doctor --json` returned `ok=true` during succession intake.
- Mainline verification on 2026-07-21: `./scripts/check.sh` passed with 597
  tests, clippy with warnings denied, and formatting; split-icon and Ghostty
  resource validators also passed.
- Repository audit:
  `docs/REPO_AUDIT_limux_2026-07-21.md`.
- Succession evidence:
  `LIMUX_SUCCESSION_ONBOARDING_FROM_tutu_2026-07-21.md`.
- Active manager inbox: `LIMU_INBOX/`.

## Protected Inherited Work

Lifo's uncommitted TaskMaster master #29 RED-state tests were preserved without
loss on `limu/pane-reflow-task29-20260721`:

- Commit: `7e0eb0756ad6471387479b2e71ee562017d5e9bc`
- Remote: `origin/limu/pane-reflow-task29-20260721`
- Known state: focused compile fails only because
  `surface_resize_should_apply_during_interaction` has not yet been
  implemented. Do not merge until the tests turn green and live resize evidence
  exists.

## TaskMaster

- Active tag remains `limux-resource-crash-20260716`.
- `master` task 31: **in-progress** — manager succession and repository hygiene.
- `master` task 29: **in-progress** — terminal word-wrap/reflow.
- `master` task 32: **pending / HIGH** — OMP terminal scroll-yank and periodic
  flash. The header/scrollbar interaction is a hypothesis, not a proven root
  cause; Git history shows scrollbar commit `fc23ac2` predates local header PR
  #59.
- Resource-crash tasks 2 and 3 are in `review`, but their PRs remain
  conflicting and retain exact-head P2 findings. Task 4 is `in-progress` and
  still lacks its operator-gated isolated-preview runtime evidence.

## Open PR Disposition

| PR | State | Manager disposition |
|---|---|---|
| #58 | Conflicting mixed TaskMaster/handoff/docs branch | Port only still-valid handoff/attestation content, then close as superseded. |
| #67 | Conflicting renderer diagnostics branch; exact-head P2 remains | Rebuild from current main, fix the restrictive socket-mode finding, reverify. Do not merge old branch. |
| #68 | Conflicting bounded logging branch; exact-head P2 remains | Rebuild from current main, fix the stderr-fd finding, reverify. Do not merge old branch. |

## Worktrees and Disk

- Primary checkout: `/home/riche/MCPs/limux`.
- Retained release worktree:
  `/tmp/limux-release-0.2.3-20260719`. It is clean and its head is an ancestor
  of main, but Hamo's no-loss hold remains. Do not remove without explicit
  operator release.
- Checkout size during audit: approximately 18 GiB, including 16 GiB ignored
  `target/`. Normal deletion is not authorized by the archive-not-delete rule;
  `cargo clean` or equivalent requires an explicit regenerable-artifact
  exception.
- No local or remote branches were deleted during succession.

## Inbox

- `BUG_FROM_tutu_2026-07-21_omp-pane-scroll-yank-flash.md`: TaskMaster master
  #32; reproduce before accepting the proposed cause.
- `INCIDENT_FROM_reve_2026-07-19_new-pane-terminal-never-initializes.md`: legacy
  0.2.2 evidence; requires a bounded v0.2.3 retest before claiming current
  impact.
- `DESIGN_QUESTION_FROM_nava_2026-07-21_hcom-tui-limux-symbiosis.md`:
  exploratory; pane colors and focus primitives already exist. Defer product
  commitment until the hcom TUI sequence clears.
- `docs/LIMUX_RUNTIME_CLOSEOUT_DECISION_PACKET_2026-07-16_LIFO.html` remains an
  untracked operator/historical artifact. It was not modified, staged, renamed,
  or archived.

## Historical Handoff Index

| File | Use |
|---|---|
| `HAMO_HANDOFF.md` | v0.2.3 release and force-restart detail; current release provenance. |
| `LIFO_HANDOFF.md` | Deep Lifo implementation/session history. |
| `LIFO_CL_MGR_HANDOFF.md` | Earlier manager-lane history. |
| `NATO_HANDOFF.md` | cmux/parity planning history. |
| `HALO_HANDOFF.md` and `HANDOFF.md` | June 2026 history; not current runtime truth. |
| `FYI.md` | Append-only journal; archive-first condensation is still pending. |

## Immediate Next Actions

1. Commit and push the reconciliation bundle: TaskMaster #31/#32, this handoff,
   `LIMU_INBOX/`, Tutu onboarding, and the repo audit. Keep the protected Lifo
   HTML packet unstaged.
2. Request operator decisions on porting PR #67/#68, releasing the retained
   worktree hold, and allowing deletion of regenerable `target/` artifacts.
3. Prioritize the operator-facing Task 32 scroll-yank reproduction against the
   existing Task 29 RED implementation lane.
4. Keep root `HANDOFF.md` unchanged until canonical pointer ownership is
   explicitly transferred.

