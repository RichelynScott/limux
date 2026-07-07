# Limux Verification Run Template

Copy this file to `docs/verification/runs/<YYYYMMDD>-<install-id>.md` for each
post-install verification run. Do not edit historical run files except to add a
clearly dated correction note.

## Header

| Field | Value |
|---|---|
| Run file | `docs/verification/runs/<YYYYMMDD>-<install-id>.md` |
| Checklist version | `v1` |
| Checklist git SHA | `<git rev-parse HEAD:docs/verification/post-install-checklist-v1.md>` |
| Source SHA under test | `<git rev-parse HEAD>` |
| Install id | `<install-id>` |
| Channel | `preview:default` / `stable` |
| Launcher used | `~/.local/bin/limux-preview` |
| `<launcher used> --version` output | `<paste output from the launcher named above>` |
| Date/time started | `<YYYY-MM-DD HH:MM TZ>` |
| Date/time completed | `<YYYY-MM-DD HH:MM TZ>` |
| Operator | `<name>` |
| Overall verdict | `PASS` / `FAIL` |
| Promotion decision | `PROMOTED` / `BLOCKED` / `NOT APPLICABLE` |

## Command Evidence

```bash
git rev-parse --verify HEAD
~/.local/bin/limux-preview --version
~/.local/bin/limux-preview doctor --json
bash scripts/tests/runtime-isolation-smoke.sh
```

## Verdict Table

| # | Item | Verdict | Evidence pointer |
|---|---|---|---|
| 1 | Build identity and doctor | `PASS` / `FAIL` / `N/A` |  |
| 2 | Fresh pane typing, modifier chords, and paste | `PASS` / `FAIL` / `N/A` |  |
| 3 | Mouse selection copy and stuck-click watch | `PASS` / `FAIL` / `N/A` |  |
| 4 | Window controls and edge hitbox | `PASS` / `FAIL` / `N/A` |  |
| 5 | Drag-resize soak with live agent TUI | `PASS` / `FAIL` / `N/A` |  |
| 6 | Sidebar resize, collapse, and restore | `PASS` / `FAIL` / `N/A` |  |
| 7 | Multi-workspace session restore | `PASS` / `FAIL` / `N/A` |  |
| 8 | Notification toast and sidebar dot | `PASS` / `FAIL` / `N/A` |  |
| 9 | Runtime channel isolation | `PASS` / `FAIL` / `N/A` |  |
| 10 | Pane attention overlay and per-pane flags | `PASS` / `FAIL` / `N/A` |  |

## Findings

### Passed Items

- `<item>`: `<evidence>`

### Failed Items

- `<item>`: `<symptom, reproduction, evidence path, task created/reopened>`

### Not Applicable Items

- `<item>`: `<why>`

## TaskMaster Write-Back

Record the exact reviewed TaskMaster commands run after this verification:

```bash
task-master-reviewed set-status --id=<id> --status=<status>
task-master-reviewed update-task --id=<id> --prompt="<evidence note>"
task-master-reviewed add-task --prompt="<new failing behavior and evidence>"
```

## Promotion

Promotion is allowed only when the run is a full run and every checklist item is
`PASS`.

If promoted, record:

```bash
scripts/user-local-install/install-user-local.sh --apply --profile release --channel stable --install-id <verified-sha-id>
~/.local/bin/limux-stable --version
```

Stable relaunch confirmation:

- Operator closed preview or left it isolated.
- Operator launched `~/.local/bin/limux-stable` or `Limux Stable`.
- `limux-stable --version` matches the verified source SHA and install id.

If blocked, record the blocking item numbers, evidence paths, and task IDs.
