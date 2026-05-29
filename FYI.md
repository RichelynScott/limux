# FYI.md

Append-only journal for significant Limux session decisions and implementation notes.

## 2026-05-29 - Limux Agent-Team Protocol Safety And Resume Plan
### What:
Fixed the highest-risk `agent-team` behavior by moving generated protocol output from `AGENTS.md` to `LIMUX_AGENTS.md` by default, added `--protocol-path`, and documented the next zero-friction protocol discovery phase.

### Why:
The operator workflow depends on one visible Codex session plus one Claude Code session per project, often across 4+ projects with subagents, adversarial review, and hcom cross-team communication. Generated runtime protocol files must not clobber authoritative repo instructions.

### How:
Implemented and pushed `cec067f fix(cli): protect agent-team protocol output`; then ran a five-subagent brainstorm that recommended explicit instruction-source references instead of silent inheritance or copying. Created `HANDOFF.md` and refreshed the Limux vs Multica and Limux+hcom docs for morning resumption.

### Impact:
Existing repo `AGENTS.md` files are protected from default `agent-team` protocol generation. The next safe step is to make `LIMUX_AGENTS.md` easier for agents to discover by adding generated markers, detected instruction-source pointers, no-overwrite semantics, and a local extension file.

### Related:
`cec067f` | `HANDOFF.md` | `docs/cmux-parity-plan.md` | `docs/limux-hcom-workflow.md` | `docs/limux-vs-multica-decision-guide.md`

## 2026-05-29 - Next Steps Decision Packet
### What:
Created a dark-mode HTML decision packet for the operator to review current Limux status, next-step options, execution mode, skills, and acceptance criteria.

### Why:
The operator asked for an easier-to-read status update with selectable choices and a copy-back response before continuing implementation.

### How:
Used the `$html-decision-packet` pattern and current repo evidence from `HANDOFF.md`, `docs/cmux-parity-plan.md`, recent commits, and the install/security report. The packet defaults to the recommended path: implement Phase 5A zero-friction protocol discovery in the current session.

### Impact:
The next session can ask the operator to open `docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html`, copy selections back, and proceed without reconstructing the prior discussion.

### Related:
`docs/LIMUX_NEXT_STEPS_STATUS_DECISION_PACKET_2026-05-29.html` | `a1447e7` | `HANDOFF.md`

## 2026-05-29 - Phase 5A Agent-Team Protocol Discovery
### What:
Implemented Phase 5A for `limux agent-team`: generated `LIMUX_AGENTS.md` files now include a stable generated marker, an `Instruction Sources` section for detected `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`, metadata for path/modified time/deterministic hash, and a documented `LIMUX_AGENTS.local.md` local policy sidecar.

### Why:
The operator selected Limux + hcom as the primary orchestration path and needed near-zero-friction discovery without hidden prompt inheritance, copying, or clobbering authoritative repo instruction files.

### How:
Used TDD. Added RED tests for marker output, instruction-source references without content copying, unmarked sidecar refusal, explicit force overwrite, and symlink refusal. Hardened protocol writes with preflight validation, atomic temp-file replacement, no-overwrite semantics for unmarked files, and `--force-protocol-overwrite`.

### Impact:
`limux agent-team --dry-run` and live generation preserve existing repo `AGENTS.md` files, refuse unmarked `LIMUX_AGENTS.md` sidecars unless forced, refuse symlink protocol paths, and give agents direct pointers to authoritative instruction files. Verification passed for `cargo test -p limux-cli agent_team`, `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check`. The full `./scripts/check.sh` gate remains blocked until `ghostty/zig-out/lib/libghostty.so` is present. Claude plugin review timed out after 120 seconds and is not counted as passed.

### Related:
`rust/limux-cli/src/main.rs` | `README.md` | `docs/cmux-parity-plan.md` | `HANDOFF.md`

## 2026-05-29 - GTK Surface Send Text Readiness
### What:
Updated the live GTK bridge `surface.send_text` path so `TerminalHandle::send_text == false` returns a conflict error instead of a successful payload with `ok: true`.

### Why:
Automatic agent bootstrap depends on reliable send failure semantics. A resolved terminal surface that is not yet writable must not look successful to `limux-cli`, `agent-team`, or future bootstrap/adapters.

### How:
Added a small `surface_send_text_response` helper in `rust/limux-host-linux/src/window.rs`, wired `ControlCommand::SendText` through it, and added focused unit tests for writable and not-ready responses.

### Impact:
The GTK bridge now preserves the distinction between “surface found” and “surface writable.” `cargo test -p limux-cli`, `cargo fmt --check`, `cargo clippy -p limux-cli --all-targets -- -D warnings`, and `git diff --check` passed. Host-crate test execution is blocked in this environment because `pkg-config` is missing, causing GTK sys crates to fail before Rust test compilation.

### Related:
`rust/limux-host-linux/src/window.rs` | `rust/limux-host-linux/src/terminal.rs` | `docs/cmux-parity-plan.md` | `HANDOFF.md`
