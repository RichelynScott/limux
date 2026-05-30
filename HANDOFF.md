# Limux Session Handoff

Last updated: 2026-05-29 23:23 EDT

## Immediate Next Action

Phase 5C durable `limux agent-team` coordination files are implemented. The
current flow writes protected generated protocol to `LIMUX_AGENTS.md`, seeds
`LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` when missing, launches peer
panes with bare agent commands, waits for pane readiness, sends each peer a
sanitized one-line bootstrap prompt after all coordination files exist, then
submits it with explicit Enter. `--no-bootstrap`, `--no-launch`, and `--dry-run`
all skip prompt sends. `--dry-run` still materializes the generated protocol
and seeds missing roster/ledger files; it only skips host contact.

Recommended next scoped action: implement a reviewer/capture wrapper plus
documented consensus and cross-team broadcast conventions. The wrapper should
spawn a focused reviewer, capture/read the result, write a ledger entry or
consensus report, and send hcom pointers only to relevant teams.

Current verification baseline:

```bash
cargo fmt --check
git diff --check
LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/xvfb-smoke-test.sh
```

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

Initial Codex execution attempt status: `BLOCKED BEFORE MUTATION`. The pre-mutation evidence
and apt simulation ran, then execution stopped at `sudo apt-get update` because
sudo required a password. The run was cancelled instead of collecting or
handling a password in chat. No apt package install occurred, and `pkg-config`
was still absent at that point.

Second Codex attempt status: `STILL BLOCKED BEFORE MUTATION`. After the operator ran
`sudo -v` locally, Codex checked `sudo -n true` in its execution context. Sudo
still returned `sudo: a password is required`, which indicates the local sudo
cache did not carry into the Codex PTY/session. No package mutation occurred.

Manual operator execution status: `APT PREREQUISITES INSTALLED`. The operator
ran the approved apt lane manually in a trusted terminal. Post-install checks
show `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and
`libwebkitgtk-6.0-dev` installed. `pkg-config --modversion gtk4 libadwaita-1
webkitgtk-6.0` reports `4.14.5`, `1.5.0`, and `2.52.3`.

Previous blocker resolved: the host test now finds
`ghostty/zig-out/lib/libghostty.so`. The `ghostty/` submodule is initialized at
the pinned commit, and project-scoped Zig `0.15.2` was used from
`$HOME/.cache/limux-tools`. Zig is still not installed system-wide and is not
expected on `PATH`.

The draft-only Ghostty/Zig mutation review for that next gate is:

```text
docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md
```

The operator approved the exact v2 command block on 2026-05-29. Current v2
artifact SHA256:
`dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`.

Consensus gate result was `GO for explicit operator approval; WAIT for
execution`. Reviewers `niru`, `zori`, `kazu`, and the local Claude plugin
cleared v2 for approval consideration. The consensus report is:

```text
docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md
```

Execution result: `COMPLETE WITH WRAPPER DEVIATION DOCUMENTED`. The v2 lane used
project-scoped Zig `0.15.2` from official Zig metadata, SHA256
`02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`, the pinned
`am-will/ghostty` submodule commit
`81ab8ffa90185221782baf785e85387321e16f8d`, and evidence under:

```text
docs/evidence/limux-ghostty-zig-20260530T002418Z-18756/
```

Focused host verification passed:

```bash
CARGO_NET_OFFLINE=true LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" cargo test --locked -p limux-host-linux surface_send_text_response
```

Result: `2 passed; 0 failed; 186 filtered out`. The later follow-up removed the
`unused_mut` warning at `rust/limux-host-linux/src/window.rs:4340`, and the full
workspace gate now passes.

Execution wrapper deviation: the command extraction wrapper captured all
`bash` fences in the review doc, so it first ran the README illustrative block:
`git submodule update --init --recursive` and
`(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)`. The build
failed immediately because `zig` was not on `PATH`; the approved v2 block then
ran successfully. Follow-up inspection found `ghostty` at the pinned commit,
`ghostty/.gitmodules` absent/non-empty check passed, and
`git -C ghostty submodule status --recursive` returned no nested submodules.

Start here:

```bash
git status --short --branch
sed -n '1,220p' HANDOFF.md
sed -n '70,130p' docs/cmux-parity-plan.md
sed -n '210,285p' docs/limux-hcom-workflow.md
rg -n "run_agent_team|build_agents_md|LIMUX_AGENTS|LIMUX_TEAM_ROSTER|LIMUX_REVIEW_LEDGER" rust/limux-cli/src/main.rs
```

Phase 5A completed in `rust/limux-cli/src/main.rs`:

1. Added a generated-file marker to `LIMUX_AGENTS.md`.
2. Added an `Instruction Sources` section that detects `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md`.
3. The section references those files directly instead of copying or merging their contents.
4. Metadata includes path, modified time, and deterministic `fnv1a64` content hash for regular files.
5. Repo instruction files stay authoritative; `LIMUX_AGENTS.md` only adds runtime topology and messaging protocol.
6. Existing unmarked `LIMUX_AGENTS.md` files are refused by default; `--force-protocol-overwrite` is required to replace one.
7. `LIMUX_AGENTS.local.md` is documented as the durable local policy sidecar; Limux does not create or overwrite it.

Phase 5B completed in `rust/limux-cli/src/main.rs`,
`rust/limux-host-linux/src/window.rs`,
`rust/limux-host-linux/src/terminal.rs`, and `scripts/xvfb-smoke-test.sh`:

1. Added `--no-bootstrap` for live `agent-team` runs.
2. Kept generated pane-create commands as bare launchers such as `codex` or
   `claude`; arbitrary orientation text is sent only after the pane is created.
3. Wrote `LIMUX_AGENTS.md` before any bootstrap prompt send.
4. Sanitized generated bootstrap prompts more strictly than normal typed text:
   no CR, no tab, no LF, no bidi format controls, and no zero-width
   display-spoofing characters.
5. Sent the prompt through `surface.send_text`, then submitted it through
   `surface.send_key enter` so shells that treat paste/newline conservatively
   still receive the message.
6. Made live smoke use fake `codex`/`claude` binaries to prove the prompt was
   received after protocol write.
7. Fixed Ghostty Enter key submission for command-launch paths.

Phase 5C completed in `rust/limux-cli/src/main.rs` and
`scripts/xvfb-smoke-test.sh`:

1. Added default `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md`
   coordination files.
2. Added `--roster-path <path>`, `--ledger-path <path>`, and
   `--force-roster-overwrite`.
3. Seeded the roster and ledger when missing before any live bootstrap prompt.
4. Preserved existing roster and ledger files by default; the ledger remains
   create-if-missing only.
5. Rejected symlink, non-regular, and overlapping roster/ledger/protocol
   targets.
6. Pointed generated `LIMUX_AGENTS.md` and bootstrap prompts to the durable
   roster and ledger.
7. Expanded CLI tests and Xvfb smoke proof for creation, preservation, forced
   marked-roster replacement, unmarked force refusal, symlink refusal,
   overlapping-path refusal, and fake-agent file visibility.

Recommended acceptance tests:

```bash
cargo test -p limux-cli agent_team
cargo test -p limux-cli
cargo fmt --check
cargo clippy -p limux-cli --all-targets -- -D warnings
LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/xvfb-smoke-test.sh
git diff --check
```

`libghostty.so` and host GTK/pkg-config prerequisites are present locally. The
focused host warning is fixed, and the full workspace/Xvfb gates now pass with
the local Ghostty library. The smoke script exports `LD_LIBRARY_PATH`
automatically when `ghostty/zig-out/lib` exists; the full check still needs the
explicit `LD_LIBRARY_PATH` prefix.

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
| 2026-05-29 19:24 EDT | Sudo cache follow-up | Operator ran `sudo -v` locally, but `sudo -n true` inside Codex still required a password. No packages were installed. |
| 2026-05-29 19:51 EDT | Manual apt prerequisite completion | Operator manually completed the approved apt prerequisite lane. GTK/WebKit pkg-config checks pass. Host test now fails at the separate Ghostty/Zig gate. |
| 2026-05-29 20:10 EDT | Ghostty/Zig mutation review | Created draft-only review for project-scoped Zig 0.15.2 download, pinned Ghostty submodule initialization, `libghostty.so` build, and host test verification. Decision is `WAIT` pending explicit approval. |
| 2026-05-29 20:15 EDT | Ghostty/Zig consensus gate | `niru`, `zori`, `kazu`, and Claude plugin reviewed v1, returned `WAIT`, v2 was patched, then v2 re-review returned GO for operator approval. |
| 2026-05-29 20:32 EDT | Approved Ghostty/Zig execution | Verified v2 SHA, built `ghostty/zig-out/lib/libghostty.so`, captured evidence logs, and passed the locked offline host send-text test. Wrapper deviation documented: an earlier README bash fence initialized the top-level `ghostty` submodule before the approved v2 block; no nested submodules or system mutation were found. |
| 2026-05-29 20:47 EDT | Full gate and Xvfb smoke restored | Removed the host `unused_mut` warning, updated Xvfb smoke from `softpipe`/OpenGL 3.3 to `llvmpipe`/OpenGL 4.3, accepted current `new-pane --json` refs, and verified `cargo fmt --check`, `git diff --check`, `./scripts/check.sh`, and `./scripts/xvfb-smoke-test.sh`. |
| 2026-05-29 21:10 EDT | Shell-quoted launch snippet hardening | Added central generated-snippet shell quoting, quoted generated `LIMUX_AGENTS.md` scratch-pane commands, rejected unquoted extra `new-pane` positionals, removed nested prompt examples from docs, and verified focused CLI tests, full workspace check, and Xvfb smoke. Claude plugin review timed out; hcom reviewers converged on GO for the manual snippet path and deferred typed-PTY control-character policy before auto-bootstrap. |
| 2026-05-29 21:36 EDT | Typed-PTY control-character guard | Added shared typed-text validation in `limux-protocol`; enforced it in the CLI, standalone core dispatcher, live GTK bridge parser, and GTK host send sink; documented `send-key` as the control-key route; expanded Xvfb smoke stage 7 to reject ESC/BEL/C1 payloads across send/new-pane/respawn/paste/new-workspace. Claude plugin review timed out after 240 seconds, so it is not counted as passed; hcom reviewers `kazu`, `zori`, and `niru` had already converged on the policy shape. |
| 2026-05-29 22:31 EDT | Phase 5B automatic bootstrap | Added post-launch `agent-team` bootstrap prompts, `--no-bootstrap`, protocol-write-before-send behavior, stricter generated-prompt validation, explicit Enter submission, command-launch Enter fixes, fake-agent Xvfb proof, and refreshed workflow/decision/handoff docs. |
| 2026-05-29 23:23 EDT | Phase 5C durable roster and review ledger | Added `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` seeding, `--roster-path`, `--ledger-path`, `--force-roster-overwrite`, no-overwrite ledger preservation, marked-roster force replacement, symlink/nonregular/overlapping path refusal, bootstrap pointers, CLI tests, Xvfb fake-agent file-visibility proof, and refreshed workflow/decision/handoff docs. |

## Current State

- Branch: `main`
- Phase 5B baseline commit: `0d2597b feat(cli): bootstrap agent-team peers after launch`
- Latest implementation in this handoff: Phase 5C durable roster and review-ledger seeding after Phase 5B automatic agent-team bootstrap.
- Working tree should be clean after committing/pushing Phase 5C.

## Architectural Decisions Locked In

1. **No silent inheritance.** `LIMUX_AGENTS.md` should not copy, merge, or reinterpret `AGENTS.md` by default.
2. **Authority split.** Repo files such as `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` remain authoritative project instructions.
3. **Runtime sidecar.** `LIMUX_AGENTS.md` is generated Limux runtime context: peers, surfaces, messaging, human notification, and routing.
4. **Durable coordination files.** `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` are create-if-missing durable operator files, not generated overwrite targets. Live surface/pane/workspace IDs remain in `LIMUX_AGENTS.md`.
5. **Zero-friction path.** Reduce friction through automated discovery, explicit pointers, environment variables, bootstrap, and later adapters, not hidden prompt composition.
6. **Launch automation waits.** Generated launch snippets should only start the agent binary. Automatic launch/bootstrap sends bounded prompt text only after protocol/roster/ledger write and pane readiness through guarded typed-text plus explicit `send-key enter`.

## Subagent Brainstorm Synthesis

The proposed future shape:

```text
AGENTS.md / CLAUDE.md / GEMINI.md = project instructions
LIMUX_AGENTS.md                  = generated runtime protocol
LIMUX_AGENTS.local.md            = optional durable local team policy
LIMUX_TEAM_ROSTER.md             = durable project/team routing roster
LIMUX_REVIEW_LEDGER.md           = durable review/consensus ledger
.limux/ adapters                 = later tool-specific discovery helpers
```

Phase ordering:

1. **Done:** Improve generated `LIMUX_AGENTS.md` with instruction-source detection, generated marker, no-overwrite guard, and local-policy extension point.
2. **Done:** Fix `surface.send_text` readiness/failure semantics in the GTK host bridge and verify it through the full workspace gate.
3. **Done:** Add caller-shell quoting tests and generated-snippet hardening before expanding automatic launch/bootstrap behavior.
4. **Done:** Define and test the typed-PTY control-character policy for `limux send`, respawn, paste-buffer, `pane.create --command`, `workspace.create --command`, direct socket callers, and the live GTK host sink.
5. **Done:** Implement two-phase automatic bootstrap: launch the agent binary, wait for pane readiness, then send prompt text through guarded `surface.send_text` plus explicit Enter.
6. **Done:** Seed a project/team roster and durable review/consensus ledger.
7. **Next:** Add a reviewer/capture wrapper and consensus/cross-team broadcast conventions.
8. **Optional later:** Add runtime-specific `.limux/` adapters for Codex, Claude Code, Gemini, and OpenCode.

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
- Preserve `--no-launch` and `--no-bootstrap` behavior for `agent-team`;
  neither path should send bootstrap prompts.
- Preserve existing `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` by
  default. The ledger is append/manual state and must not be overwritten by
  `agent-team`. `--force-roster-overwrite` is only for marked Limux rosters.
- Use `apply_patch` for manual edits.
- Do not edit `/home/riche/.claude` from this Limux session.

## Known Risks And Blockers

- `ghostty/zig-out/lib/libghostty.so` exists locally after the approved build gate, but it is a generated artifact. Fresh clones or cleaned worktrees must rebuild it through the reviewed lane before host/workspace checks.
- Host-crate tests moved past the prior `pkg-config` and `libghostty` blockers. The `unused_mut` warning at `rust/limux-host-linux/src/window.rs:4340` is fixed.
- The Xvfb smoke harness requires Mesa software OpenGL 4.3 for the pinned Ghostty. It now defaults to `llvmpipe` and can be overridden with `LIMUX_SMOKE_GALLIUM_DRIVER` for local Mesa debugging.
- `zig` is intentionally not on `PATH`; the reviewed lane used project-scoped Zig under `$HOME/.cache/limux-tools`.
- Caller-shell generated snippet tests now cover spaces, quotes, `$`, command substitution, backticks, semicolons, control characters, newlines, exact JSON preservation, and side-effect inertness.
- Typed-PTY control characters are now rejected everywhere the current control surface can inject typed terminal text. Intentional control keys must use `surface.send_key` / `limux send-key`.
- Bootstrap prompt generation now rejects CR, tab, LF, bidi format controls, and zero-width display-spoofing characters even though the broader typed-text policy still allows tab/LF/CR for manual multiline messages. Keep that stricter boundary for generated automatic prompts.
- Instruction-source hashes are deterministic `fnv1a64` metadata for change detection, not cryptographic integrity claims.
- Claude plugin adversarial review did not complete for the shell-quoting lane: normal mode timed out after 180 seconds, and `--bare` mode failed because Claude was not logged in under bare mode. hcom reviewer `kazu` provided the Claude-family shell-safety lens instead. For the typed-PTY lane, the normal plugin review timed out after 240 seconds and is not counted as passed.
- Claude plugin adversarial review completed for Phase 5B. It found no security-blocking defect, but flagged reliability issues that were handled before commit: removed trailing-LF double submission, made fail-fast partial-side-effect behavior explicit in the error path, and widened the command-launch readiness budget. Residual: live smoke uses fake instant agents, so real Codex/Claude cold-start/TUI readiness remains a future robustness target.
- Phase 5C roster/ledger files are Markdown coordination surfaces, not an
  automatic source of truth. Agents still need to keep owners, hcom names,
  related teams, and ledger entries current during work.

## Morning Resume Prompt

```text
Please resume the Limux work from HANDOFF.md. Phase 5A zero-friction protocol discovery, GTK `surface.send_text` readiness/failure reporting, shell-quoted launch snippets, typed-PTY control-character guards, Phase 5B automatic `agent-team` bootstrap, and Phase 5C durable roster/review-ledger seeding are implemented and verified. Host prerequisites are installed, the approved Ghostty/Zig gate built `ghostty/zig-out/lib/libghostty.so`, `./scripts/check.sh`, debug Xvfb smoke, and release Xvfb smoke pass locally. Next implement the reviewer/capture wrapper plus consensus and cross-team broadcast conventions.
```
