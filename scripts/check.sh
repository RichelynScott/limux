#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

cd "$ROOT_DIR"

# Verification commands should be stable when run from inside a Limux pane.
unset LIMUX_WORKSPACE_ID LIMUX_SURFACE_ID LIMUX_PANE_ID LIMUX_TAB_ID LIMUX_SOCKET LIMUX_SOCKET_PATH

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
