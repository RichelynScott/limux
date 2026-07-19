# Changelog

All notable Limux changes should be recorded here when a PR merges.

## [0.2.3] - 2026-07-19

### Added

- Added durable, atomic runtime lifecycle markers so clean and unclean host
  exits can be distinguished without trusting a partially written state file.
- Added isolated visibility controls and renderer counters used to verify
  hidden-workspace resource behavior without changing the daily-driver host.

### Changed

- Hidden, minimized, and unmapped terminal surfaces are now marked occluded and
  use a reduced fallback renderer cadence while terminal child processes stay
  alive.
- Restored agents are suspended after an unclean host exit instead of being
  started automatically, preventing crash-recovery session storms.
- Live header sections use clearer spacing without changing their configured
  order or visibility.

### Fixed

- #70 copies selected terminal text on Ctrl+C while preserving the normal PTY
  interrupt when no selection exists, including matched repeat/release events.
- #71 gives command-launching `pane.create` requests a method-specific response
  deadline and returns conservative retry-safety metadata when the outcome may
  be ambiguous, preventing blind duplicate-pane retries.
- Runtime lifecycle commits are serialized and written atomically so concurrent
  shutdown paths cannot corrupt recovery state.
- Native window mapping is tracked when deciding renderer visibility, including
  minimized-window and hidden-workspace transitions.
- Host builds now fail immediately when the Ghostty GLAD sources are missing
  instead of producing a binary that fails later with an unresolved GL loader
  symbol.

### Release Notes

- This release reconciles the previously installed runtime aggregate at
  `1005f58d92a1` with the versioned source release. Native OMP runtime identity
  remains a separately owned OMP/hcom integration and is not claimed by this
  Limux release.

## [0.2.2] - 2026-07-15

### Added

- Added a left-aligned live application header showing the active workspace,
  active pane count, Limux process-tree RAM and CPU use, and live directory
  managers resolved from `hcom list mgrs --json`.
- Added configurable header section ordering and visibility through
  `header.sections` in `~/.config/limux/settings.json`.
- Added low-priority TaskMaster task 26 to make the active workspace row in the
  left sidebar unmistakable at a glance without conflicting with unread,
  favorite, focus, hover, or manual highlight states.

### Changed

- Moved the Limux name and version from the centered title position to the left
  side of the application header, with bold separators and a bold active
  workspace name.

### Fixed

- Moved process-tree sampling off the GTK thread into one bounded worker and
  added a hard deadline with child cleanup for live `hcom` manager queries.
- Bound manager-query results to their originating workspace directory so stale
  completions cannot overwrite the active workspace, while keeping new
  directories immediately eligible for refresh and showing an empty manager
  list when no workspace directory is active.

## [0.2.1] - 2026-07-10

### Added

- #32 `feat: add pane width lock`
- #50 `feat(skill): stage reconcile via Limux workflow`

### Changed

- #47 `fix(ui): improve pane width usability`
- #52 `fix(agent-team): require explicit live targets`
- #53 `fix(cli): add safe help and scoped surface close`

### Fixed

- #45 `fix(host): keep tab rename entry targetable`
- #48 `fix(host): remove readable pane width floor`
- #49 `fix(host): bound restored split-tree teardown`

### Documentation And Task State

- #46 `chore(taskmaster): close product hygiene lane`
- #51 `docs: reconcile primary checkout status`
- #54 `chore(tasks): close scoped CLI fixes`
- #55 `docs(hcom): add same-surface recovery ladder`

## [0.2.0] - 2026-07-08

### Added

- Added human-facing release progression on top of the existing machine identity:
  crate version `0.2.0`, `--version` output with semver, git SHA, build profile,
  install id, and runtime channel.
- Added this changelog as the versioned summary surface for merged work.

### Backfilled Merged PRs

- #33 `fix: stop Adw titlebar launch crash`
- #34 `fix: bound agent hook notification latency`
- #35 `docs: reconcile restore TaskMaster lane`
- #36 `fix(host): clear stale restored agents`
- #37 `fix(host): restore hcom-managed agents through hcom`
- #38 `fix(host): stagger restored agent startup`
- #39 `feat(host): classify PRD-E control routes`
- #40 `feat(host): wire pane focus control route`
- #41 `feat(host): wire pane resize control route`
- #42 `docs+scripts: operator-ratified convergence stamps + boundary tripwire`
- #43 `fix(release): add 0.2.0 version identity`
- #44 `feat(host): wire surface control mutations`

### Notes

- Git SHA remains the machine identity for exact build provenance.
- Semver is the human release layer and should be bumped at merge-wave
  boundaries.
