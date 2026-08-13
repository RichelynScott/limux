#!/usr/bin/env bash
# scripts/cargo-env.sh - Resolve the shared Cargo target directory for Limux.
#
# Fleet shared-caches mandate (git-worktree-hygiene.md clause 3): Rust builds
# use one shared target strategy (CARGO_TARGET_DIR) so that the primary
# checkout and any sanctioned repo-local worktree do not each accumulate an
# independent multi-GiB compiler-output tree. This script is the single
# canonical resolver for that directory.
#
# The shared target is ALWAYS the owning repository root's target/ directory,
# resolved through `git rev-parse --git-common-dir` so the answer is correct
# even when invoked from inside a linked worktree. The value is ALWAYS
# absolute, which is what scripts/check.sh's target/target symlink tripwire
# requires (a relative CARGO_TARGET_DIR is what can create that loop).
#
# Usage:
#   scripts/cargo-env.sh --print-target   # print the shared target path
#   scripts/cargo-env.sh --env            # print `export CARGO_TARGET_DIR=...`
#   scripts/cargo-env.sh <cargo args...>  # run cargo with CARGO_TARGET_DIR set
#
# Scripts that invoke cargo should source the env form at the top:
#   eval "$("$ROOT_DIR/scripts/cargo-env.sh" --env)"
#
# This script never deletes, moves, or modifies build artifacts.

set -euo pipefail

refuse() {
  printf '%s\n' "exit 2 - REFUSED: $*; cargo did not run" >&2
  exit 2
}

if ! git_common_dir=$(git rev-parse --git-common-dir 2>/dev/null); then
  refuse "not inside a Limux Git checkout"
fi
if [[ $git_common_dir != /* ]]; then
  git_common_dir="$PWD/$git_common_dir"
fi
if ! git_common_dir=$(cd -P -- "$git_common_dir" 2>/dev/null && pwd -P); then
  refuse "Git common directory '$git_common_dir' is unavailable"
fi
if [[ $(git rev-parse --is-bare-repository 2>/dev/null || true) == "true" ]]; then
  refuse "bare repositories have no shared Limux build root"
fi
if [[ $(basename -- "$git_common_dir") != ".git" ]]; then
  refuse "unexpected Git common directory '$git_common_dir'"
fi

repo_root=$(dirname -- "$git_common_dir")
shared_target="$repo_root/target"

if [[ ${1:-} == "--print-target" ]]; then
  if (( $# != 1 )); then
    refuse "--print-target accepts no additional arguments"
  fi
  printf '%s\n' "$shared_target"
  exit 0
fi

if [[ ${1:-} == "--env" ]]; then
  if (( $# != 1 )); then
    refuse "--env accepts no additional arguments"
  fi
  printf 'export CARGO_TARGET_DIR=%q\n' "$shared_target"
  exit 0
fi

if (( $# == 0 )); then
  refuse "a Cargo subcommand is required"
fi

export CARGO_TARGET_DIR="$shared_target"
exec cargo "$@"
