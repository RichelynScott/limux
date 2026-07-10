# Command Surface

Run all hcom commands with the current session's explicit `--name`.

## Snapshot

```bash
git status --short --branch
git diff --name-status
git worktree list --porcelain
hcom list -v --json --name <self>
hcom list mgrs --json --name <self>
hcom mgr show <project> --json --name <self>
limux identify --json
limux --json list-panes --workspace <workspace-ref>
limux --json list-panels --workspace <workspace-ref>
limux --json surface-health --workspace <workspace-ref>
```

Inspect each worktree with `git -C <path> status --short --branch`. Resolve PR
state with `gh pr list --state all --head <branch>`.

## Safe Resume

```bash
limux read-screen --workspace <workspace-ref> --surface <surface-ref> --lines 30
limux send --workspace <workspace-ref> --surface <surface-ref> \
  'hcom r <verified-session-or-name> --run-here --go'
limux send-key --workspace <workspace-ref> --surface <surface-ref> Enter
hcom list -v --json --name <self>
```

If the stored UUID is absent, use lifecycle events and transcript search to
recover the real transcript. Verify the file and cwd before one retry.

These are command shapes, not permission grants. Composed skills and project
instructions govern mutation and worktree removal.
