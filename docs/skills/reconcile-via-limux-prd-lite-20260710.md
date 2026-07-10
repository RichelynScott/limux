# PRD-Lite: Reconcile Via Limux Skill

Date: 2026-07-10
Owner: Limux project manager lane
Classification: public mechanics; private fleet overlays remain global-only
Status: draft for cross-runtime global-config review

## Goal

Provide a repeatable, no-loss workflow for reconciling a dirty primary checkout
and multiple Git worktrees by combining live Limux topology, hcom identity and
resume, GitHub branch/PR evidence, TaskMaster, and exact-path ownership.

## Scope

- Read-only state snapshot across Git, hcom, Limux, GitHub, and TaskMaster.
- Exact dirty-path and worktree owner classification.
- Selection of only the sessions whose ownership is required.
- Explicit healthy-surface resume through `hcom r` inside Limux.
- hcom-delivered bounded cleanup briefs after identity verification.
- Fresh-base branch/PR preservation and no-loss worktree closure.
- Global promotion through dual manager inboxes and runtime-owned mirrors.

## Non-Goals

- No replacement for `dirty-worktree-owner-cleanup`, `hcom`,
  `limux-use-guide`, `github-cli`, or `git-worktree-hygiene` mechanics.
- No automatic deletion, reset, force removal, force push, merge, install, or
  runtime restart.
- No global live-skill mutation from the public Limux repository.
- No private owner roster, authority aliases, or fleet policy in public skill
  content.
- No independent branch/worktree naming grammar.

## Acceptance

1. Skill validation, Python compile, no-delete static scan, and live read-only
   snapshot smoke pass.
2. A real dirty-worktree reconciliation proves explicit topology checks,
   stale-UUID recovery, safe owner resume, hcom assignment, and final status.
3. Missing transcript and delayed live-delivery cases fail transparently and
   do not trigger repeated guessed resumes.
4. No unique peer/user/TaskMaster/evidence state is lost.
5. Branch/worktree handling composes with the active cross-runtime Git-lane and
   shared-cache/no-loss design.
6. Codex and Claude global-config owners complete a handshake and adversarial
   review before either runtime installs a global mirror.

## Evidence

- Source skill: `skills/reconcile-via-limux/`
- Draft PR: <https://github.com/RichelynScott/limux/pull/50>
- Initial skill commit: `0ead3dd`
- Live dogfood recovered Buro's real transcript after a stale hcom UUID,
  resumed it in an explicit right-side Limux surface, and routed a bounded
  worktree cleanup through hcom.
- The dogfood exposed a delayed endpoint-migration defect now owned by the
  hcom manager lane; that failure is incorporated into the skill safeguards.

## Review Gate

The global-config owners must independently test the snapshot helper, review
the owner-resume and worktree-removal boundaries adversarially, reconcile the
skill with current global Git-lane policy, and return accept/revise/reject.
Global installation remains held until those gates pass.
