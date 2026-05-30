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

## 2026-05-29 - Install Posture Decision Packet
### What:
Created a dark-mode decision packet for whether to allow a fuller normal install posture after Phase 5A and GTK send-text hardening.

### Why:
The operator is close to allowing a regular install, but the safer next step is a bounded host prerequisite install/build lane rather than a full system Limux install. The decision packet makes the tradeoff explicit and gives a paste-back response.

### How:
Used the `$html-decision-packet` pattern. The packet recommends installing only the host build/test prerequisites after a mutation-script review gate, then running host tests, `scripts/check.sh`, and the Xvfb smoke path before automatic bootstrap work.

### Impact:
The next session can open `docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html`, copy back the selected install posture, and proceed without reconstructing the package/security discussion.

### Related:
`docs/LIMUX_INSTALL_POSTURE_DECISION_PACKET_2026-05-29.html` | `docs/install-security-report-2026-05-29.md` | `d60d2a3` | `edd781e`

## 2026-05-29 - Host Prerequisite Mutation Review
### What:
Prepared a draft-only mutation review for the bounded host prerequisite install/build lane.

### Why:
The operator selected a minimal host build/test prerequisite lane, with `$mutation-script-wave` before any `sudo apt install`, rather than a full Limux system install.

### How:
Ran read-only recon for OS, installed tools, package status, apt candidates, dependency simulation, Ghostty submodule/lib state, README prerequisites, `scripts/check.sh`, and the Xvfb smoke harness. Wrote the exact draft command block and review synthesis to `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md`.

### Impact:
Mutation wave decision is `WAIT`: the apt prerequisite lane is bounded and reviewable, but it still needs explicit human approval before execution. Zig acquisition and Ghostty build remain a separate follow-up gate.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | SHA256 `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec`

## 2026-05-29 - Host Prerequisite Execution Gate Stop
### What:
Attempted to execute the approved bounded host prerequisite command block from `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md`.

### Why:
The operator explicitly approved the exact apt prerequisite command block with SHA256 `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec` so the previously blocked host GTK test could move past the missing `pkg-config` prerequisite.

### How:
Recomputed the SHA256 and confirmed it matched. Ran the frozen block until the first privileged mutation command. The pre-mutation evidence and apt simulation ran; the transaction remained the reviewed `2 upgraded, 160 newly installed, 0 to remove and 94 not upgraded` lane. Execution then stopped at `sudo apt-get update` because sudo required a password. The run was cancelled instead of collecting or handling a password in chat.

### Impact:
No apt package install occurred. `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and `libwebkitgtk-6.0-dev` remain absent. The next continuation point is to make sudo credentials available outside chat, re-verify the same review artifact SHA, and rerun the approved prerequisite block. Zig acquisition, Ghostty submodule initialization, and Ghostty build remain separate gates.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`

## 2026-05-29 - Sudo Cache Did Not Carry Into Codex PTY
### What:
Retried the approved prerequisite lane after the operator ran `sudo -v` locally, but did not execute the apt install.

### Why:
Before rerunning the frozen apt block, the session checked whether cached sudo credentials were visible inside Codex with `sudo -n true`.

### How:
Verified the mutation review artifact SHA still matched `de2a31ac73a1f85b9c559b479507b3a541871771a194b6c5f77a8a9e6150bbec` and confirmed the repo was clean. `sudo -n true` returned `sudo: a password is required`, so the Codex execution context still cannot run privileged commands without prompting for a password.

### Impact:
No OS package mutation occurred. The apt prerequisite lane remains blocked from inside Codex unless sudo credentials are made available to the same execution context, or the operator runs the approved command block manually in their own terminal. The approved review artifact was not edited, preserving its SHA.

### Related:
`docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`

## 2026-05-29 - Host Prerequisites Installed, Ghostty Gate Reached
### What:
The operator manually ran the approved host prerequisite apt lane in a trusted terminal. `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and `libwebkitgtk-6.0-dev` are now installed.

### Why:
The previous Codex execution context could not access cached sudo credentials, so the operator completed the bounded apt prerequisite install manually while preserving the reviewed package scope.

### How:
Verified post-install state with `dpkg-query` and `pkg-config --modversion gtk4 libadwaita-1 webkitgtk-6.0`. Versions are `pkg-config 1.8.1-2build1`, `pkgconf 1.8.1-2build1`, GTK `4.14.5`, libadwaita `1.5.0`, and WebKitGTK `2.52.3`.

### Impact:
The host test moved past the prior GTK/pkg-config sys-crate blocker and now fails at the expected next gate: `limux-ghostty-sys` cannot find `ghostty/zig-out/lib/libghostty.so`. The `ghostty/` submodule is still uninitialized and `zig` is still not on `PATH`. Zig acquisition, Ghostty submodule initialization, and Ghostty build remain a separate reviewed gate.

### Related:
`rust/limux-ghostty-sys/build.rs` | `HANDOFF.md`

## 2026-05-29 - Ghostty/Zig Mutation Review Prepared
### What:
Prepared a draft-only mutation review for the next Ghostty/Zig build gate.

### Why:
The apt prerequisite lane is complete, and the active host-test blocker is now missing `ghostty/zig-out/lib/libghostty.so`. Resolving that requires fetching the pinned Ghostty submodule and acquiring/building with Zig, which is a separate external-code and native-build supply-chain lane.

### How:
Reviewed README build instructions, `.gitmodules`, current submodule state, `scripts/package.sh`, the pinned Ghostty `build.zig.zon`, official Zig release metadata, and local package/tool availability. Wrote an exact draft command block that uses a project-scoped Zig `0.15.2` tarball from `ziglang.org`, verifies SHA256 `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`, initializes only the pinned `ghostty` submodule, builds `libghostty.so`, and reruns the host readiness test.

### Impact:
Mutation wave decision is `WAIT`: the next lane is bounded and reviewable, but it still needs explicit human approval before execution. No Ghostty/Zig commands were executed.

### Related:
`docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md`

## 2026-05-29 - Ghostty/Zig Security Consensus Gate
### What:
Ran a multi-session security consensus gate on the Ghostty/Zig mutation review using `kazu`, `zori`, `niru`, and the local Claude plugin adversarial review.

### Why:
The next Limux blocker requires downloading Zig, initializing the pinned Ghostty submodule, and building native external code. The operator asked for a consensus security gate before proceeding.

### How:
Sent a durable hcom review brief to the named reviewers, collected v1 `WAIT` findings, patched the mutation review to v2, and ran a narrow v2 re-review. V2 added execution-time Zig metadata cross-checks, fresh per-run extraction, archive containment checks, non-recursive submodule init, explicit `am-will/ghostty` trust-anchor documentation, offline locked Cargo test, and durable evidence logs.

### Impact:
Consensus result is `GO for explicit operator approval; WAIT for execution`. The frozen v2 artifact SHA is `dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`. No Ghostty/Zig command block was executed.

### Related:
`docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md` | `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md` | hcom thread `limux-ghostty-zig-gate`

## 2026-05-29 - Approved Ghostty/Zig Build Gate Executed
### What:
Executed the approved Ghostty/Zig v2 build gate after verifying artifact SHA256 `dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`.

### Why:
The host GTK test was blocked because `limux-ghostty-sys` could not find `ghostty/zig-out/lib/libghostty.so`.

### How:
Verified the frozen review artifact hash, command syntax, repo status, Zig metadata from official `index.json`, Zig archive SHA256 and byte size, archive containment, pinned Ghostty commit `81ab8ffa90185221782baf785e85387321e16f8d`, and absence of nested Ghostty submodules. Built `libghostty.so` with project-scoped Zig `0.15.2`, captured dynamic-link evidence, and ran `CARGO_NET_OFFLINE=true cargo test --locked -p limux-host-linux surface_send_text_response`.

Execution wrapper note: the shell extraction command accidentally captured an earlier illustrative README bash fence before the approved v2 block. That first fence initialized the top-level `ghostty` submodule and attempted `zig build`, which failed immediately because `zig` was not on `PATH`; the approved v2 block then executed successfully. Follow-up inspection found the submodule at the pinned commit, no nested submodules, and no extra system mutation.

### Impact:
`ghostty/zig-out/lib/libghostty.so` now exists locally, and the focused host test passed offline with 2 tests passing. Evidence is stored under `docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/`. The focused host test exposed an existing `unused_mut` warning in `rust/limux-host-linux/src/window.rs`; full clippy/check work should address that before claiming the complete workspace gate.

### Related:
`docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/` | `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md` | `HANDOFF.md`
