# PRD: Product Hygiene Versioning And Tab Rename Focus

## Introduction

Limux has two operator-visible product hygiene issues. First, the displayed
version still reads as `0.1.19` even after substantial merged work, and there is
no durable changelog that maps version progression to merged PRs. Second, the
tab rename entry can fail to receive pointer focus because it is inserted into a
non-targetable tab label container, causing typing to fall through to the
terminal pane.

## Goals

- Make `limux --version` communicate human release progression and machine
  provenance in one line.
- Start the next merge-wave version at `0.2.0`.
- Add a changelog entry backfilling merged PRs #33 through #41.
- Record the tab rename focus diagnosis and defer its code change until the
  active surface-group lane has merged.

## Non-Goals

- Do not change packaging workflows beyond reading the workspace version and
  existing install metadata.
- Do not edit `pane.rs` or `window.rs` for the tab rename fix in this branch.
- Do not alter PRD-E route behavior, hcom restore behavior, or runtime channel
  semantics.

## User Stories

### US-001: Human-readable version identity

As the operator, I want `limux --version` to show semver, git SHA, build
profile, install id, and channel so I can tell whether a running build includes
recent work.

Acceptance criteria:

- [ ] Workspace crate version is bumped to `0.2.0`.
- [ ] CLI `--version` renders `<binary> 0.2.0 (<sha>, <profile>) install-id=<id> channel=<channel>`.
- [ ] Host `--version` uses the same shared render contract.
- [ ] Existing install metadata remains compatible and optional.
- [ ] Focused tests for version rendering pass.

### US-002: Changelog backfill

As the operator, I want a per-version changelog so I can map the current human
version to merged work.

Acceptance criteria:

- [ ] `CHANGELOG.md` exists.
- [ ] `0.2.0` includes merged PRs #33 through #41.
- [ ] README install examples and version diagnostics match `0.2.0`.

### US-003: Tab rename focus diagnosis

As the operator, I want the tab rename textbox to accept pointer focus so typing
does not go into the terminal pane.

Acceptance criteria:

- [ ] Diagnosis records that the rename entry is inserted into a
      `set_can_target(false)` tab title container.
- [ ] Code mutation waits for the surface-group lane to merge.
- [ ] Follow-up implementation branch is planned from post-surface-group
      `main`.

## Technical Notes

- Existing `install-info.json` already includes `install_id`, `channel`, and
  `source_sha`; this PR keeps that contract and adds version/profile stamps for
  human inspection.
- `BuildInfo` remains the shared source for CLI, host, doctor, and identify
  build metadata.
- The tab rename fix should temporarily make the rename-entry parent
  targetable, restore the original targetability after commit/cancel, and add a
  regression check for editable tab entries.

## Verification Plan

- `cargo test -p limux-core render_version_line`
- `cargo test -p limux-cli version`
- `cargo test -p limux-host-linux version`
- `cargo check -p limux-cli`
- `cargo check -p limux-host-linux`
- `git diff --check`

## Gate Map

- Version/changelog: approved by NATO on hcom #331071.
- Tab rename code: blocked until the surface-group PR merges; diagnosis only in
  this branch.
- Package execution: no package installs or host package-resolution commands.
