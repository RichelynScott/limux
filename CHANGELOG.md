# Changelog

All notable Limux changes should be recorded here when a PR merges.

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
