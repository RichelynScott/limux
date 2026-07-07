# Limux Terminal Input Regression - 2026-07-01

Author/runtime/date: lifo / Codex gpt-5.5 (xhigh) / 2026-07-01.

## Summary

Limux terminal input corruption reported on 2026-07-01 was isolated to the
June 29 local runtime install path, not to the user's keyboard, shell prompt,
WSLg key delivery, or the later Ctrl+V shortcut experiment.

The confirmed good cutoff is:

- GOOD: `/home/riche/.local/limux-reviewed/lifo-hermes-highlight-cedcb3a`
  from 2026-06-27.

The confirmed bad lineage is:

- BAD: `/home/riche/.local/limux-reviewed/resize-stability-60d9603`
  from 2026-06-29.
- BAD: later diagnostic builds derived from that runtime, including
  `/home/riche/.local/limux-reviewed/raw-key-input-202607010843`.

## User-Visible Symptoms

- Prompt started with visible question marks, for example
  `?➜ ... ?✗`, before the user typed anything.
- Some letters duplicated or transformed during normal typing, while numbers
  and punctuation often behaved normally.
- Earlier autosuggestion/bracketed-paste-looking text could appear during
  space/backspace, but disabling Limux-local zsh autosuggestions was not the
  full fix.

## Evidence

1. GTK key-event logging added in commit `96eee0b` showed one clean physical
   press/release event per typed key and no stuck modifiers while the terminal
   buffer still corrupted text.
2. A temporary `LIMUX_RAW_KEYS=1` diagnostic build bypassed
   `IMMulticontext::filter_keypress`; corruption still reproduced. That ruled
   out GTK IME filtering as the primary cause.
3. Rolling the active symlinks back to the June 27 runtime produced a clean
   prompt immediately and normal physical typing in the same user environment.
4. The June 29 install manifest recorded:

   ```text
   Ghostty resources: /home/riche/MCPs/limux/ghostty/src
   Ghostty terminfo: not found
   ```

   That is source material, not a valid installed runtime resource bundle.

## Root Cause

Commit `60d9603` bundled two unrelated changes into the same install lineage:

- terminal resize coalescing in `rust/limux-host-linux/src/terminal.rs`;
- Ghostty runtime resource resolution/install relaxation in
  `rust/limux-host-linux/src/main.rs` and
  `scripts/user-local-install/install-user-local.sh`.

The resource side accepted a directory when it contained only
`shell-integration`, added `ghostty/src` as a candidate, and copied that source
tree into `share/limux/ghostty` during the user-local install. The installed
runtime then had shell integration files but no compiled sibling terminfo
entries (`g/ghostty` or `x/xterm-ghostty`).

That invalid resource shape made terminal control-sequence behavior unstable:
prompt integration bytes appeared as literal `?` glyphs before typing, and
normal input could render as corrupted terminal text despite clean GTK key
events.

## Fix

The source fix is intentionally narrow:

- `is_ghostty_resources_dir` now requires both `shell-integration` and compiled
  sibling terminfo.
- `ghostty/src` is no longer a runtime resource candidate.
- the user-local installer no longer auto-selects or copies `ghostty/src`.
- regression tests assert source-only shell integration is rejected.
- the temporary `LIMUX_RAW_KEYS` diagnostic path was removed.

Keep the resize-coalescing feature and other workspace/sidebar/highlight work;
the bad part was accepting a source checkout as runtime Ghostty resources.

## Future Guardrail

Before installing a future local runtime, dry-run or inspect the manifest. A
valid install must not say `Ghostty resources: .../ghostty/src`. If Ghostty
resources are bundled, the resolved resources directory must have:

- `shell-integration/`
- a sibling terminfo directory containing `g/ghostty` or `x/xterm-ghostty`

If the built checkout lacks `ghostty/zig-out/share/ghostty` and compiled
terminfo, it is safer for Limux to run without setting `GHOSTTY_RESOURCES_DIR`
than to point Ghostty at `ghostty/src`.

## Closing Note (2026-07-07, PRD-B)

The packaging-side guardrails for this regression class shipped with PRD-B
(Wave 0, roadmap W0.2):

- The user-local installer now stages a valid resource bundle by default:
  `shell-integration/` (from a prebuilt Ghostty share dir, or the vendored
  source tree's `shell-integration/` only) plus a compiled sibling
  `share/limux/terminfo/` produced either by copying prebuilt entries or by
  `tic -x`-compiling the vendored snapshot at
  `scripts/user-local-install/resources/ghostty.terminfo` (provenance in the
  file header; kept in sync with the submodule by
  `scripts/user-local-install/generate-ghostty-terminfo.py --check`).
- The installer aborts loudly when no valid bundle can be staged and rejects
  source-only shapes (`ghostty/src`-style) outright — mirroring the runtime
  contract in `is_ghostty_resources_dir` (`rust/limux-host-linux/src/main.rs`).
  `--allow-missing-ghostty-resources` is the explicit escape hatch and stamps
  the manifest `Ghostty resource shape: DEGRADED` plus a
  `DEGRADED: no ghostty resources` marker.
- Manifests now carry `Ghostty resource shape:` and `Ghostty resource origin:`
  fields; the audit grep above ("must not say `Ghostty resources:
  .../ghostty/src`") keeps working unchanged.
- CI runs `scripts/tests/validate-ghostty-resources.sh`
  (`.github/workflows/rust-quality.yml`), which asserts the staged shape,
  the manifest invariants, SHA256SUMS coverage of shipped resources, and the
  source-only rejection regression test.
