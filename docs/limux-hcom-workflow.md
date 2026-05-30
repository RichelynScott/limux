# Limux + hcom Workflow Guide

Last reviewed: 2026-05-30
Repo state reviewed: `main` with Phase 5D1 `review prepare` scaffold on top of
Phase 5C durable `agent-team` roster and review ledger seeding

## Executive Summary

Limux is best used as the live local workspace layer for coding-agent teams.
It owns panes, surfaces, workspace layout, same-project message injection,
screen reads, GUI notifications, low-friction `agent-team` peer bootstrap
through generated `LIMUX_AGENTS.md` protocol files, durable
roster/review-ledger sidecars, and file-first review request preparation.

hcom is best used as the cross-session and cross-project coordination bus.
It owns named agent discovery, direct messages across tools, event history,
threads, and communication with teams outside the current Limux workspace.

Use them together like this:

| Need | Use |
|---|---|
| Codex asks Claude in the same project to review a diff | `limux send --surface ...` |
| Parent checks whether a child pane is stuck | `limux read-screen --surface ... --lines 80` |
| Prepare a review without launching a pane | `limux review prepare --artifact <path> --reviewer claude --lens security --summary "..."` |
| Spawn a short-lived reviewer beside the current pane | `limux agent-team --agents codex,claude --cwd "$PWD"` for a paired team, or `limux new-pane --direction right --command 'codex'` plus the prompt from `limux review prepare` for one-off panes |
| Get human attention inside the GUI | `limux notify ...` |
| Tell another project team about a relevant change | `uvx hcom send @agent --intent inform --thread ... --name tipi -- "..."` |
| Preserve decisions, plans, reviews, or handoffs | Write a durable file, then send a Limux or hcom pointer |

## What Limux Is

Limux is a Linux terminal workspace manager built with GTK4, libadwaita, and
embedded Ghostty rendering. The important workflow feature is its Unix socket
control layer. Agents running inside Limux terminals inherit environment
variables that let them identify and control their own workspace without extra
configuration.

Every Limux-spawned terminal inherits:

```bash
LIMUX_WORKSPACE_ID
LIMUX_SURFACE_ID
LIMUX_PANE_ID
LIMUX_TAB_ID
LIMUX_SOCKET
```

That means an agent can:

- identify its current workspace and surface
- split its own pane
- launch another agent CLI in a new pane
- send structured text to another agent's terminal
- read another surface's visible output
- raise a GUI notification for human input

Relevant source references:

- [README agent integrations](../README.md#agent-integrations)
- [Contributor guide](../AGENTS.md)
- [cmux parity plan](cmux-parity-plan.md)
- [Limux A2A skill](../skills/limux-a2a/SKILL.md)

## Mental Model

Use a two-layer operating model:

```text
Human
  |
  | GUI attention, visual workspace, pane observation
  v
Limux workspace per project
  |
  | same-project stdin messages, pane spawns, screen reads
  v
Codex + Claude + temporary reviewers
  |
  | cross-project notices, named-agent messages, threads
  v
hcom
  |
  v
Other project teams
```

Limux should handle immediate local coordination. hcom should handle messages
that need to cross workspace, project, runtime, or machine boundaries.

## Recommended Project Layout

For a normal workload with four or more active projects:

| Scope | Recommendation |
|---|---|
| One project | One Limux workspace rooted at that repo |
| Core pair | At least one Codex pane and one Claude Code pane |
| Orchestrator | One visible lead pane that owns synthesis and user updates |
| Reviewers | Temporary panes or hcom-launched agents with narrow prompts |
| Durable state | Repo files such as `docs/`, `reviews/`, `HANDOFF.md`, or `FYI.md` |
| Cross-team signal | hcom threads with short pointers to durable files |

The project orchestrator should synthesize raw agent output before anything is
sent outside the project team. Do not broadcast raw terminal chatter to other
teams.

## Same-Project Commands

Run these from inside a Limux terminal.

Identify the current pane:

```bash
limux identify --json
```

Launch Claude beside the current pane:

```bash
limux new-pane --direction right --command 'claude'
```

Launch a focused Codex reviewer:

```bash
limux new-pane --direction down --command 'codex'
```

After the pane is created and you have its surface id, send arbitrary prompt
text with `limux send --surface ...` instead of embedding that text inside
`--command`. This keeps prompts containing quotes, `$`, backticks, semicolons,
or newlines out of the child pane's launch shell.

Typed-PTY safety policy:

- `limux send`, `paste-buffer`, `respawn-pane`, `new-pane --command`, and
  `new-workspace --command` reject terminal control characters except tab, LF,
  and CR.
- Multiline `<agent-msg>` envelopes are allowed because they rely on LF.
- Use `limux send-key` for intentional control keys such as Ctrl-C instead of
  embedding ESC, BEL, CSI, NUL, or other control bytes in text.
- Do not use these text paths for ANSI styling, OSC control sequences, binary
  payloads, or terminal escape experiments.

Read a child pane:

```bash
limux read-screen --surface "$child_surface" --lines 80
```

Send a structured message to a peer:

```bash
limux send --surface "$peer_surface" $'<agent-msg from="codex" to="claude" id="'"$(uuidgen)"'" ts="'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'">\n<request>Review the latest diff and report blocking issues only.</request>\n</agent-msg>\n'
limux send-key --surface "$peer_surface" enter
```

Notify the human:

```bash
limux notify \
  --subtitle "review needed" \
  --body "Consensus conflict in project X" \
  "Input needed"
```

## Cross-Project hcom Pattern

Use hcom when a message needs to leave the current Limux workspace.

List available sessions:

```bash
uvx hcom list --name tipi
```

Send a non-blocking update to another project lead:

```bash
uvx hcom send @target \
  --intent inform \
  --thread project-x-auth \
  --name tipi \
  -- "Project X changed shared auth assumptions. See docs/decisions/auth-2026-05-28.md."
```

Ask for a reply:

```bash
uvx hcom send @target \
  --intent request \
  --thread shared-release-risk \
  --name tipi \
  -- "Please check whether this release assumption affects your project. Details: reviews/release-risk-2026-05-28.md"
```

Keep hcom messages short. Put long findings, review reports, plans, and
handoffs in files first.

## Subagent And Review Pattern

Use this sequence for adversarial review and consensus:

1. The project orchestrator writes or identifies the artifact under review.
2. Run `limux review prepare` to create a file-backed request and pending
   ledger entry before sending prompt text to any reviewer.
3. Spawn reviewers as Limux panes if they only need the current repo.
4. Use hcom if the reviewer lives in another runtime, project, or machine.
5. Require reviewers to write file-backed findings or leave clear pane output.
6. The orchestrator reads outputs, resolves conflicts, and writes a synthesis.
7. If another project is affected, send only the synthesis pointer through hcom.

Prepare a review request:

```bash
limux review prepare \
  --artifact <path-or-ref> \
  --reviewer claude \
  --lens security \
  --summary "Review this diff for blocking issues"
```

This writes `reviews/<review-id>.md`, appends a pending entry to
`LIMUX_REVIEW_LEDGER.md`, and prints the exact reviewer prompt. It does not
launch or message a reviewer pane yet. Use `--dry-run` when you want to inspect
the planned request, ledger entry, and prompt without writing files.

Good reviewer prompt shape:

```text
Review this artifact for blocking issues only.

Scope:
- Files: <paths>
- Question: <specific decision>
- Output: findings with file:line references, or PASS

Do not rewrite code unless asked.
Leave the final answer visible in your pane.
Parent surface if blocked: <surface-id>.
```

## Safety Rules

Use these defaults across all teams:

- Limux messages are for short same-project coordination.
- hcom messages are for named cross-session coordination.
- Durable files are the source of truth for plans, decisions, handoffs, and reviews.
- Never rely on terminal scrollback as the only record of a decision.
- Do not send more than about 200 lines through `limux send`; write a file and send the path.
- Do not broadcast raw reviewer output across projects; synthesize first.
- Ask before destructive operations, infrastructure changes, production data access, or security-sensitive changes.

## Important Limux Caveat

Be careful with:

```bash
limux agent-team --cwd "$PWD"
```

The current implementation writes generated runtime protocol to
`LIMUX_AGENTS.md` in the shared cwd by default, then launches peer agent panes
and seeds `LIMUX_TEAM_ROSTER.md` plus `LIMUX_REVIEW_LEDGER.md` when missing.
It sends each peer a short bootstrap prompt after those coordination files are
written. This is safer than the previous `AGENTS.md` behavior and protects
load-bearing repo instructions.

Important remaining rule: do not treat `LIMUX_AGENTS.md` as an inherited or
merged copy of `AGENTS.md`. Repo instruction files such as `AGENTS.md`,
`CLAUDE.md`, and `GEMINI.md` remain authoritative. `LIMUX_AGENTS.md` should
only describe the Limux runtime team, messaging protocol, coordination-file
pointers, and operator-escalation rules.

`LIMUX_AGENTS.md` now includes an `Instruction Sources` section that points to
detected `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` files with path, mtime, and
hash metadata. Existing unmarked `LIMUX_AGENTS.md` files are protected by
default; use `--force-protocol-overwrite` only when replacing one is intentional.

Use `--protocol-path <path>` when you want the generated protocol somewhere
other than the shared cwd. Use `--no-bootstrap` when you want the panes launched
without the post-launch prompt.

Durable coordination files:

- `LIMUX_TEAM_ROSTER.md` maps projects, agents, hcom names, owners, related
  teams, routing rules, and coordination-file paths. Live workspace/pane/surface
  IDs stay in the current generated `LIMUX_AGENTS.md` runtime protocol so the
  durable roster does not become stale routing data.
- `LIMUX_REVIEW_LEDGER.md` is the durable place for reviewer findings,
  consensus decisions, accepted risks, unresolved risks, and cross-team
  notifications.
- Existing roster and ledger files are preserved by default. Use
  `--roster-path <path>` or `--ledger-path <path>` for alternate files.
  `--force-roster-overwrite` intentionally resets only marked Limux roster
  files; the ledger remains create-if-missing only.
- `--dry-run` does not contact a Limux host, but it still materializes the
  protocol and seeds missing roster/ledger files. Use temporary output paths for
  preview-only runs.

## Suggested Operating Cadence

Start of project session:

1. Open the repo in its own Limux workspace.
2. Start one lead Codex or Claude pane.
3. Start the paired peer pane.
4. Run `uvx hcom list --name tipi` to see active cross-project peers.
5. Record any project-specific coordination file paths.

During execution:

1. Use Limux for local peer messages and pane observation.
2. Use temporary panes for focused review or implementation tasks.
3. Use hcom only for cross-project dependencies or explicit external review.
4. Write durable summaries before notifying other teams.

End of task:

1. Run the repo quality gate.
2. Write or update the handoff/review summary.
3. Send hcom pointers only to teams that need the information.
4. Use `limux notify` for human attention when a decision is needed.

## Best Next Improvements

The highest-leverage Limux improvements for this workflow are:

1. Add Phase 5D2 full reviewer spawn/capture wrapper. This should start a
   reviewer pane, send the `review prepare` prompt after readiness, capture
   evidence to `reviews/`, update the ledger, and print only short hcom
   pointers.
2. Add a `review collect` or `review complete` path that records reviewer
   verdicts back into the existing ledger entry without rewriting unrelated
   content.
3. Add documented conventions for consensus reports and cross-team broadcasts:
   GO, WAIT, NO-GO, accepted risk, unresolved risk, and targeted hcom pointer
   examples.
4. Add optional runtime-specific `.limux/` adapters for Codex, Claude Code,
   Gemini, and OpenCode if direct protocol discovery needs deeper integration.
5. Add a later machine-readable roster/ledger adapter if Markdown sidecars are
   not enough for automation.

## Current Prime Snapshot

As of this review:

- `agent-team` writes `LIMUX_AGENTS.md` by default and supports
  `--protocol-path`.
- `agent-team` seeds `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md` when
  missing and supports `--roster-path`, `--ledger-path`, and
  `--force-roster-overwrite`.
- Existing repo `AGENTS.md` files are not written by default.
- Generated `LIMUX_AGENTS.md` files include a generated marker, instruction
  sources, durable roster/ledger pointers, sidecar policy guidance, and
  no-overwrite protection for unmarked sidecars.
- Live `agent-team` launches peer panes with bare agent commands, writes the
  protocol, roster, and ledger before bootstrap, then submits a sanitized
  one-line bootstrap prompt with explicit Enter. `--no-bootstrap` and
  `--no-launch` skip prompt sends.
- `limux review prepare` creates durable review request files, appends pending
  `LIMUX_REVIEW_LEDGER.md` entries, supports dry-run planning, and refuses
  existing requests, leaf symlink/non-regular/overlapping targets, and
  control-character prompt fields. Use trusted output directories; parent path
  components are not recursively audited for symlinks.
- Generated `new-pane --command` examples quote launch commands and avoid
  nested arbitrary prompt text.
- Typed terminal text now rejects ESC/BEL/C1/control payloads except tab, LF,
  and CR across CLI, bridge/core, and host send-sink paths.
- Latest tag is `v0.1.19`.
- Full `./scripts/check.sh` and `./scripts/xvfb-smoke-test.sh` pass locally
  with the local Ghostty library; the smoke script now exports the required
  `LD_LIBRARY_PATH` automatically when `ghostty/zig-out/lib` exists.
- The next recommended implementation is the Phase 5D2 reviewer spawn/capture
  wrapper plus documented consensus/cross-team broadcast conventions, tracked from
  [`cmux-parity-plan.md`](cmux-parity-plan.md).
