# Fresh Limux Successor Prompt

Paste the content below into a fresh Codex session opened at `/home/riche/MCPs/limux`.

---

You are the fresh Limux manager and sole writer for `/home/riche/MCPs/limux`. The predecessor `gula` session is read-only/standby. Do not resume or reclaim the `gula` hcom identity; use the fresh identity provided to your session.

Mission: take over all current Limux work without losing peer-owned state. Your first implementation lane is the storage anti-refill request: establish a shared Cargo target/cache strategy and a reviewed retention/build-wave mechanism that prevents repeated Limux build artifacts from refilling constrained storage. Continue all non-gated work to a verified branch/PR outcome. Do not clean existing artifacts, install/restart Limux, activate the renderer policy, or patch vendored Ghostty without the applicable authorization.

Definition of done for intake:

1. Read `/home/riche/MCPs/limux/AGENTS.md`, `/home/riche/MCPs/limux/GULA_HANDOFF.md`, and this prompt completely.
2. Load `$prime`, `$hcom`, `$dirty-worktree-owner-cleanup`, `$taskmaster-reviewed`, `$tdd`, and `$methodical-modification-protocol` before their matching actions.
3. Verify Git branch/head/remote, the full dirty-path count, worktree inventory, open GitHub PRs, hcom manager state, and TaskMaster state. Do not infer current state from the predecessor summary when a read-only check can decide it.
4. Post a concise commentary acknowledgement containing: your hcom name, current branch and HEAD, `origin/main`, dirty-entry count grouped by top-level path, confirmation that tracked/staged dirt is absent or its exact exception, the first task you will execute, and the branch you will create.
5. Send one short hcom acknowledgement to `@gula` pointing to your own durable handoff/branch. Do not depend on live delivery; inbox storage is acceptable.
6. Create a fresh successor-owned implementation branch from the current `origin/main` in the primary checkout before editing. Do not create a worktree unless concurrent writers genuinely require one; if that happens, the only sanctioned root is this repository's ignored `.worktrees/<owner-topic>/` and it must use the shared Cargo cache.
7. Create your own `<YOUR_HCOM_NAME>_HANDOFF.md` on your branch at the first real milestone. Treat `GULA_HANDOFF.md` and `GULA_SUCCESSOR_PROMPT.md` as predecessor-owned, read-only records.

Known gates and boundaries:

- Preserve every pre-existing untracked file. The predecessor snapshot counted 1,835 entries: 1,831 beneath `GULA_EVIDENCE/`, `AUTOPILOT_LOG.md`, one `LIMU_INBOX/` alert, and two `docs/research/` files. Recount before relying on this number.
- Never use `git add .`, blanket restore/reset/clean, delete APIs, or `/tmp` for load-bearing work.
- Six historical `/tmp` worktree records point to already-missing directories. Inventory only; do not prune or recreate them during intake.
- `origin/main` contains merged PR #136 at `204a3b6eb2cf955373f26df5e1d04a644fd0ccb7`. Fetch and verify before branching.
- TaskMaster tag `limux-resource-crash-20260716` has task 4 in progress and task 7 blocked. Do not repurpose either for anti-refill. Inspect the store through `task-master-reviewed`; create or route a dedicated task through the reviewed workflow before multi-step implementation. Never edit `.taskmaster/tasks/tasks.json` manually.
- Renderer task 7 remains blocked until an owned/upstream Ghostty C API can remove Limux-injected environment variables from terminal children. Keep that separate from the storage anti-refill lane.
- The installed daily-driver resource problem is unresolved. Source work is authorized; live installation, restart, promotion, cleanup, destructive retention, and renderer activation are not authorized by this prompt.
- Coordinate the shared Cargo design with the current hcom source owner (`zori` was the prior owner observed by `gula`) and route the incident result to `nafo`/`momo` using owner-first hcom plus durable file pointers. Re-resolve live owners first; names may have changed.

Anti-refill success criteria:

- Re-measure current repository `target/` allocation and relevant build-process evidence with tool path, exact version, command, workload parameters, timestamp, source SHA, dirty state, and build profile recorded.
- Establish one shared Cargo target/cache mechanism for the primary checkout and sanctioned concurrent worktrees without creating independent heavy per-worktree compiler-output trees.
- Establish an evidence-derived, reviewed retention boundary and build-wave discipline for constrained windows. Do not invent thresholds; derive them from measured evidence or obtain an operator decision.
- Add tests before implementation where behavior is code-controlled. Run the narrowest relevant checks during iteration and the repository's proportionate quality gate before claiming completion.
- Commit and push exact owned paths on a successor-owned branch, open/update the PR when ready, independently verify remote bytes, and leave the live runtime unchanged unless separately authorized.

Begin by performing the read-only intake checks and posting the acknowledgement as a commentary update; then continue without waiting unless you reach a genuine gate.

---
