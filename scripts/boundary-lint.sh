#!/usr/bin/env bash
# Boundary lint — hcom-convergence DP-7 tripwire (operator-ratified 2026-07-07).
# Changes touching identity/messaging/resume/roster surfaces require an
# HCOM_MGR boundary review before merge. Mechanical gate: the branch must carry
# a `Boundary-Review: hcom` commit trailer when gated surfaces change. Add it
# TRAILER-LAST — after the reviewer clears your current head — as a zero-code
# commit, so the reviewed SHA and the marker coincide. The trailer is coupled to
# history on purpose: a force-push rewrites it away and the gate fails closed,
# demanding fresh review. A PR `boundary-reviewed` label was deliberately NOT
# adopted (and never was DP-7 policy — the packet ratifies the trailer only): a
# label attaches to the PR, not to a commit, so it has no field in which the
# reviewed SHA can be recorded. A marker that cannot express WHICH head was
# reviewed cannot express staleness either — it stays green across force-pushes
# to code no reviewer has seen, which makes it invisible rather than merely weak.
# The trailer's coupling to history is the safety property: rewriting history
# removes it and the gate correctly fails closed. A future label path would need
# an explicit SHA binding. (Defect found + analyzed by huno + levu, DP-7 read by
# tutu, 2026-07-25.)
# Local iteration escape hatch: BOUNDARY_REVIEWED=1 ./scripts/check.sh
# Decision record: docs/LIMUX_HCOM_CONVERGENCE_DECISION_PACKET_2026-07-07.html
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root_dir"

# Surfaces gated by PATH (any change to these files is boundary-relevant).
gated_paths=(
  "rust/limux-cli/src/agent_hooks.rs"
  "rust/limux-host-linux/src/layout_state.rs"
)
# Surfaces gated by CONTENT: other limux-cli changes only trip the gate when
# the diff text touches boundary tokens (protocol generation, rosters,
# ledgers, identity/resume vocabulary).
content_gated_pathspec="rust/limux-cli/src"
content_tokens='hcom|HCOM_|resume|session_id|LIMUX_AGENTS|ROSTER|LEDGER|build_agents_md'

if [[ "${BOUNDARY_REVIEWED:-0}" == "1" ]]; then
  printf 'boundary-lint: skipped (BOUNDARY_REVIEWED=1 override)\n' >&2
  exit 0
fi

if ! base="$(git merge-base HEAD origin/main 2>/dev/null)"; then
  # No origin/main (fork/shallow/standalone checkout): this is a fleet process
  # gate, not a build-correctness gate — skip rather than block outsiders.
  printf 'boundary-lint: skipped (origin/main not resolvable)\n' >&2
  exit 0
fi

# Collect changed files: branch commits since merge-base + staged + unstaged.
changed_files="$( { git diff --name-only "$base"...HEAD -- 2>/dev/null;
                    git diff --name-only --cached -- 2>/dev/null;
                    git diff --name-only -- 2>/dev/null; } | sort -u )"

hits=()
for path in "${gated_paths[@]}"; do
  if grep -qxF "$path" <<<"$changed_files"; then
    hits+=("$path (path-gated)")
  fi
done

# Content gate: scan added/removed diff lines under limux-cli for tokens,
# excluding the path-gated files already reported.
# NOTE: no `grep -q` inside pipelines here — under pipefail, an early -q match
# SIGPIPEs the upstream command and the pipeline falsely reports failure
# (bot-caught on #42). Capture fully, then test.
content_diff="$( { git diff "$base"...HEAD -- "$content_gated_pathspec" 2>/dev/null;
                   git diff --cached -- "$content_gated_pathspec" 2>/dev/null;
                   git diff -- "$content_gated_pathspec" 2>/dev/null; } || true )"
content_hits="$({ grep -E "^[-+][^-+]" <<<"$content_diff" 2>/dev/null \
  | grep -Ev "^[-+]{3}" \
  | grep -E "$content_tokens"; } || true)"
if [[ -n "$content_hits" ]]; then
  hits+=("$content_gated_pathspec/** (content-gated: diff touches ${content_tokens})")
fi

if [[ "${#hits[@]}" -eq 0 ]]; then
  exit 0
fi

# Gated surfaces changed — require the boundary-review trailer on the branch.
# Same pipefail/SIGPIPE hazard as above: capture the log fully, then grep it.
#
# AUTHORING RULE (matters — a blank line silently breaks your marker):
# keep `Boundary-Review: hcom` in the FINAL trailer paragraph, adjacent to the
# papa-git Session-ID/Agent/Protocol-Version block with NO blank line before it.
# A blank line demotes it to body text: `git log --format='%(trailers:key=Boundary-Review)'`
# then returns EMPTY. (papa-git does NOT cause this — it appends adjacent to
# whatever it finds and faithfully preserves the author's paragraph break;
# verified 2026-07-25 on two commits with identical trees c8b7c288, one with the
# blank line demoted and one without parsing correctly.)
#
# WHY THIS IS A RAW GREP, NOT `%(trailers)` / `git interpret-trailers` — a
# deliberate choice, do not "improve" it without reading this:
#   - grep is FAIL-OPEN: a commit body containing a prose line
#     `Boundary-Review: hcom` matches with no review. Accepted for a PROCESS
#     tripwire (a determined actor bypasses a local script anyway; this catches
#     FORGETTING review, not deliberate forgery). Stated honestly, not hidden.
#   - a trailer PARSER is fail-closed and forge-resistant, and it works TODAY for
#     any correctly-authored commit — it is NOT blocked on any papa-git change.
#     The only cost of migrating is historical markers written the blank-line way
#     (a docs + backfill problem, not a cross-lane dependency).
# So a future move to fail-closed parsing is viable; it just is not free, and it
# is a decision, not a cleanup. (Finding: levu 2026-07-25, via %(trailers)
# disagreeing with this grep — two readings of one artifact; the disagreement is
# the tell, and it also caught tutu's initial wrong "papa-git demotes" diagnosis.)
branch_log="$(git log "$base"..HEAD --format=%B 2>/dev/null || true)"
if grep -q '^Boundary-Review: hcom' <<<"$branch_log"; then
  printf 'boundary-lint: gated surfaces changed; Boundary-Review: hcom trailer present\n' >&2
  exit 0
fi

{
  printf '\nboundary-lint FAILED — hcom-boundary surfaces changed without review marker.\n\n'
  printf 'Gated changes detected:\n'
  for hit in "${hits[@]}"; do printf '  - %s\n' "$hit"; done
  printf '\nThese surfaces (identity / messaging / resume / rosters / protocol\n'
  printf 'generation) are gated by the hcom convergence policy (DP-7,\n'
  printf 'operator-ratified 2026-07-07). Before merging:\n'
  printf '  1. Request a boundary review from HCOM_MGR (resolve the live owner\n'
  printf '     via `hcom list mgrs`).\n'
  printf '  2. Record it TRAILER-LAST: after the reviewer clears your current\n'
  printf '     head, add a zero-code-delta commit carrying `Boundary-Review: hcom`\n'
  printf '     as the final commit; the reviewer confirms that head. Verify a\n'
  printf '     zero-delta commit by TREE IDENTITY, not by diff:\n'
  printf '       [ "$(git rev-parse A^{tree})" = "$(git rev-parse B^{tree})" ]\n'
  printf '     A later force-push removes the trailer and this gate fails closed.\n'
  printf '  Local iteration only: BOUNDARY_REVIEWED=1 ./scripts/check.sh\n'
  printf 'Decision record: docs/LIMUX_HCOM_CONVERGENCE_DECISION_PACKET_2026-07-07.html\n\n'
} >&2
exit 1
