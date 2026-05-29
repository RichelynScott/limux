# Limux Session Handoff

Last updated: 2026-05-29 19:08 EDT

## Immediate Next Action

Phase 5A zero-friction protocol discovery for `limux agent-team` is implemented
and locally verified. GTK bridge `surface.send_text` now reports not-ready
terminal surfaces as a conflict instead of returning `ok: true`. The next
scoped code option is shell-quoting/test coverage for future automatic
bootstrap, after host GTK/pkg-config/Ghostty prerequisites are available.

The operator requested an easier-to-read status/options artifact on
2026-05-29. Use this packet if a decision needs to be confirmed before coding:

```text
docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html
```

For the current install-prerequisite decision, use:

```text
docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html
```

The mutation review for the selected bounded host prerequisite lane is:

```text
docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md
```

The operator approved the exact command block in that file on 2026-05-29.
Artifact SHA256 at approval and pre-run verification:
`de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec`.

Execution attempt status: `BLOCKED BEFORE MUTATION`. The pre-mutation evidence
and apt simulation ran, then execution stopped at `sudo apt-get update` because
sudo required a password. The run was cancelled instead of collecting or
handling a password in chat. No apt package install occurred, and `pkg-config`
is still absent.

Recommended continuation: have the operator make sudo credentials available
outside chat, re-verify the artifact SHA above, then rerun the approved
prerequisite block from the review file. Do not widen the block: no Zig
download, no submodule init/update, no Ghostty build, and no system-wide Limux
install in this lane.

Start here:

```bash
git status --short --branch
sed -n '1,220p' HANDOFF.md
sed -n '70,130p' docs/cmux-parity-plan.md
sed -n '210,285p' docs/limux-hcom-workflow.md
rg -n "run_agent_team|build_agents_md|resolve_agent_team_protocol_path|LIMUX_AGENTS" rust/limux-cli/src/main.rs
```

Phase 5A completed in `rust/limux-cli/src/main.rs`:

1. Added a generated-file marker to `LIMUX_AGENTS.md`.
2. Added an `Instruction Sources` section that detects `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`.
3. The section references those files directly instead of copying or merging their contents.
4. Metadata includes path, modified time, and deterministic `fnv1a64` content hash for regular files.
5. Repo instruction files stay authoritative; `LIMUX_AGENTS.md` only adds runtime topology and messaging protocol.
6. Existing unmarked `LIMUX_AGENTS.md` files are refused by default; `--force-protocol-overwrite` is required to replace one.
7. `LIMUX_AGENTS.local.md` is documented as the durable local policy sidecar; Limux does not create or overwrite it.

Recommended acceptance tests:

```bash
cargo test -p limux-cli agent_team
cargo fmt --check
cargo clippy -p limux-cli --all-targets -- -D warnings
git diff --check
```

Run `./scripts/check.sh` only after `libghostty.so` is present. The last full gate attempt failed because `libghostty` was missing, not because of the CLI changes.

For host-side GTK tests, this environment also needs `pkg-config` available on
`PATH`; otherwise `gio-sys`, `glib-sys`, `gobject-sys`, `cairo-sys-rs`, and
`gdk-pixbuf-sys` fail before Rust test compilation starts.

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
| 2026-05-29 17:00 EDT | Phase 5A implementation | Added generated marker, instruction-source metadata, no-overwrite guard, explicit force flag, local policy sidecar docs, and regression tests. |
| 2026-05-29 17:00 EDT | Verification | `cargo test -p limux-cli agent_team`, `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. |
| 2026-05-29 17:00 EDT | Cross-family review attempt | Claude plugin read-only review timed out after 120 seconds without findings; do not treat it as a passed review. |
| 2026-05-29 17:29 EDT | GTK send-text readiness fix | Updated the live GTK `surface.send_text` handler to convert `TerminalHandle::send_text == false` into a conflict error. Added focused unit tests for the response helper. |
| 2026-05-29 17:29 EDT | Verification | `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. `cargo test -p limux-host-linux surface_send_text_response` is blocked because `pkg-config` is missing. |
| 2026-05-29 17:44 EDT | Host prerequisite mutation review | Created draft-only mutation review for apt prerequisites. Decision is `WAIT` pending explicit approval. Zig/Ghostty remain separate gates. |
| 2026-05-29 19:07 EDT | Approved prerequisite block attempt | Verified artifact SHA, ran the pre-mutation evidence and apt simulation, then stopped at the first sudo command because a password was required. No packages were installed. |

## Current State

- Branch: `main`
- Code commit pushed before this handoff: `cec067f fix(cli): protect agent-team protocol output`
- Latest implementation in this handoff: GTK `surface.send_text` readiness/failure hardening.
- Latest pushed status/report commit before Phase 5A implementation:
  `1c12e97 docs(decision): add limux next steps packet`
- Working tree should be clean after committing/pushing the Phase 5A implementation.

## Architectural Decisions Locked In

1. **No silent inheritance.** `LIMUX_AGENTS.md` should not copy, merge, or reinterpret `AGENTS.md` by default.
2. **Authority split.** Repo files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` remain authoritative project instructions.
3. **Runtime sidecar.** `LIMUX_AGENTS.md` is generated Limux runtime context: peers, surfaces, messaging, human notification, and routing.
4. **Zero-friction path.** Reduce friction through automated discovery, explicit pointers, environment variables, and later bootstrap/adapters, not hidden prompt composition.
5. **Launch automation waits.** Full two-phase automatic launch/bootstrap should wait until shell quoting is hardened and the live GTK/Xvfb path can be verified with host prerequisites installed.

## Subagent Brainstorm Synthesis

The proposed future shape:

```text
AGENTS.md / CLAUDE.md / GEMINI.md = project instructions
LIMUX_AGENTS.md                  = generated runtime protocol
LIMUX_AGENTS.local.md            = optional durable local team policy
.limux/ adapters                 = later tool-specific discovery helpers
```

Phase ordering:

1. **Done:** Improve generated `LIMUX_AGENTS.md` with instruction-source detection, generated marker, no-overwrite guard, and local-policy extension point.
2. **Done, pending host-crate build prerequisite verification:** Fix `surface.send_text` readiness/failure semantics in the GTK host bridge.
3. **WAIT:** Implement two-phase agent bootstrap only after shell-quoting tests and host smoke verification exist.
4. **Optional later:** Add runtime-specific `.limux/` adapters for Codex, Claude Code, Gemini, and OpenCode.

## Key Files For Context

| File | Purpose |
|---|---|
| `/home/riche/MCPs/limux/rust/limux-cli/src/main.rs` | `agent-team`, protocol generation, hook setup, tests. |
| `/home/riche/MCPs/limux/rust/limux-host-linux/src/window.rs` | GTK bridge command handling; `surface.send_text` now errors if terminal injection reports not-ready. |
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
- Host-crate tests currently need `pkg-config` on `PATH`; this environment still did not have it after the 2026-05-29 19:07 EDT sudo gate stop.
- Shell-injected launch/bootstrap commands need tests for spaces, quotes, `$`, backticks, semicolons, and newlines before automation expands.
- Instruction-source hashes are deterministic `fnv1a64` metadata for change detection, not cryptographic integrity claims.

## Morning Resume Prompt

```text
Please resume the Limux work from HANDOFF.md. Phase 5A zero-friction protocol discovery is implemented and GTK `surface.send_text` now returns a conflict when the terminal is not ready. Before automatic bootstrap, install/verify host prerequisites (`pkg-config`, GTK dev libraries, Ghostty lib), run host/Xvfb checks, then add shell-quoting tests for launch/bootstrap commands.
```
