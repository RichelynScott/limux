# Review Brief: Limux Cursor IDE Integration Plan

Date: 2026-06-30
Author/runtime: lifo / Codex
Target artifact:
`docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`

## Reviewer Task

Perform an adversarial review of the target plan. Do not implement code. Focus
on correctness, missing constraints, operational risk, and whether the proposed
v1/v2 split is the right shape for Limux and Cursor.

Return concise findings in this format:

```text
Verdict: PASS | PASS_WITH_CHANGES | WAIT | NO_GO
Top Findings:
1. [severity] finding with exact plan section and concrete fix
2. ...
Missing Evidence:
- ...
Recommended Plan Changes:
- ...
```

Severity labels:

- P0: must fix before drafting implementation.
- P1: should fix before implementation.
- P2: useful improvement or clarification.

## Context

Limux is a GTK4/libadwaita terminal workspace manager that embeds Ghostty for
terminal rendering. The operator uses it heavily to run Codex, Claude, Hermes,
and other terminal agent sessions. They miss Cursor's left-side file explorer
while working in Limux.

The current plan recommends v1 as a Cursor/VS Code-compatible extension bridge
plus a small Limux "Open in Cursor" host action. It explicitly avoids embedding
the GTK app in Cursor and avoids true live terminal attach until v2.

Key constraints:

- Keep v1 low risk and no-dependency where possible.
- Avoid host npm/npx/package execution in normal v1 operation.
- Treat arbitrary terminal text injection from Cursor as out of scope for v1.
- Support the same WSL/Linux environment first. Windows-host Cursor talking to
  WSL Limux directly is out of v1 unless Cursor is running through WSL/Remote.
- Preserve Limux as the owner of live PTY/Ghostty surfaces.

## Requested Review Lenses

Use the lens assigned by the runner. Suggested lenses:

- Architecture fit with Limux GTK/Ghostty/control bridge.
- Cursor/VS Code extension API feasibility.
- Unix socket protocol and trust boundary.
- WSL/runtime deployment failure modes.
- Security of workspace/folder launch and command exposure.
- Testability and acceptance criteria.
- V2 attach mode feasibility and risk.
- Scope discipline and implementation sequencing.

## Evidence To Use

Primary local files:

- `AGENTS.md`
- `README.md`
- `docs/cmux-parity-plan.md`
- `docs/future-improvements/limux-cursor-integration-options-after-pr-greenlight.md`
- `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`
- `rust/limux-host-linux/src/control_bridge.rs`
- `rust/limux-host-linux/src/window.rs`
- `rust/limux-cli/src/main.rs`

Hermes CLI command shape reference:

- `https://hermesagent.org.cn/en/docs/user-guide/cli`
- Example command: `hermes chat --model "<provider>/<model-name>" -q "..."`

## Output Rule

Keep the response under 900 words. Do not include chain of thought. Cite file
paths and concrete sections where possible.
