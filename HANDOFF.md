# Limux Session Handoff

Last updated: 2026-05-29 20:15 EDT

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

Initial Codex execution attempt status: `BLOCKED BEFORE MUTATION`. The pre-mutation evidence
and apt simulation ran, then execution stopped at `sudo apt-get update` because
sudo required a password. The run was cancelled instead of collecting or
handling a password in chat. No apt package install occurred, and `pkg-config`
is still absent.

Second Codex attempt status: `STILL BLOCKED BEFORE MUTATION`. After the operator ran
`sudo -v` locally, Codex checked `sudo -n true` in its execution context. Sudo
still returned `sudo: a password is required`, which indicates the local sudo
cache did not carry into the Codex PTY/session. No package mutation occurred.

Manual operator execution status: `APT PREREQUISITES INSTALLED`. The operator
ran the approved apt lane manually in a trusted terminal. Post-install checks
show `pkg-config`, `pkgconf`, `libgtk-4-dev`, `libadwaita-1-dev`, and
`libwebkitgtk-6.0-dev` installed. `pkg-config --modversion gtk4 libadwaita-1
webkitgtk-6.0` reports `4.14.5`, `1.5.0`, and `2.52.3`.

Current blocker: the host test now reaches `limux-ghostty-sys` and fails
because `ghostty/zig-out/lib/libghostty.so` is missing. The `ghostty/`
submodule is still uninitialized and `zig` is still not on `PATH`.

Recommended continuation: run a separate reviewed Ghostty/Zig gate. Do not
bundle it into the already-completed apt prerequisite lane. Do not run ad hoc
Zig downloads, submodule init/update, Ghostty build, or system-wide Limux
install without the next explicit review/approval step.

The draft-only Ghostty/Zig mutation review for that next gate is:

```text
docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md
```

Current review decision: `WAIT` until the operator explicitly approves the
exact v2 command block in that file. Current v2 artifact SHA256:
`dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5`.

Consensus gate result: `GO for explicit operator approval; WAIT for execution`.
Reviewers `niru`, `zori`, `kazu`, and the local Claude plugin cleared v2 for
approval consideration. Execution still requires the operator to approve the
exact v2 SHA. The consensus report is:

```text
docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md
```

The v2 lane recommends project-scoped Zig `0.15.2` from official Zig metadata,
SHA256 `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`, the
pinned `am-will/ghostty` submodule commit
`81ab8ffa90185221782baf785e85387321e16f8d`, build evidence under
`docs/evidence/`, and then `CARGO_NET_OFFLINE=true cargo test --locked -p
limux-host-linux surface_send_text_response`.

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
| 2026-05-29 19:24 EDT | Sudo cache follow-up | Operator ran `sudo -v` locally, but `sudo -n true` inside Codex still required a password. No packages were installed. |
| 2026-05-29 19:51 EDT | Manual apt prerequisite completion | Operator manually completed the approved apt prerequisite lane. GTK/WebKit pkg-config checks pass. Host test now fails at the separate Ghostty/Zig gate. |
| 2026-05-29 20:10 EDT | Ghostty/Zig mutation review | Created draft-only review for project-scoped Zig 0.15.2 download, pinned Ghostty submodule initialization, `libghostty.so` build, and host test verification. Decision is `WAIT` pending explicit approval. |
| 2026-05-29 20:15 EDT | Ghostty/Zig consensus gate | `niru`, `zori`, `kazu`, and Claude plugin reviewed v1, returned `WAIT`, v2 was patched, then v2 re-review returned GO for operator approval. Execution remains WAIT until exact v2 SHA approval. |

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
- Host-crate tests moved past the prior `pkg-config` blocker after manual apt install; the active host-crate blocker is now missing `ghostty/zig-out/lib/libghostty.so`.
- `zig` is not on `PATH`, and the `ghostty/` submodule is uninitialized. Review this as a separate supply-chain/build gate before fetching/building.
- Shell-injected launch/bootstrap commands need tests for spaces, quotes, `$`, backticks, semicolons, and newlines before automation expands.
- Instruction-source hashes are deterministic `fnv1a64` metadata for change detection, not cryptographic integrity claims.

## Morning Resume Prompt

```text
Please resume the Limux work from HANDOFF.md. Phase 5A zero-friction protocol discovery is implemented and GTK `surface.send_text` now returns a conflict when the terminal is not ready. Before automatic bootstrap, install/verify host prerequisites (`pkg-config`, GTK dev libraries, Ghostty lib), run host/Xvfb checks, then add shell-quoting tests for launch/bootstrap commands.
```
