# TaskMaster tag reconciliation - 2026-07-07

## Finding

The `cmux-parity-20260707` TaskMaster tag is present and populated on clean
`origin/main` at `b26312715162`. It contains eight tasks, with PRD-H tracked as
task `8`.

The earlier "empty tag" symptom came from checking TaskMaster from the dirty
primary checkout on the spent `lifo/hermes-workspace-highlight-resize-20260627`
branch. In that checkout, `.taskmaster/` appears as untracked relative to the
current branch and can diverge from the clean main-backed store.

## Reconciliation Rule

- Use a clean worktree based on `origin/main` for `cmux-parity-20260707`
  TaskMaster inspection or mutation.
- Do not use the dirty primary checkout as the source of truth for this tag.
- Do not attach restore-lane work to the `master` tag task `8`; that is the
  Cursor workspace-selection task in a different tag.
- Restore-lane work belongs under `cmux-parity-20260707` task `8`
  (`Implement Session Restore Correctness Pack`).

## 2026-07-07 Update

The approved P1 restore decomposition was added under PRD-H task `8`:

- `8.1` - exited-agent restore safety and stale socket cleanup
- `8.2` - hcom-managed agent resume path
- `8.3` - staggered restore batching

These subtasks implement the approved three-PR shape for the remaining restore
lane and avoid double-building a separate restore task outside PRD-H.
