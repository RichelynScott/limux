# PRD-A: Runtime Trust — Build Identity, `limux doctor`, Log Triage

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:05 UTC
**Purpose:** Kill the installed-vs-running-vs-source drift defect class (≥4 false
bug reports, 2 wrong root-cause hypotheses) by making every build identifiable
and every mismatch detectable in one command.

- **Priority:** P0 (Wave 0 — roadmap W0.1 + W0.4)
- **Dependencies:** none (unblocks PRD-C verify-loop)
- **Effort:** M (Codex-revised from S: 2× build.rs, installer edit, doctor
  with 5 checks + JSON schema + staged-mismatch harness, log-triage
  classifier + fixtures, panic hook, identity in two dispatchers)
- **Execution model:** lifo + subagents; every commit gated by
  `./scripts/check.sh` — gate policy: **no NEW failures vs baseline** (repo
  CLAUDE.md documents a known-failing baseline test,
  `hook_session_id_falls_back_to_transcript_stem`; verify baseline first)
- **Channel targeting:** implement on a preview-channel install; stable untouched until verified

## Problem Statement

Every Limux build ever logged reports `version=0.1.19` (workspace `Cargo.toml`
version, never bumped), and neither `--version` output nor host-log start lines
carry a git SHA or install-id. Consequences documented in FYI.md / handoffs:
the 2026-06-22 "GTK criticals" report was a stale `29fd2ff` build; 2026-06-27
the symlink wasn't updated before restart; 2026-06-29 open windows kept an old
host binary after reinstall; today the operator runs `resize-live-sync-ae26e0a`
while `origin/main` is a ~6,300-line superset. Log evidence cannot be tied to
the build that produced it, and there is no one-shot command that answers
"what am I actually running, and is it stale?"

## Goals

1. Every binary knows and reports its exact build identity (git SHA + dirty
   flag + profile), and every install knows its install-id + channel.
2. One command (`limux doctor`) detects and explains drift: symlink vs running
   host vs socket vs channel.
3. Host log start lines and panics carry build identity; known-benign WSLg
   noise is classifiable mechanically.

## User Stories

### US-1: As the operator, I can see exactly which build I am running
- [ ] `limux --version` prints: crate version, git SHA (short), dirty flag,
      build profile, and — when resolvable from the install root — install-id
      and channel (e.g. `limux-cli 0.1.19 (a1b2c3d, release) install-id=foo-20260707 channel=stable`).
      NOTE: `--version` does not exist today — limux-cli uses a hand-rolled
      arg parser (string match arms, e.g. `"--socket"` at main.rs:179); this
      is a NEW flag added to that parser, not an extension of existing
      version plumbing.
- [ ] `limux-host` logs one start line containing the same identity fields
      before any other output.
- [ ] `system.identify` response gains a `build` object:
      `{"sha":"a1b2c3d","dirty":false,"profile":"release","install_id":"…","channel":"…"}` —
      additive field, no existing keys changed.
- [ ] Identity is embedded at compile time via a `build.rs` in
      `rust/limux-cli/` and `rust/limux-host-linux/` (emit
      `LIMUX_BUILD_SHA`, `LIMUX_BUILD_DIRTY`, `LIMUX_BUILD_PROFILE` env via
      `cargo:rustc-env`); when git is unavailable at build time the value is
      the literal `unknown`, never a fabricated SHA.
- [ ] Install-id + channel are read at runtime from an `install-info.json`
      written by `scripts/user-local-install/install-user-local.sh` into the
      install root, located relative to `current_exe()` (walk up ≤3 levels);
      absent file → fields omitted, no error.

### US-2: As the operator or an agent, I can detect drift in one command
- [ ] New CLI subcommand `limux doctor [--json]` reports, each as
      `ok | warn | fail` with a one-line explanation:
      (a) (Codex-revised — the installer creates CHANNEL-SPECIFIC launcher
      names: legacy `limux`/`limux-cli`, stable `limux-stable*`, preview
      `limux-preview[-<id>]*`, per `install-user-local.sh:201-229,433-434`)
      doctor enumerates ALL `limux*` symlinks in `${prefix}/bin` that resolve
      under `${prefix}/limux-reviewed/`, groups them by channel, and reports
      each launcher's install root. Resolution must walk
      symlink → bash wrapper script (`wrapper_content`,
      install-user-local.sh:345-369) → the `libexec/` binary → install root
      (the symlink target is a wrapper script, not the binary);
      (b) every running Limux host process, discovered by iterating
      `/proc/[0-9]*`, readlink of `exe` (skip on ANY error — EACCES/ENOENT
      races), matching basename `limux-host` OR `limux` whose path contains
      `limux-reviewed/` or `target/`, same-uid only — and whether its binary
      path matches a known install root;
      (c) per discovered channel socket: connectable? and does the host's
      `system.identify.build.sha` match the CLI's own SHA?
      (d) stale socket files that fail to connect;
      (e) Ghostty resource shape of the active install (present/absent —
      shape validation itself stays in PRD-B).
- [ ] A deliberately staged mismatch is detected and reported as `warn/fail`.
      (Codex-revised) Test-harness contract: doctor honors a prefix override
      (`LIMUX_USER_PREFIX`, matching the installer's existing env) so the
      test can stage two fake install roots under a temp prefix, point a
      fake `limux*` symlink→wrapper at the older root, and run a copied
      binary named `limux-host` from the newer root so `/proc/<pid>/exe`
      resolves there. Regression test in `rust/limux-cli/src/main.rs` test
      module or the Xvfb harness.
- [ ] `limux doctor --json` output is stable/parseable (documented schema in
      the subcommand help text).
- [ ] Exit codes (acceptance default; lifo may veto at import): `0` all ok,
      `1` any fail, `2` warns only.
- [ ] Doctor never mutates state (read-only; no socket writes beyond
      `system.ping`/`system.identify`).

### US-3: As a triaging agent, I can classify log noise mechanically
- [ ] `limux doctor --log-triage [--lines N]` reads the host log
      (`~/.local/state/limux/logs/limux-host.log` — NOTE (Codex-revised):
      this is a SINGLE shared path for all channels today, resolved via
      `LIMUX_HOST_LOG_PATH` override else `dirs::state_dir()`, per
      `rust/limux-host-linux/src/main.rs:44-46,257-264`; per-channel log
      paths do NOT exist and are explicitly out of scope for this PRD) and
      classifies each matching line into
      `benign-env | limux-warning | limux-error | unknown` using a
      documented pattern table (EGL/MESA-LOADER/ZINK/dri2, compositor
      popup-remap → `benign-env`; `Gtk-CRITICAL`/`GLib-GIO-CRITICAL` →
      `limux-error`; unrecognized → `unknown`).
- [ ] The pattern table lives in one source file with unit tests per pattern
      class (fixture lines lifted verbatim from the real log).
- [ ] Triage OUTPUT strips ANSI/control bytes from echoed log content
      (subprocess stderr in the log can carry escape sequences — terminal
      escape injection into the operator/agent terminal is the risk).
- [ ] A Rust panic hook in `limux-host` prints the build identity line to
      stderr before the default panic output.

## Functional Requirements

1. `build.rs` for both binaries; SHA via `git rev-parse --short=12 HEAD`
   (aligned with the installer's default install-id SHA length,
   install-user-local.sh:183), dirty via `git status --porcelain` non-empty;
   both tolerate non-git builds. (Codex-required) MUST emit
   `cargo:rerun-if-changed` for the resolved git HEAD/ref paths —
   worktree-aware: `.git` may be a FILE redirecting to
   `.git/worktrees/<name>/HEAD` — and MUST also cover dirty-state inputs:
   the worktree index path (`.git/index` or `.git/worktrees/<name>/index`)
   plus the workspace root `Cargo.toml`/`Cargo.lock` and every workspace
   path-dependency source tree that can feed either binary (`rust/*/Cargo.toml`,
   crate `build.rs` files when present, and crate `src/` trees). If the
   implementation cannot make the dirty flag rerun on all relevant workspace
   source/index changes, the dirty field must be reported as `unknown`, never
   stale `false`. Otherwise incremental rebuilds bake stale SHA/dirty values,
   recreating the exact trust defect this PRD kills.
2. Installer change: `install-user-local.sh` writes `install-info.json`
   (`install_id`, `channel`, `source_sha`, `created_utc`) into the install
   root next to `MANIFEST.md`; add it to the SHA256SUMS file list
   (currently a hardcoded list at install-user-local.sh:493-499; SHA256SUMS
   is written in `--apply` mode only — the dry-run test cannot verify this
   criterion). Malformed (not just absent) `install-info.json` → fields
   omitted + a doctor `warn`.
3. `doctor` dispatch added to the CLI match in `rust/limux-cli/src/main.rs`
   (search anchor: the `"identify" =>` arm) + `print_help()` entry.
4. Host start-line + panic hook in `rust/limux-host-linux/src/main.rs`.
5. No new crates unless already in `Cargo.lock`; process inspection uses
   `/proc` directly (Linux-only is acceptable — Limux is Linux-only).

## Non-Goals

- No auto-update / auto-reinstall behavior (doctor reports, never fixes).
- No log rewriting or subprocess stream re-plumbing (classification only;
  browser-subprocess stream tagging is deferred).
- No version bump policy change (crate version may stay 0.1.19; SHA is the
  identity).
- No Windows/`ps` portability work.

## Technical Considerations

- `system.identify` is served by both the standalone dispatcher
  (`rust/limux-core/src/lib.rs`) and the GTK bridge
  (`rust/limux-host-linux/src/control_bridge.rs`) — the `build` object must be
  added on both paths, sourced from the same const.
- Keep pure logic (pattern table, install-info parsing, drift evaluation)
  separate from I/O so it unit-tests without a display, per
  `docs/maintainability.md`.
- Respect the two-binary naming gotcha (`target/debug/limux` = GTK app;
  installed `limux` = CLI).

## Success Metrics

- Zero future triage incidents where the running build cannot be identified
  from the log alone.
- `limux doctor` run on the operator's machine before/after W0.3 install
  correctly reports the current stale-build condition, then reports clean.

## Testing Instructions

```bash
./scripts/check.sh
cargo test -p limux-cli doctor -- --nocapture
cargo test -p limux-host-linux build_identity -- --nocapture
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh   # extended with a doctor invocation
scripts/user-local-install/install-user-local.sh --dry-run --profile release --install-id prd-a-check
```

## Rollback Plan

Docs + additive code only: revert the PRD-A commits (`git revert`); no
persistent-state migrations. `install-info.json` is ignored by older binaries.

## Open Questions

1. Should `doctor` also compare against the repo checkout's HEAD when run
   inside the repo? (Default: no — repo state is developer concern; keep scope.)
2. Exit codes: propose `0` all-ok, `1` any fail, `2` warns only — confirm with
   lifo at import time.
