# Limux Session Handoff

Last updated: 2026-05-29 14:38 EDT

## Immediate Next Action

Implement the next scoped Limux improvement: **Phase 5A zero-friction protocol discovery** for `limux agent-team`.

The operator requested an easier-to-read status/options artifact on
2026-05-29. Use this packet if a decision needs to be confirmed before coding:

```text
docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html
```

Start here:

```bash
git status --short --branch
sed -n '1,220p' HANDOFF.md
sed -n '70,130p' docs/cmux-parity-plan.md
sed -n '210,285p' docs/limux-hcom-workflow.md
rg -n "run_agent_team|build_agents_md|resolve_agent_team_protocol_path|LIMUX_AGENTS" rust/limux-cli/src/main.rs
```

Then use TDD in `rust/limux-cli/src/main.rs` for Phase 5A:

1. Add a generated-file marker to `LIMUX_AGENTS.md`.
2. Add an `Instruction Sources` section that detects repo instruction files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`.
3. Make the section reference those files directly instead of copying or merging their contents.
4. Add file metadata for provenance, such as path, modified time, and hash, if cheap and deterministic.
5. Preserve the rule: repo instruction files stay authoritative; `LIMUX_AGENTS.md` only adds runtime topology and messaging protocol.
6. Add a no-overwrite guard for existing unmarked `LIMUX_AGENTS.md`; require an explicit force flag if replacement is needed.
7. Add or document a durable local extension point such as `LIMUX_AGENTS.local.md` for human/team policy notes that survive regeneration.

Recommended acceptance tests:

```bash
cargo test -p limux-cli agent_team
cargo fmt --check
cargo clippy -p limux-cli --all-targets -- -D warnings
git diff --check
```

Run `./scripts/check.sh` only after `libghostty.so` is present. The last full gate attempt failed because `libghostty` was missing, not because of the CLI changes.

## Completed This Session

| Time | Item | Result |
|---|---|---|
| 2026-05-29 early AM | Limux vs Multica decision packet | Created readable Markdown and dark-mode HTML decision guide with copy-back selections. |
| 2026-05-29 early AM | Multica adoption decision | User chose to keep Limux + hcom primary and defer Multica until after Limux fixes. |
| 2026-05-29 early AM | Global `$html-decision-packet` update request | Routed to `@kazu`; Kazu completed Sources/Evidence support in the global pattern/template/skill. |
| 2026-05-29 01:37 EDT | `agent-team` clobber fix | Commit `cec067f` changed default protocol output from `AGENTS.md` to `LIMUX_AGENTS.md` and added `--protocol-path`. |
| 2026-05-29 01:40 EDT | Verification | `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. |
| 2026-05-29 01:40 EDT | Full quality gate | `./scripts/check.sh` failed only because `libghostty` was missing; build prerequisite: `cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast`. |
| 2026-05-29 02:00 EDT | Subagent brainstorm | Five native subagents converged on reference-based instruction discovery, not silent inheritance/copying. |
| 2026-05-29 02:06 EDT | Stop-point docs | Created this handoff and FYI entry; refreshed decision/workflow docs for morning resumption. |

## Current State

- Branch: `main`
- Code commit pushed before this handoff: `cec067f fix(cli): protect agent-team protocol output`
- Latest pushed status/report commit before the next implementation step:
  `a1447e7 docs(security): add install dependency report`
- Current untracked docs before this handoff work:
  - `docs/limux-hcom-workflow.md`
  - `docs/limux-hcom-workflow.html`
  - `docs/limux-vs-multica-decision-guide.md`
  - `docs/limux-vs-multica-decision-guide.html`
- These docs are now intentionally part of the session record and should be committed with this handoff.

## Architectural Decisions Locked In

1. **No silent inheritance.** `LIMUX_AGENTS.md` should not copy, merge, or reinterpret `AGENTS.md` by default.
2. **Authority split.** Repo files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` remain authoritative project instructions.
3. **Runtime sidecar.** `LIMUX_AGENTS.md` is generated Limux runtime context: peers, surfaces, messaging, human notification, and routing.
4. **Zero-friction path.** Reduce friction through automated discovery, explicit pointers, environment variables, and later bootstrap/adapters, not hidden prompt composition.
5. **Launch automation waits.** Full two-phase automatic launch/bootstrap should wait until host send readiness is fixed and shell quoting is hardened.

## Subagent Brainstorm Synthesis

The proposed future shape:

```text
AGENTS.md / CLAUDE.md / GEMINI.md = project instructions
LIMUX_AGENTS.md                  = generated runtime protocol
LIMUX_AGENTS.local.md            = optional durable local team policy
.limux/ adapters                 = later tool-specific discovery helpers
```

Phase ordering:

1. **GO now:** Improve generated `LIMUX_AGENTS.md` with instruction-source detection, generated marker, no-overwrite guard, and local-policy extension point.
2. **GO next:** Fix `surface.send_text` readiness/failure semantics in the GTK host bridge.
3. **WAIT:** Implement two-phase agent bootstrap only after readiness and shell-quoting tests exist.
4. **Optional later:** Add runtime-specific `.limux/` adapters for Codex, Claude Code, Gemini, and OpenCode.

## Key Files For Context

| File | Purpose |
|---|---|
| `/home/riche/MCPs/limux/rust/limux-cli/src/main.rs` | `agent-team`, protocol generation, hook setup, tests. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | GTK bridge command handling; `surface.send_text` currently ignores the boolean return from terminal injection. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/terminal.rs` | `TerminalHandle::send_text` returns `false` when the Ghostty surface is not realized. |
| `/home/riche/MCPs/limux/docs/cmux-parity-plan.md` | Roadmap and current open bridge/protocol work. |
| `/home/riche/MCPs/limux/docs/limux-hcom-workflow.md` | Operator workflow for Limux plus hcom. |
| `/home/riche/MCPs/limux/docs/limux-vs-multica-decision-guide.md` | Decision record for Limux vs Multica and selected path. |
| `/home/riche/MCPs/limux/docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html` | Dark-mode copy-back packet for selecting the next implementation path. |
| `/home/riche/MCPs/limux/FYI.md` | Append-only session journal. |

## Critical Behavior Rules

- Do not modify repo `AGENTS.md` as part of `agent-team` runtime protocol generation.
- Do not implement hidden prompt inheritance. Use explicit detected source references.
- Do not launch hcom-managed workers for bounded local repo work unless a persistent cross-tool runtime is actually needed.
- Preserve `limux agent-team --dry-run` without a running host.
- Preserve `--no-launch` behavior when future bootstrap work starts.
- Use `apply_patch` for manual edits.
- Do not edit `/home/riche/.claude` from this Limux session.

## Known Risks And Blockers

- `./scripts/check.sh` needs `ghostty/zig-out/lib/libghostty.so`; build it before claiming the full workspace gate passes.
- Full automatic bootstrap has a readiness race until `surface.send_text` reports failure correctly through the GTK bridge.
- Shell-injected launch/bootstrap commands need tests for spaces, quotes, `$`, backticks, semicolons, and newlines before automation expands.
- `LIMUX_AGENTS.md` is safer than `AGENTS.md`, but still needs generated-marker and no-overwrite semantics.

## Morning Resume Prompt

```text
Please resume the Limux work from HANDOFF.md. Start with Phase 5A zero-friction protocol discovery for `limux agent-team`: generated marker, Instruction Sources section, no-overwrite guard for unmarked `LIMUX_AGENTS.md`, and local policy extension point. Use TDD and keep repo `AGENTS.md` authoritative.
```
