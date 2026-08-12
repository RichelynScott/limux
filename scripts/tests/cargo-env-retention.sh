#!/usr/bin/env bash
# scripts/tests/cargo-env-retention.sh - Tests for the shared-Cargo-target
# mechanism (scripts/cargo-env.sh), build-wave disk gate
# (scripts/disk-gate.sh), and non-destructive retention report
# (scripts/target-retention.sh).
#
# These tests exercise script behavior only; they never invoke cargo and
# never modify the real repository target/ tree.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
test_root="${TMPDIR:-/tmp}/limux-cargo-env-test-${$}-${RANDOM}"
fixture="${test_root}/repo"

fail() {
    printf 'cargo-env-retention: FAIL: %s\n' "$*" >&2
    exit 1
}

cleanup() {
    rm -rf "$test_root"
}
trap cleanup EXIT

mkdir -p "$fixture"
git -C "$fixture" init -q
git -C "$fixture" config user.email test@example.invalid
git -C "$fixture" config user.name "cargo-env test"
printf 'fixture\n' > "$fixture/README.md"
git -C "$fixture" add README.md
git -C "$fixture" commit -qm init

CARGO_ENV="$repo_root/scripts/cargo-env.sh"
DISK_GATE="$repo_root/scripts/disk-gate.sh"
RETENTION="$repo_root/scripts/target-retention.sh"

# --- cargo-env.sh: --print-target from the main checkout -----------------
target="$(cd "$fixture" && "$CARGO_ENV" --print-target)"
[[ "$target" == "$fixture/target" ]] \
    || fail "print-target from main checkout: expected '$fixture/target', got '$target'"
[[ "$target" == /* ]] \
    || fail "print-target must be absolute, got '$target'"

# --- cargo-env.sh: --print-target from a linked worktree (shared) --------
git -C "$fixture" worktree add -q "$fixture/.worktrees/wt1" -b test-wt 2>/dev/null \
    || fail "could not create fixture worktree"
wt_target="$(cd "$fixture/.worktrees/wt1" && "$CARGO_ENV" --print-target)"
[[ "$wt_target" == "$fixture/target" ]] \
    || fail "print-target from worktree: expected shared '$fixture/target', got '$wt_target'"

# --- cargo-env.sh: --env emits an absolute export -------------------------
env_line="$(cd "$fixture" && "$CARGO_ENV" --env)"
[[ "$env_line" == "export CARGO_TARGET_DIR=$fixture/target" ]] \
    || fail "--env: expected 'export CARGO_TARGET_DIR=$fixture/target', got '$env_line'"

# --- cargo-env.sh: refuses without a subcommand --------------------------
if (cd "$fixture" && "$CARGO_ENV" >/dev/null 2>&1); then
    fail "cargo-env with no arguments must refuse"
fi

# --- cargo-env.sh: refuses unknown flags ---------------------------------
if (cd "$fixture" && "$CARGO_ENV" --bogus >/dev/null 2>&1); then
    fail "cargo-env with unknown flag must refuse"
fi

# --- cargo-env.sh: refuses outside a git checkout -------------------------
outside="${test_root}/outside"
mkdir -p "$outside"
if (cd "$outside" && "$CARGO_ENV" --print-target) >/dev/null 2>&1; then
    fail "cargo-env outside a git checkout must refuse"
fi

# --- disk-gate.sh: report-only mode exits 0 -------------------------------
mkdir -p "$fixture/target/debug"
printf 'x' > "$fixture/target/debug/marker"
(cd "$fixture" && "$DISK_GATE" --report >/dev/null 2>&1) \
    || fail "disk-gate --report must exit 0"
(cd "$fixture" && "$DISK_GATE" --report 2>&1) | grep -q "target_dir: $fixture/target" \
    || fail "disk-gate --report must print the shared target dir"

# --- disk-gate.sh: operator limit above allocation passes -----------------
(cd "$fixture" && "$DISK_GATE" --max-target-gib 100 >/dev/null 2>&1) \
    || fail "disk-gate with a generous operator limit must pass"

# --- disk-gate.sh: operator limit below allocation blocks -----------------
if (cd "$fixture" && "$DISK_GATE" --max-target-gib 0 >/dev/null 2>&1); then
    fail "disk-gate with limit 0 and a non-empty target must block"
fi

# --- disk-gate.sh: limit 0 with no target passes ---------------------------
rm -rf "$fixture/target"
(cd "$fixture" && "$DISK_GATE" --max-target-gib 0 >/dev/null 2>&1) \
    || fail "disk-gate with limit 0 and no target must pass"

# --- disk-gate.sh: invalid limit value refuses -----------------------------
if (cd "$fixture" && "$DISK_GATE" --max-target-gib banana >/dev/null 2>&1); then
    fail "disk-gate with a non-numeric limit must refuse"
fi

# --- target-retention.sh: report-only, no artifact mutation ---------------
mkdir -p "$fixture/target/debug/deps"
printf 'keep' > "$fixture/target/debug/deps/artifact"
(cd "$fixture" && "$RETENTION" --report >/dev/null 2>&1) \
    || fail "target-retention --report must exit 0"
(cd "$fixture" && "$RETENTION" --report 2>&1) | grep -q "no artifacts were deleted" \
    || fail "target-retention --report must state report-only"
[[ -f "$fixture/target/debug/deps/artifact" && "$(cat "$fixture/target/debug/deps/artifact")" == "keep" ]] \
    || fail "target-retention must not modify artifacts"

# --- target-retention.sh: --report-file writes the report ------------------
report_file="$test_root/report.txt"
(cd "$fixture" && "$RETENTION" --report --report-file "$report_file" >/dev/null 2>&1) \
    || fail "target-retention --report-file must exit 0"
[[ -s "$report_file" ]] || fail "target-retention --report-file must write a non-empty report"
grep -q "profile/debug" "$report_file" \
    || fail "target-retention report must include the debug profile breakdown"

# --- target-retention.sh: refuses without --report --------------------------
if (cd "$fixture" && "$RETENTION" >/dev/null 2>&1); then
    fail "target-retention without --report must refuse"
fi

# --- target-retention.sh: refuses unknown arguments -------------------------
if (cd "$fixture" && "$RETENTION" --report --bogus >/dev/null 2>&1); then
    fail "target-retention with an unknown argument must refuse"
fi

printf 'cargo-env-retention: PASS (shared target, disk gate, retention report)\n'
