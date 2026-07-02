#!/usr/bin/bash
set -euo pipefail

HERMES_BIN="/home/riche/.local/bin/hermes"
TIMEOUT_BIN="/usr/bin/timeout"
DATE_BIN="/usr/bin/date"

if [[ $# -ne 1 ]]; then
  printf 'Usage: %s <minimax-security|minimax-runtime|minimax-tests|minimax-sequencing>\n' "${0##*/}" >&2
  exit 2
fi

for required in "$HERMES_BIN" "$TIMEOUT_BIN" "$DATE_BIN"; do
  if [[ ! -x "$required" ]]; then
    printf 'Required executable is missing or not executable: %s\n' "$required" >&2
    exit 2
  fi
done

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd -P)"
cd "$REPO_ROOT"

slot="$1"

case "$slot" in
  minimax-security)
    lens="security and socket trust boundary"
    output="$SCRIPT_DIR/hcom-minimax-security.md"
    ;;
  minimax-runtime)
    lens="WSL runtime failures, stale sockets, and multi-runtime targeting"
    output="$SCRIPT_DIR/hcom-minimax-runtime.md"
    ;;
  minimax-tests)
    lens="test strategy and acceptance gates"
    output="$SCRIPT_DIR/hcom-minimax-tests.md"
    ;;
  minimax-sequencing)
    lens="scope discipline, implementation sequencing, and rollout gates"
    output="$SCRIPT_DIR/hcom-minimax-sequencing.md"
    ;;
  *)
    printf 'Invalid slot: %s\nAllowed: minimax-security, minimax-runtime, minimax-tests, minimax-sequencing\n' "$slot" >&2
    exit 2
    ;;
esac

if [[ -z "${MINIMAX_API_KEY:-}" ]]; then
  printf 'MINIMAX_API_KEY is not set. Run only through the reviewed SCRIM exact grant; do not put secrets in argv, config, chat, hcom, or shell history.\n' >&2
  exit 2
fi

prompt="Limux Cursor integration adversarial review.

Read these files:
- docs/reviews/limux-cursor-ide-integration-20260630/REVIEW_BRIEF.md
- docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md
- docs/reviews/limux-cursor-ide-integration-20260630/MANAGER_SYNTHESIS.md

Lens: ${lens}.

Do not edit files. Return concise findings only:
Verdict: PASS | PASS_WITH_CHANGES | WAIT | NO_GO
Top Findings:
1. [P0/P1/P2] finding with exact plan/source reference and concrete fix
Missing Evidence:
- ...
Recommended Plan Changes:
- ...

Keep response under 900 words. Do not include secrets or chain of thought."

{
  printf '# Hermes MiniMax Direct Review\n\n'
  printf 'Slot: %s\n' "$slot"
  printf 'Model: MiniMax-M3\n'
  printf 'Provider: minimax\n'
  printf 'Generated: %s\n\n' "$(TZ=America/New_York "$DATE_BIN" '+%Y-%m-%d %H:%M:%S EST')"
  printf '```text\n'
  set +e
  "$TIMEOUT_BIN" 900 "$HERMES_BIN" chat --provider minimax --model "MiniMax-M3" -Q --max-turns 30 -q "$prompt"
  status=$?
  set -e
  printf '\n```\n\n'
  printf 'Exit status: %s\n' "$status"
} >"$output" 2>&1

exit "$status"
