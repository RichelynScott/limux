#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

cd "$ROOT_DIR"

# Verification commands should be stable when run from inside a Limux pane.
unset LIMUX_WORKSPACE_ID LIMUX_SURFACE_ID LIMUX_PANE_ID LIMUX_TAB_ID LIMUX_SOCKET LIMUX_SOCKET_PATH

# hcom-boundary tripwire (convergence DP-7) — fast-fails before the cargo gate.
bash "$ROOT_DIR/scripts/boundary-lint.sh"

# Recursive-symlink tripwire. A link inside target/ that points back at target/
# makes `du -L`, `find -L`, `rsync -L`, `tar -h` and naive backup walkers recurse
# until they hang or blow up. Observed 2026-07-28: target/target -> target, a
# 29-byte link costing ~0 space that inflated a disk audit by 1.3 GB and would
# have hung the pre-compact audit. No CARGO_TARGET_DIR is set anywhere in this
# repo, so nothing legitimately creates it.
if [[ -L "$ROOT_DIR/target/target" ]]; then
    printf 'check: recursive symlink loop: target/target -> %s\n' \
        "$(readlink "$ROOT_DIR/target/target")" >&2
    printf 'check: remove it with `rm target/target` (a link, not a tree).\n' >&2
    printf 'check: if a build script sets CARGO_TARGET_DIR, it must be absolute.\n' >&2
    exit 1
fi

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
