# TaskMaster Update Ambiguous Result On Review Status - 2026-07-13

Status: Contained; terminal result truncated/ambiguous; affected task parked
Project: `/home/riche/MCPs/limux`
Affected task: `master/15`

## Exact Command

```text
task-master-ai-reviewed update-task --id=15 --tag master --prompt='Preserve title, description, review status, priority, and scope. Add exact source references: docs/future-improvements/limux-runtime-isolation-and-window-ui-plan-20260701.md sections Window Chrome Fixes and Acceptance; merged implementation evidence PR #33. Make the test strategy non-empty and evidence-based: focused GTK host tests/checks, window control visibility for minimize/maximize/fullscreen-windowed behavior, pointer hit testing at each edge/corner, X11 and Wayland compositor caveat recording, and live reviewed-runtime confirmation. State that review may close only when merged Git evidence and live UI acceptance both pass; otherwise retain review with the exact residual.'
```

## Observed Provider And Schema Failures

The main MiniMax provider returned a task with status `review`. The AI response
schema rejected that status even though it is accepted by TaskMaster's status
command and already present in the store:

```text
Invalid option: expected one of "pending"|"in-progress"|"blocked"|"done"|"cancelled"|"deferred"
```

The same response also included `updatedAt`, which the object schema rejected.
MiniMax text fallback then failed schema validation. Ollama structured repair
also failed. The captured output ends exactly as the fallback role begins its
same-provider text/JSON repair. It contains neither the terminal outcome nor an
independent nonzero exit record.

## Later Write After Truncated Terminal Result

Before command:

```text
.taskmaster/tasks/tasks.json SHA-256 7038b2441556f0fbbf7e412075346aaf297e72689091a3469dce4b3b0b5b11b6
```

After command:

```text
.taskmaster/tasks/tasks.json SHA-256 8b35759de60a7e6268dbadae37e7b8f18215234b5afbb23b0f6dc89046fe9989
```

An immediate `task-master-reviewed show 15 --format json` initially showed the
old content, but the later frozen structural comparison proves the command
eventually wrote a provider-derived task. Compared with `HEAD`:

```diff
- "id": 15
+ "id": "15"
+ details: appended the requested source references
+ testStrategy: populated the requested GTK/live acceptance contract
+ subtasks: expanded from 0 to 5 pending subtasks
```

Title, description, status `review`, dependencies, priority, and `updatedAt`
remained unchanged. The already-contained refinement task `5` is the only other
string-ID parent. Because the captured output is truncated while fallback is
still active, this evidence is consistent with a later successful fallback
whose terminal output was omitted. It does not prove a transactional
write-after-failure defect.

Current task-store SHA-256 after the already-running append-note batch completed:
`219de423b704e6d599095372eb9f4b00632cd60582a0b27ac6b61e231446eaf9`.
Those later notes targeted unaffected numeric tasks only and are separately
accounted for; they did not target task 15.

## Transcript Evidence

Transcript:
`/home/riche/.codex/sessions/2026/06/19/rollout-2026-06-19T14-53-19-019ee13a-f948-7080-a37d-20dfad526aa1.jsonl`

- command call ID: `call_fEEZESMGRUdkVHN3I8lGxOYv`
- yielded completion/output call ID: `call_8ODKFQ6NMe9pKJYMICm0Dbgn`
- immediate show call ID: `call_Puf83y2qJvIhTq6tiljDVsNM`

The yielded output contains sanitized provider stderr/stdout through the start
of the final fallback attempt, including MiniMax malformed/repair failure,
schema rejection for `review` and `updatedAt`, and GLM repair failure. It is not
a complete terminal record. No credential values are present in this report.

## Containment

- No manual JSON edit or ID normalization was attempted.
- No further AI update, dependency, expansion, status, or append-note operation
  will target `master/15` until the TaskMaster manager provides a supported
  repair.
- Source and verification requirements may be preserved through literal
  `append-note` only if the manager confirms that operation is safe for the
  affected string-ID task; until then task 15 is read-only.
- All TaskMaster mutation is currently stopped under manager containment; only
  read-only inventory and independent docs/skill drafting continue.

## Source-Fix Requirements

1. The AI task schema must accept every TaskMaster status accepted by
   `set-status`, including `review`, or the update path must preserve valid
   existing statuses outside provider output.
2. Provider execution must expose an unambiguous terminal result and exit code;
   output capture must not omit the final fallback verdict.
3. CLI string IDs must be normalized to the stored numeric ID before any
   preservation write.
4. Add tests for CLI-string IDs, existing `review` status, provider/fallback
   schema failure, complete terminal observability, and no-write-on-proven-failure.
