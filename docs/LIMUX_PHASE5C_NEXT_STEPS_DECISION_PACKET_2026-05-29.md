# Limux Phase 5C Next Steps Decision Packet

Generated: 2026-05-29 23:32 EDT  
Scope: `/home/riche/MCPs/limux`  
Current pushed commit: `6a73909 feat(cli): seed agent-team roster and review ledger`

## Current State

Phase 5C is complete and pushed. `limux agent-team` now creates the three-file
coordination base for local agent teams:

| File | Role | Safety behavior |
|---|---|---|
| `LIMUX_AGENTS.md` | Generated runtime protocol, current peer surfaces, message format, instruction-source pointers | Marker protected; unmarked files are not overwritten unless explicitly forced |
| `LIMUX_TEAM_ROSTER.md` | Durable ownership, project/team routing, hcom names, related teams, coordination-file pointers | Created if missing; existing files preserved; force only replaces marked Limux rosters |
| `LIMUX_REVIEW_LEDGER.md` | Durable review findings, consensus decisions, accepted risks, and cross-team notification records | Created if missing; never overwritten by `agent-team` |

Verification passed before the Phase 5C commit:

```bash
cargo fmt --check
cargo test -p limux-cli agent_team
cargo test -p limux-cli
cargo clippy -p limux-cli --all-targets -- -D warnings
git diff --check
LD_LIBRARY_PATH="$PWD/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./scripts/check.sh
LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
./scripts/xvfb-smoke-test.sh
```

Claude plugin follow-up review found no commit-blocking issues after fixes.

## Recommended Default

Proceed with **Phase 5D1: Reviewer Workflow Scaffold** before building a full
spawn/capture wrapper.

This is the best next step because it turns the new ledger into a reliable
review workflow without betting everything on real Codex/Claude TUI readiness
or fragile transcript capture in the first pass.

## Roadmap

| Phase | Recommendation | Why |
|---|---|---|
| 5D1 | Reviewer workflow scaffold | Gives every review a durable request file, evidence slot, and ledger entry |
| 5D2 | Reviewer spawn and bootstrap wrapper | Starts a reviewer pane and sends a bounded prompt using the existing Phase 5B path |
| 5D3 | Capture and finalize wrapper | Reads visible output, stores evidence, and updates the ledger |
| 5D4 | Consensus and hcom pointer conventions | Converts reviewer findings into GO/WAIT/NO-GO summaries and targeted hcom messages |
| Later | Machine-readable adapters | Only add JSON/JSONL or `.limux/` adapters if Markdown sidecars are not enough |

## Option A: Phase 5D1 Reviewer Workflow Scaffold

Recommended.

### Proposed CLI Shape

```bash
limux review prepare \
  --artifact <path-or-ref> \
  --reviewer <codex|claude|gemini|opencode|manual> \
  --lens <security|correctness|maintainability|ux|release> \
  --summary <short-review-goal> \
  [--cwd <path>] \
  [--ledger-path <path>] \
  [--reviews-dir <path>] \
  [--dry-run]
```

### What It Would Do

1. Create a review request file such as:

   ```text
   reviews/2026-05-29T2332Z-phase5d1-claude-security.md
   ```

2. Include in that file:

   - artifact under review,
   - requested reviewer and lens,
   - scope boundaries,
   - commands/evidence expected,
   - output format,
   - risk level,
   - ledger path,
   - no-secrets/no-raw-transcript policy.

3. Append a pending entry to `LIMUX_REVIEW_LEDGER.md`.
4. Print the exact prompt or `limux send` payload that should go to the reviewer.
5. In `--dry-run`, print and report the files it would write without touching the host.

### Acceptance Criteria

- Does not launch real agents yet.
- Does not depend on a running Limux host for `--dry-run`.
- Creates review request files atomically.
- Appends ledger entries instead of rewriting the ledger.
- Refuses symlink and non-regular review/ledger paths.
- Rejects control characters in generated prompt text.
- Tests cover request-file creation, ledger append, dry-run, symlink refusal, and malformed arguments.
- Docs show how to use it with the Phase 5C roster and ledger.

### Verification

```bash
cargo test -p limux-cli review
cargo test -p limux-cli agent_team
cargo fmt --check
cargo clippy -p limux-cli --all-targets -- -D warnings
git diff --check
```

## Option B: Full Reviewer Spawn/Capture Wrapper

More ambitious. Do this after 5D1 unless the operator wants a bigger jump.

### Proposed CLI Shape

```bash
limux review run \
  --artifact <path-or-ref> \
  --reviewer claude \
  --lens security \
  --summary "Review Phase 5D diff for commit blockers" \
  [--cwd <path>] \
  [--capture-lines 200] \
  [--no-hcom]
```

### What It Would Do

1. Create the request file and pending ledger entry.
2. Split a reviewer pane with `limux new-pane`.
3. Send the generated review prompt after pane readiness.
4. Capture visible reviewer output with `read-screen`.
5. Save evidence under `reviews/`.
6. Update the ledger entry with result status.
7. Optionally print a short hcom pointer, not raw reviewer output.

### Risks

- Real Codex/Claude TUI readiness can be slower than fake-agent smoke.
- `read-screen` is viewport-oriented and may miss long output.
- It is easy to over-automate consensus before the evidence format is stable.

## Option C: Real-Agent Readiness Smoke

Useful if the next concern is reliability rather than workflow design.

### Goal

Run a bounded smoke with real Codex/Claude panes to prove Phase 5B/5C bootstrap
works outside fake binaries.

### Acceptance Criteria

- Runs in a disposable test repo.
- Does not use real secrets.
- Captures evidence files, not only terminal scrollback.
- Documents cold-start timing and any prompt-readiness failure mode.

## Option D: Consensus And Cross-Team Convention Docs

Lower implementation risk. Useful if you want all teams aligned before new CLI
work.

### Deliverables

- `docs/limux-review-consensus-conventions.md`
- Ledger entry examples for GO, WAIT, NO-GO, accepted risk, and cross-team pointer.
- hcom message examples that reference files instead of pasting long review output.
- Roster routing examples for 4+ projects.

## Recommendation

Pick **Option A**.

It gives the next implementation a narrow scope, strong tests, and a direct
connection to the Phase 5C ledger. It also sets up Options B and D without
requiring a full agent-spawn/capture pipeline immediately.

## Copy-Back Payload

```text
My decisions after reviewing the Limux Phase 5C next-steps packet:

1. Main next step:
   - Proceed with Option A: Phase 5D1 Reviewer Workflow Scaffold.

2. Scope:
   - Add a `limux review prepare` style workflow.
   - Create durable review request files under `reviews/`.
   - Append pending entries to `LIMUX_REVIEW_LEDGER.md`.
   - Print the reviewer prompt or send payload.
   - Do not launch real reviewer panes yet.

3. Follow-up priority:
   - After 5D1, evaluate Option B: full reviewer spawn/capture wrapper.

4. Execution mode:
   - Current Codex session should implement the scoped Phase 5D1 step.

5. Skills:
   - $methodical-modification-protocol
   - $test-driven-development
   - $agent-orchestration
   - $adversarial-review or $adversarial-assessment
   - $code-audit

6. Additional notes:
   - None.

Please proceed according to these selections.
```

## Sources And Evidence

- `HANDOFF.md`
- `FYI.md`
- `README.md`
- `docs/cmux-parity-plan.md`
- `docs/limux-hcom-workflow.md`
- `docs/limux-vs-multica-decision-guide.md`
- Commit `6a73909 feat(cli): seed agent-team roster and review ledger`
