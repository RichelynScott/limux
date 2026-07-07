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
- [ ] (Codex-revised — layout must match the RUNTIME contract, not a nested
      shape) After `install-user-local.sh --apply`, the install root contains
      `share/limux/ghostty/shell-integration/` AND the **sibling**
      `share/limux/terminfo/` with compiled entry FILES at
      `terminfo/x/xterm-ghostty` (and/or `terminfo/g/ghostty`). This is the
      exact shape `is_ghostty_resources_dir` + `has_ghostty_terminfo` require
      (`rust/limux-host-linux/src/main.rs:60-72`: shell-integration inside the
      resources dir; terminfo as a sibling under the resources dir's parent —
      NEVER nested inside `ghostty/`), and matches the installer's existing
      target layout (`install-user-local.sh:395-414`).
- [ ] `infocmp -A <install-root>/share/limux/terminfo xterm-ghostty` exits 0,
      AND the entry file `share/limux/terminfo/x/xterm-ghostty` exists as a
      regular file (the runtime checks file paths, not infocmp).
- [ ] The install `MANIFEST.md` keeps the existing value shapes
      (`Ghostty resources: <path>`, `Ghostty terminfo: <path>`) and adds two
      new fields: `Ghostty resource shape: valid|DEGRADED` and
      `Ghostty resource origin: <source description>` (origin NEVER goes into
      the `Ghostty resources:` line — the historical invariant "a valid
      install must not say `Ghostty resources: .../ghostty/src`" from
      `docs/terminal-input-regression-20260701.md:89` must keep working as a
      grep).
- [ ] A pane opened in the installed build gets `TERM=xterm-ghostty` resolved
      successfully (no `?` prompt-byte corruption). (Codex-revised) Mechanism:
      the Xvfb smoke harness currently runs the cargo build, not an install
      root — this criterion requires a scoped harness extension that targets
      the install root via the existing `LIMUX_HOST_BIN` override plus a
      `read-screen` assertion on `echo $TERM`. If that extension proves
      out-of-budget, the fallback acceptance is the repo-build proxy (staged
      resources + `GHOSTTY_RESOURCES_DIR` pointed at the staged bundle) with
      the install-root check deferred to the PRD-C live checklist.
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
- [ ] A test script `scripts/tests/validate-ghostty-resources.sh` builds or
      stages a resource bundle and asserts: shell-integration present,
      compiled terminfo entry FILES present, `infocmp` succeeds, manifest
      fields present. (Codex-revised) It is wired into CI as an explicit new
      step in `.github/workflows/rust-quality.yml` — note `validate-split-icons.sh`
      is currently wired into NO workflow, so it is not a precedent; add it
      to the same new step while there.
- [ ] A regression test encodes the `ghostty/src` failure class: given a
      staged source-only directory, the installer's shape check rejects it.
- [ ] `docs/terminal-input-regression-20260701.md` gains a closing note
      pointing at the new guardrails (append, don't rewrite history).

## Functional Requirements

1. **Resource production** — deterministic default, in this order
   (Codex-revised: the vendored tree contains NO tic-consumable terminfo
   source — `ghostty/src/terminfo/` is Zig code; Ghostty's `ghostty.terminfo`
   is generated at build time by running the built binary with `+terminfo`,
   per `ghostty/src/build/GhosttyResources.zig:38-50` — so a source must be
   vendored explicitly):
   a. If a vendored-Ghostty build output already exists
      (`ghostty/zig-out/share/ghostty/`), stage from it (current installer
      already prefers this path — keep).
   b. **Default path:** vendor a pinned `ghostty.terminfo` snapshot into
      Limux at `scripts/user-local-install/resources/ghostty.terminfo` with a
      provenance header (upstream Ghostty version + how it was generated +
      date); at install time `tic -x` compiles it into
      `share/limux/terminfo/`, and `shell-integration/` is copied from the
      vendored source tree (`ghostty/src/shell-integration/` — bash, zsh,
      fish, elvish, nushell). The vendored `ghostty/` tree itself remains
      read-only; all outputs stage into the install root.
      NOTE: the ghostty submodule may be uninitialized in fresh worktrees —
      the installer/brief must run `git submodule update --init ghostty`
      first (checkout-only; this does NOT trigger the zig-BUILD consensus
      gate).
   c. Full `zig build` of vendored Ghostty is allowed but NOT required by
      this PRD (prior consensus gates on zig mutation work apply —
      `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_GATE_2026-05-29.md`; status there is
      "GO for operator approval; execution WAIT").
2. **Shape validation function** shared, not duplicated: the runtime contract
   lives in `is_ghostty_resources_dir` + `has_ghostty_terminfo`
   (`rust/limux-host-linux/src/main.rs:60-72`; current guard test:
   `resource_env_ignores_shell_integration_without_terminfo`, main.rs:785).
   The installer implements the IDENTICAL contract with a comment in both
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
- (Codex-revised) `tic -x -o <dir>` sets the output database LOCATION;
  directory-tree vs hashed-db is fixed by the ncurses build (directory form
  is the Debian/Ubuntu default). The load-bearing guard is asserting
  `<dir>/x/xterm-ghostty` exists as a file AFTER tic, not trusting the flag.
- SHA256SUMS currently hashes a fixed 6-file list apply-only
  (`install-user-local.sh:492-499`); covering a variable resource file set
  requires enumerating (`find share/limux -type f | sort | xargs sha256sum`
  style). Dry-run can never verify SHA256SUMS or the tic compile — the
  dry-run shape verdict is a plan statement, and the PRD accepts that.
- Manifest field contract (`Ghostty resource shape:` / `Ghostty resource
  origin:`) is shared with PRD-A's doctor resource-presence check — PRD-A
  checks shape directly on disk; the manifest fields are the human/audit
  record. Keep both consistent.

## Success Metrics

- Fresh preview install shows valid resources in manifest; `infocmp` passes.
- Zero recurrence of the `?`-prompt/corrupted-typing class attributable to
  resource shape.

## Testing Instructions

```bash
./scripts/check.sh
bash scripts/tests/validate-ghostty-resources.sh          # new
scripts/user-local-install/install-user-local.sh --dry-run --channel preview --profile release --install-id prd-b-check
scripts/user-local-install/install-user-local.sh --apply  --channel preview --profile release --install-id prd-b-check
infocmp -A ~/.local/limux-reviewed/preview/default/prd-b-check/share/limux/terminfo xterm-ghostty
test -f ~/.local/limux-reviewed/preview/default/prd-b-check/share/limux/terminfo/x/xterm-ghostty
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
