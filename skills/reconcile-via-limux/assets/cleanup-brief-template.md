# Owner Cleanup Brief

Repository: `<absolute project path>`
Owned worktree/paths: `<exact path list>`
Branch and PR: `<branch>` / `<PR URL and state>`
Manager: `<hcom name>`

Mission: classify and reconcile only the paths above. Do not touch peer or
unknown paths.

Definition of done:

1. Classify each dirty item as unique work, generated output, or
   current-main-identical.
2. Preserve unique work through an exact-path commit/PR or durable handoff.
3. Give generated output an authorized disposition without blanket deletion.
4. Report branch, HEAD, upstream, PR state, exact final `git status`, and any
   blocker to normal worktree removal.
5. Stop after reporting; do not merge, install, remove another worktree, or
   mutate the live runtime.
