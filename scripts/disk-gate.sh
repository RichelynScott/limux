#!/usr/bin/env bash
# scripts/disk-gate.sh - Build-wave discipline for constrained disk windows.
#
# A build-wave is a bounded sequence of Cargo invocations (check/test/build)
# that share one measured disk budget. This script reports the current
# allocation of the shared Limux target/ tree and, when an explicit
# operator-selected threshold is supplied, fails closed BEFORE a build starts
# if the target allocation already exceeds it.
#
# The threshold is NEVER invented here. It is supplied by the operator via
# --max-target-gib (or the LIMUX_TARGET_MAX_GIB environment variable) and is
# the only enforcement this script performs. Without a threshold the script
# is measurement/report-only: it prints the allocation and exits 0.
#
# Usage:
#   scripts/disk-gate.sh --report            # print allocation, exit 0
#   scripts/disk-gate.sh --max-target-gib 6  # fail if target/ > 6 GiB
#   LIMUX_TARGET_MAX_GIB=6 scripts/disk-gate.sh
#
# This script never deletes, moves, or modifies build artifacts.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$("$ROOT_DIR/scripts/cargo-env.sh" --print-target)"

max_gib=""
if [[ ${1:-} == "--max-target-gib" ]]; then
  if (( $# != 2 )); then
    printf '%s\n' "exit 2 - REFUSED: --max-target-gib requires exactly one value" >&2
    exit 2
  fi
  max_gib="$2"
elif [[ -n "${LIMUX_TARGET_MAX_GIB:-}" ]]; then
  max_gib="$LIMUX_TARGET_MAX_GIB"
fi

if [[ -n "$max_gib" ]]; then
  if ! [[ "$max_gib" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    printf '%s\n' "exit 2 - REFUSED: --max-target-gib must be a non-negative number, got '$max_gib'" >&2
    exit 2
  fi
fi

# du -s reports allocated KiB (1 KiB = 1024 bytes). df moves on allocated
# bytes, so this is the honest figure for disk-pressure decisions.
if [[ -d "$TARGET_DIR" ]]; then
  target_kib="$(du -sk "$TARGET_DIR" 2>/dev/null | awk '{print $1}')"
else
  target_kib=0
fi
target_gib="$(awk -v k="$target_kib" 'BEGIN { printf "%.2f", k / (1024 * 1024) }')"

printf 'target_dir: %s\n' "$TARGET_DIR"
printf 'target_allocated_kib: %s\n' "$target_kib"
printf 'target_allocated_gib: %s\n' "$target_gib"

if [[ -n "$max_gib" ]]; then
  # Compare in raw KiB, not the rounded GiB display, so a small target/ can
  # never slip past a tight operator limit through display rounding.
  max_kib="$(awk -v m="$max_gib" 'BEGIN { printf "%d", m * 1024 * 1024 }')"
  if (( target_kib > max_kib )); then
    printf '%s\n' "exit 1 - DISK GATE: shared target/ allocation ${target_gib} GiB exceeds operator-selected limit ${max_gib} GiB; build-wave blocked. Run scripts/target-retention.sh --report for the breakdown." >&2
    exit 1
  fi
  printf 'disk gate: target/ %s GiB within operator limit %s GiB\n' "$target_gib" "$max_gib"
fi

exit 0
