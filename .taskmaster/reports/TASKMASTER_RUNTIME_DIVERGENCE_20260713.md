# TaskMaster Runtime Divergence Evidence - 2026-07-13

Status: Preserved for TaskMaster manager triage
Reporter: lifo
Project: `/home/riche/MCPs/limux`
Affected tag: `taskmaster-refinement-20260713`

## Boundary

- No credential values are included.
- No raw TaskMaster package command, MCP, `npx`, or `npm exec` was used.
- No command used `--research` or `--force`.
- Refinement task `5` must not receive status, dependency, update, or expansion
  mutations until the TaskMaster manager clears the string-ID anomaly.

## Exact Commands

Initial parse:

```text
task-master-ai-reviewed parse-prd .taskmaster/docs/limux-taskmaster-refinement-prd-20260713.md --num-tasks=6 --parse-locally --tag taskmaster-refinement-20260713
```

First correction pass:

```text
task-master-ai-reviewed update-task --id=4 --tag taskmaster-refinement-20260713 --prompt='Preserve title, description, status, priority, dependencies, and scope. Correct the command contract in details and test strategy: use task-master-ai-reviewed parse-prd .taskmaster/docs/<prd>.md --num-tasks=<n> --parse-locally --tag <tag>; add --append only when the target tag is non-empty. Never use parse, raw task-master, or --force. Immediately validate with task-master-reviewed list --tag <tag> --with-subtasks --json, sha256sum the task store, and task-master-reviewed validate-dependencies --tag <tag>. Append literal source provenance afterward with task-master-reviewed append-note --id=<id> --text=<literal source pointer> --storage=file --tag=<tag>. Preserve partial-failure and hash-drift stop rules.'
```

```text
task-master-ai-reviewed update-task --id=5 --tag taskmaster-refinement-20260713 --prompt='Preserve title, description, status, priority, dependencies, and scope. Correct the command contract in details and test strategy: complexity analysis is provider-backed and must use task-master-ai-reviewed analyze-complexity --tag <tag> --threshold 5 --output .taskmaster/reports/task-complexity-report_<tag>.json, optionally --research only when architecture or external technical context materially affects decomposition. Review with task-master-ai-reviewed complexity-report --file .taskmaster/reports/task-complexity-report_<tag>.json --tag <tag>. Expansion is provider-backed and must use task-master-ai-reviewed expand --id=<id> --tag <tag> --complexity-report <report>. Targeted expansion is default; never use --force in this pass. Require failedCount == 0 and expandedCount equal intended eligible count. Record atomic skips and reconcile only failed IDs after partial failure.'
```

Second correction pass:

```text
task-master-ai-reviewed update-task --id=5 --tag taskmaster-refinement-20260713 --prompt='Preserve all current content, status, priority, and dependencies. Make exactly one correction in testStrategy: task listing and post-expansion verification are non-AI commands and must use task-master-reviewed list --tag <tag> --with-subtasks --json, never task-master-ai-reviewed list. Keep analyze-complexity, complexity-report, and expand on task-master-ai-reviewed. Do not change the task ID or any other command ownership.'
```

## Model Role Evidence

`task-master-ai-reviewed models` reported:

| Role | Provider | Model |
|---|---|---|
| Main | `ollama` | `glm-5.2` |
| Research | `ollama` | `glm-5.2` |
| Fallback | `minimax` | `MiniMax-M3` |

The parse and all three updates reported `useResearch: false` or omitted the
research path. Their telemetry named provider `ollama`, model `glm-5.2`.

## Wrapper Evidence

The initial parse emitted:

```text
[WARN] Ollama generated malformed JSON, attempting to repair...
[INFO] Successfully repaired Ollama JSON output
```

Each update emitted the same malformed/repair pair, followed by:

```text
[WARN] AI changed task ID. Restoring original ID 4.
```

or:

```text
[WARN] AI changed task ID. Restoring original ID 5.
```

The wrapper did not print the malformed provider payload itself, so no raw
payload is available to preserve. The transcript preserves the exact warning,
repair, telemetry, and ID-restoration messages.

Transcript:
`/home/riche/.codex/sessions/2026/06/19/rollout-2026-06-19T14-53-19-019ee13a-f948-7080-a37d-20dfad526aa1.jsonl`

Relevant call IDs:

- parse: `call_RxtM6OANFyLSvFZBY7qsq0hf`, completion output at the subsequent
  wait result `call_YEpCjU5URnmXdaMhLk9ELhJG`
- first update pair: `call_0DvNU9wESrpUE5RFlsSJVhh0`
- second task-5 update: `call_To5vy9MGy2TV9J6bZ42cnbJt`

## Hashes

| Point | `.taskmaster/tasks/tasks.json` SHA-256 |
|---|---|
| Before refinement parse | `f9c3609a682f5f5921c57b832def7bb308d53b114f2020dd995a1f056d975328` |
| After parse, corrections, provenance notes, inventory completion | `7038b2441556f0fbbf7e412075346aaf297e72689091a3469dce4b3b0b5b11b6` |

The second hash includes legitimate later task-1 and task-2 status/note
operations. It is not presented as a parse-only hash.

## Current Task-5 Shape

`task-master-reviewed show 5 --format json` reports:

```json
{
  "id": "5",
  "status": "pending",
  "dependencies": [2, 4],
  "priority": "medium",
  "subtasks": []
}
```

`task-master-reviewed list --tag taskmaster-refinement-20260713
--with-subtasks --json` also reports task `5` with a string ID and the same
dependencies. `task-master-reviewed validate-dependencies --tag
taskmaster-refinement-20260713` still passes all six tasks and six dependency
edges.

## Current Disposition

TaskMaster manager Sage directed: preserve exact evidence, do not manually
normalize the ID, and avoid status/dependency/update/expand operations targeting
refinement task `5` until source triage clears it. Other tags and unrelated
tasks may continue under the single-writer rule.

After evidence preservation, the per-project model drift was corrected only
through the three documented reviewed commands. Current roles are main
`minimax/MiniMax-M3`, research `ollama/glm-5.2`, and fallback
`ollama/glm-5.2`. The resulting `.taskmaster/config.json` SHA-256 is
`844313ec907fc9e6c3d8cc32aae2e822b312c3ec70fa5b3299fa4007e6dc768b`.
