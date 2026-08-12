#!/usr/bin/env bash
# scripts/target-retention.sh - Non-destructive retention report for the
# shared Limux target/ tree.
#
# Fleet shared-caches mandate + archive-not-delete: build artifacts are
# regenerable, but this script NEVER deletes, moves, or modifies anything.
# It reports the allocation breakdown so an operator (or a reviewed
# retention policy) can decide what to reclaim. The only write it performs
# is the optional --report-file output, which is a plain text report.
#
# Usage:
#   scripts/target-retention.sh --report            # print breakdown
#   scripts/target-retention.sh --report --report-file /tmp/limux-target-report.txt
#
# Exit codes: 0 = report produced. No enforcement is performed here.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$("$ROOT_DIR/scripts/cargo-env.sh" --print-target)"

report_file=""
if [[ ${1:-} == "--report" ]]; then
  shift
  if [[ ${1:-} == "--report-file" ]]; then
    if (( $# != 2 )); then
      printf '%s\n' "exit 2 - REFUSED: --report-file requires exactly one path" >&2
      exit 2
    fi
    report_file="$2"
  elif (( $# != 0 )); then
    printf '%s\n' "exit 2 - REFUSED: unknown argument '$1'" >&2
    exit 2
  fi
else
  printf '%s\n' "exit 2 - REFUSED: usage: scripts/target-retention.sh --report [--report-file <path>]" >&2
  exit 2
fi

emit() {
  if [[ -n "$report_file" ]]; then
    printf '%s\n' "$1" >> "$report_file"
  else
    printf '%s\n' "$1"
  fi
}

if [[ -n "$report_file" ]]; then
  : > "$report_file"
fi

emit "limux target/ retention report"
emit "generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
emit "target_dir: $TARGET_DIR"

if [[ ! -d "$TARGET_DIR" ]]; then
  emit "target_allocated_kib: 0"
  emit "target_allocated_gib: 0.00"
  emit "note: shared target/ does not exist yet"
  exit 0
fi

target_kib="$(du -sk "$TARGET_DIR" 2>/dev/null | awk '{print $1}')"
target_gib="$(awk -v k="$target_kib" 'BEGIN { printf "%.2f", k / (1024 * 1024) }')"
emit "target_allocated_kib: $target_kib"
emit "target_allocated_gib: $target_gib"

# Breakdown by top-level profile directory (debug/release), then by the
# dominant Cargo subdirectories inside each profile.
for profile in debug release; do
  if [[ -d "$TARGET_DIR/$profile" ]]; then
    profile_kib="$(du -sk "$TARGET_DIR/$profile" 2>/dev/null | awk '{print $1}')"
    profile_gib="$(awk -v k="$profile_kib" 'BEGIN { printf "%.2f", k / (1024 * 1024) }')"
    emit "profile/$profile: ${profile_gib} GiB (${profile_kib} KiB)"
    for sub in deps incremental build; do
      if [[ -d "$TARGET_DIR/$profile/$sub" ]]; then
        sub_kib="$(du -sk "$TARGET_DIR/$profile/$sub" 2>/dev/null | awk '{print $1}')"
        sub_gib="$(awk -v k="$sub_kib" 'BEGIN { printf "%.2f", k / (1024 * 1024) }')"
        emit "  $sub: ${sub_gib} GiB (${sub_kib} KiB)"
      fi
    done
  fi
done

emit "note: report-only; no artifacts were deleted, moved, or modified"
exit 0
