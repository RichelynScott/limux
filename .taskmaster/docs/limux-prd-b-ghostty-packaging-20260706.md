# PRD-B: Ghostty Resource + Terminfo Packaging Correctness

**Created by:** Claude Code (nato · Claude Fable 5)
**Date:** 2026-07-06 23:10 UTC
**Purpose:** Ship valid Ghostty runtime resources (shell integration + compiled
terminfo) with every user-local install, and make the installer fail loudly on
invalid resource shape — closing the packaging gap behind the severest
user-facing regression to date (terminal typing corruption, 2026-06-29→07-01).

- **Priority:** P0 (Wave 0 — roadmap W0.2)
- **Dependencies:** none (PRD-A doctor reports resource presence; this PRD owns shape validity)
- **Effort:** S/M
- **Execution model:** lifo + subagents; `./scripts/check.sh` gate per commit
- **Channel targeting:** preview-channel install first

## Problem Statement

The 2026-06-29 install lineage regression (`docs/terminal-input-regression-20260701.md`)
was caused by packaging: commit `60d9603` relaxed Ghostty resource discovery,
the installer copied `ghostty/src` (source material — shell integration but no
compiled sibling terminfo) as runtime resources, and prompt control bytes
rendered as `?` with corrupted typed text despite clean GTK key events. The
source-side fix (`93f4d3e`, merged in PR #6) makes the host **reject**
source-only resource shapes — but the current install manifest
(`resize-live-sync-ae26e0a/MANIFEST.md`) records
`Ghostty resources: not found / Ghostty terminfo: not found`: installs now ship
**no resources at all**. Shell integration and `xterm-ghostty` terminfo remain
unshipped, and the installer treats "nothing found" as a silent pass.
`LIFO_HANDOFF.md` explicitly marks terminfo generation as an unsolved separate
packaging task.

## Goals

1. Every fresh install contains a **valid** Ghostty resource bundle:
   `shell-integration/` plus compiled sibling `terminfo/` containing
   `xterm-ghostty`.
2. The installer self-verifies resource shape and **aborts loudly** on
   invalid/absent resources unless the operator explicitly opts out.
3. CI asserts the manifest invariants so this class cannot silently regress.

## User Stories

### US-1: As the operator, a fresh install gives me working shell integration and terminfo
- [ ] After `install-user-local.sh --apply`, the install root contains
      `share/ghostty/shell-integration/` and `share/ghostty/terminfo/` with a
      compiled `xterm-ghostty` entry.
- [ ] `infocmp -A <install-root>/share/ghostty/terminfo xterm-ghostty` exits 0.
- [ ] The install `MANIFEST.md` records `Ghostty resources: <path>` and
      `Ghostty terminfo: found` (exact strings kept grep-stable), plus the
      resource origin (which build path produced them).
- [ ] A pane opened in the installed build gets `TERM=xterm-ghostty` resolved
      successfully (no `?` prompt-byte corruption) — verified via the Xvfb
      smoke harness reading the pane's `echo $TERM` output.
- [ ] `SHA256SUMS` covers the shipped resource files.

### US-2: As the operator, the installer refuses to produce a known-bad install
- [ ] If no valid resource bundle can be produced/located, `--apply` aborts
      with a clear error naming the expected shape and the documented build
      step — it does NOT complete with "resources: not found".
- [ ] An explicit `--allow-missing-ghostty-resources` flag preserves the old
      behavior for emergencies, and stamps the manifest with a visible
      `DEGRADED: no ghostty resources` marker.
- [ ] A source-only shape (`ghostty/src`-style: shell-integration without
      compiled terminfo) is rejected by the installer with the same loud
      error (mirroring the runtime check from `93f4d3e`).
- [ ] `--dry-run` reports which resource path WOULD be used and its shape
      verdict.

### US-3: As a maintainer, CI catches packaging regressions
- [ ] A test (script under `scripts/tests/`, wired into CI like
      `validate-split-icons.sh`) builds or stages a resource bundle and
      asserts: shell-integration present, compiled terminfo present,
      `infocmp` succeeds, manifest strings present.
- [ ] A regression test encodes the `ghostty/src` failure class: given a
      staged source-only directory, the installer's shape check rejects it.
- [ ] `docs/terminal-input-regression-20260701.md` gains a closing note
      pointing at the new guardrails (append, don't rewrite history).

## Functional Requirements

1. **Resource production** — decide at implementation time, in this order of
   preference, and document the choice in the PRD's implementation notes:
   a. If a vendored-Ghostty build output already exists
      (`ghostty/zig-out/share/ghostty/`), stage from it (current installer
      already prefers this path — keep).
   b. Otherwise produce a minimal valid bundle without a full Ghostty build:
      copy `shell-integration/` from the vendored source tree AND compile
      terminfo standalone with `tic -x` from Ghostty's terminfo source into
      `terminfo/` (this yields the exact valid shape the runtime check
      requires). The vendored `ghostty/` tree itself remains read-only —
      all outputs are staged into the install root or a build dir.
   c. Full `zig build` of vendored Ghostty is allowed but NOT required by
      this PRD (prior consensus gates on zig mutation work apply —
      `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md`).
2. **Shape validation function** shared, not duplicated: extract/reuse the
   runtime validation logic from `rust/limux-host-linux/src/main.rs`
   (`resolves_shell_integration_*` test lineage) OR implement the identical
   contract in the installer script with the contract documented in both
   places referencing each other.
3. Installer edits confined to `scripts/user-local-install/install-user-local.sh`
   (+ new helper under `scripts/user-local-install/` if needed).
4. `tic` availability: check at install time; absent `tic` → same loud abort
   path with install instructions (`ncurses-bin`).

## Non-Goals

- No changes to runtime resource-discovery logic in the host (already fixed
  in `93f4d3e`); this PRD is packaging-side only.
- No distro packaging (`package.sh` / AppImage / PKGBUILD / RPM) changes —
  user-local installer only; distro parity is a follow-up.
- No vendored `ghostty/` source modifications (read-only invariant).
- No font/glyph handling (Nerd Font `?`-glyph symptoms are environmental —
  documented split in `LIFO_HANDOFF.md`).

## Technical Considerations

- The runtime check (post-`93f4d3e`) requires shell-integration AND compiled
  sibling terminfo — the installer's definition of "valid" must match it
  exactly, or installs will pass installer validation and still be rejected
  at runtime.
- WSL2 note: `tic` writes hashed-db or directory-tree terminfo depending on
  ncurses build; use `tic -x -o <dir>` to force the directory form the
  runtime expects.
- Keep the manifest strings stable — PRD-A's `doctor` greps them.

## Success Metrics

- Fresh preview install shows valid resources in manifest; `infocmp` passes.
- Zero recurrence of the `?`-prompt/corrupted-typing class attributable to
  resource shape.

## Testing Instructions

```bash
./scripts/check.sh
bash scripts/tests/validate-ghostty-resources.sh          # new
scripts/user-local-install/install-user-local.sh --dry-run --profile release --install-id prd-b-check
scripts/user-local-install/install-user-local.sh --apply  --profile release --install-id prd-b-check
infocmp -A ~/.local/limux-reviewed/prd-b-check/share/ghostty/terminfo xterm-ghostty
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
```

## Rollback Plan

Installer-script revert (`git revert`); installs made under the new scheme
remain valid. The `--allow-missing-ghostty-resources` escape hatch covers
emergency installs if the resource build path breaks.

## Open Questions

1. Should the staged terminfo also install `ghostty` aliases beyond
   `xterm-ghostty`? (Default: ship exactly what Ghostty's terminfo source
   defines.)
2. Does the operator want `TERM=xterm-ghostty` default, or keep current
   default and only guarantee availability? (Default: guarantee availability;
   TERM policy unchanged this PRD.)
