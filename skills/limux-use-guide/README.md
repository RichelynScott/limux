# Limux Use Guide Skill Staging

## Audience

Agents and operators who use Limux as their terminal workspace manager, drive
Limux through its CLI/socket, diagnose a running Limux runtime, or launch agent
sessions inside Limux panes.

## Classification

Public repo safe. This directory contains mechanics-only guidance for Limux
commands, runtime channels, diagnostics, verification, and the Limux/hcom
integration boundary.

## Canonical Source

The canonical sources for this staged skill are:

- `README.md`
- `AGENTS.md`
- `./target/debug/limux-cli --help` or the matching built CLI help
- `docs/verification/post-install-checklist-v1.md`
- `docs/verification/host-owned-surface-process-attestation.md`
- `docs/verification/run-template.md`
- `docs/cmux-parity-plan.md`
- `rust/limux-cli/src/main.rs`
- `rust/limux-host-linux/src/control_registry.rs`
- `rust/limux-host-linux/src/state_mirror.rs`

When command behavior changes, update the code and user/contributor docs first,
then update this staged skill from those sources.

## Promotion Target

After review and merge, this staged skill may be promoted to global Codex and
Claude skill locations under the name `limux-use-guide`. Global promotion is a
separate owner-gated global-config task; promoted mirrors should point back to
this repo as the command/source canonical rather than becoming independent
command references.

## Forbidden Content

Do not add secrets, tokens, private fleet rosters, customer/project policy,
private hcom names, local absolute operator paths, screenshots containing
private content, generated runtime IDs, or machine-specific session state.

## Maintenance Notes

- Keep examples runnable with the current built or installed CLI.
- Prefer `limux --help`, `doctor`, `target-info`, and source files over memory.
- Mark partial PRD work as partial; do not describe planned browser-pane or
  lifecycle features as complete until the matching PRD task is closed.
