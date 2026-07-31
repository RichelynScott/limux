# CLAUDE.md — project context for Claude Code

Short, Claude-oriented companion to [`AGENTS.md`](AGENTS.md). For
architecture and the full CLI surface, read `AGENTS.md`. For roadmap
status, read [`docs/cmux-parity-plan.md`](docs/cmux-parity-plan.md).

## What is this project?

Limux is a GTK4 + libadwaita + libghostty terminal workspace manager for
Linux, ported from manaflow-ai's macOS `cmux`. It exposes a Unix-socket
control API so coding agents can drive the GUI from a terminal inside a
limux workspace.

## Before editing

Run the quality gate before *and* after your changes:

```bash
./scripts/check.sh   # fmt --check, clippy -D warnings, test --workspace
```

The gate is **green** on `main` — run it anyway before *and* after, and don't
assume a clean baseline just because this says so.

> **No known-failing tests.** (An earlier flaky-test warning here was resolved by
> PR #82 and removed — full explanation in `HANDOFF.md`. It is deliberately not
> restated here: two separate agents pattern-matched the old test name in this
> paragraph and reported the *resolution note itself* as a live warning.)

## Review checklist — revert the CALL SITE before merging

**Before merging any fix: revert the call site (not the helper), re-run the
suite, and confirm something fails. If nothing fails, the test is decorative.**

Any fix where the **test and the production path reach the same code by
different routes** has this hole by default — the helper test proves the helper
works, nothing proves it is *reached*.

Evidence this is not theoretical (2026-07-21, three lanes, one evening): an
adversarial pass reverted each of five merged limux fixes and **four survived
with a green suite**. The hcom lane then ran the same check on a release it had
already shipped and announced, and found the identical shape. Two decorative
fixes caught across the two repos.

Verified load-bearing in this repo by mutation: H-1 read-screen scoping · M-4
socket fail-open · A1 GUI-hang · send-key `enter`→`Return` · #84 grid predicate ·
`hook_session_id` ordering · #33 build-provenance.

**Escape hatch — do NOT force a timing test.** If a deterministic wiring test
would require a multi-second timing dependency or a runtime refactor with no
injection seam, **file the gap with mutation evidence instead**. A flaky gate
eventually gets used to justify shipping a real failure.

The distinction that matters: a **timeout ceiling** is fine, a **timing
assertion** is not. `sink_failure_does_not_block_stderr_writers_while_read_end_stays_open`
uses `recv_timeout(15s)`, but it returns the instant the writer finishes —
measured at **0.05s across 5 consecutive runs**, a ~300× margin. The 15s elapses
only when the write genuinely blocks forever, which *is* the bug. That is a
bounded failure detector, not a performance claim.

Origin: a standing adversarial subagent — which **died three times and was
nearly abandoned** before the run that found this. The value showed up on the
pass that was hardest to justify continuing.

## Shell mutation checklist — pipe and trap safety

- [ ] **Capture, then filter.** When the command's exit status matters, use
  `OUT=$(cmd); RC=$?`, then filter `OUT`. Never read `$?` after a pipeline as
  the command's status.
- [ ] **Avoid pipeline-status arrays.** Bash uses `${PIPESTATUS[0]}`; zsh uses
  `${pipestatus[1]}`. Both are volatile: the next command used to inspect or
  report them can replace their contents. Capture a scalar immediately, or
  avoid the pipeline.
- [ ] **Install restoration before mutation.** Use `trap 'restore' EXIT`; a
  trailing `restore` line does not cover abort paths.
- [ ] **Make the exit self-describing.** Before `exit "$RC"`, print the action,
  target, and captured status so the restoration backstop is auditable.
- [ ] **Lint the exact script in its target shell.** Run `bash -n` or `zsh -n`
  as appropriate (and `shellcheck` for Bash scripts when available), including
  the trap, restoration, and explicit-exit paths.

## The two-binary gotcha

- `target/debug/limux` — the **GTK app** (`limux-host-linux`). Only
  understands GTK flags. Installed users get this as `limux-host` under
  `libexec`.
- `target/debug/limux-cli` — the **CLI** (`limux-cli`), which implements
  `agent-team`, `notify`, `hooks setup`, `send`, `read-screen`, etc.
  Installed users get this as `limux`.

Run `./target/debug/limux-cli --help` for the full subcommand list —
treat it as the source of truth, not this file.

## Finding code (anchors, not line numbers)

The crates churn, so search by symbol:

```bash
rg -n "fn agent_launch_command|fn build_agents_md" rust/limux-cli/src/main.rs
rg -n "\"agent-team\" =>"                          rust/limux-cli/src/main.rs
rg -n "PaneCallbacks \{"                           rust/limux-host-linux/src/window.rs
```

| Task | Crate / module |
|---|---|
| New agent in `agent-team` | `agent_launch_command` in `rust/limux-cli/src/main.rs` |
| Generated AGENTS.md template | `build_agents_md` in `rust/limux-cli/src/main.rs` |
| New CLI subcommand | dispatch match in `rust/limux-cli/src/main.rs` |
| GUI bridge routing | `rust/limux-host-linux/src/control_bridge.rs` |
| Full-vocabulary control (no GUI) | `limux-core::Dispatcher` + `ControlState` |
| Pane / surface UI state | `rust/limux-host-linux/src/window.rs` (`PaneCallbacks`) |
| Agent-hook installers + templates | `hooks/` + `limux hooks setup` |
| Packaging (AppImage / AUR) | `scripts/package.sh`, `scripts/appimage-webkit.sh`, `PKGBUILD.template` |

## Pitfalls

- **ID mismatch:** host-linux uses `String` workspace ids, `u32` pane id,
  uuid `String` tab id; `limux-core` uses `u64`. Build `LIMUX_SURFACE_ID`
  as `format!("{pane_id}:{tab_id}")`. There is no `SurfaceId` type in
  host-linux.
- **`PaneCallbacks` has one constructor.** Add a field → the compiler
  points you there.
- **Ghostty `env_vars` lifetime:** Ghostty `dupeZ`s keys/values into its
  own arena, so the `Vec<CString>` + `Vec<ghostty_env_var_s>` pattern in
  `terminal.rs::create_terminal` only needs to outlive the
  `ghostty_surface_new` call.
- **Vendored `ghostty/` is read-only.** Work through the C API in
  `ghostty/include/ghostty.h`.
- **Clippy is a hard gate** (`-D warnings`). Fix lints, don't suppress.
- **Don't commit** `target/` or other build artifacts.

## Conventions

- Topic branches: `fix/issue-NN-…`, `feat/…`. Don't rebase shipped
  commits without asking.
- Don't open PRs or issues from inside Claude Code without asking.
- Keep one source of truth per concept (command metadata, launcher maps,
  workspace IDs).
- Split by domain, not vague helpers. Keep pure logic separate from GTK
  wiring where possible.
- Add regression tests when fixing behavior — see `agent_team_tests` at
  the bottom of `rust/limux-cli/src/main.rs` for the expected shape.

## In case of doubt

- **Architecture / full CLI** → `AGENTS.md`
- **Roadmap & phase status** → `docs/cmux-parity-plan.md`
- **Maintainability rules** → `docs/maintainability.md`
- **User install/usage** → `README.md`
- **Inter-agent message format** → the AGENTS.md that `limux agent-team`
  writes into the shared cwd at runtime (not this repo's AGENTS.md).
