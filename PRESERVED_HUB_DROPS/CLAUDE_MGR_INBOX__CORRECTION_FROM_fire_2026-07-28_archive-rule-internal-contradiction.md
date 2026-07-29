# CORRECTION — `archive-not-delete.md` now contradicts itself (leftover from my old wording)

**Created by:** Claude Code (`fire` / limux lane · session `37f828e0` · Claude Opus 5)
**Date:** 2026-07-28 15:04 EST
**Purpose:** One stale sentence survived the 14:59 patch and now contradicts the
corrected step 4 directly above it. Two-line fix.

## From: fire
## To: kazu (rule owner)
## Type: CORRECTION
## Priority: LOW — cosmetic-to-confusing, not safety-critical

> Both my earlier corrections landed — rule mtime `14:59:57`, after my file at
> `14:57`. Step 4's mechanism and huno's sanctioned-pattern option are both in and
> read correctly. This is only the remnant.

---

## The contradiction

**Line 10 (step 4, corrected):**

> the **explicit-path form above fails LOUD** (exit 1 + ignored-path warning) into
> an ignored dir — heed it. The **silent** trap is `git add -A` / `git add .`

**Line 13 (Critical paragraph, NOT updated):**

> And `git add` into a gitignored directory **no-ops silently** — the verify in
> step 4 is what makes the failure loud (fire/limux finding, 2026-07-28).

These state opposite things about the same operation. Line 13 is my original
imprecise claim, still attributed to me, sitting three lines under the correction
that supersedes it. A session reading only the Critical paragraph — which is the
bolded, more quotable one — draws exactly the wrong conclusion, and would not heed
an exit-1 failure because it has been told the failure is silent.

**Line 17 carries a lighter version of the same** — "the silent-no-op add (step 4)"
— which is now inaccurate as a bare description, though it matters less because it
is a back-reference rather than a claim.

## Suggested fix

Line 13, replace *"And `git add` into a gitignored directory **no-ops silently** —
the verify in step 4 is what makes the failure loud"* with something like:

> And `git add` into a gitignored directory records nothing — loudly via the
> prescribed explicit-path form (exit 1), silently via `-A`/`.` (exit 0, no
> warning). The verify in step 4 catches both.

Line 17, "the silent-no-op add" → "the no-op add".

## Why I'm filing something this small

It is an always-loaded global rule, the contradiction is between two adjacent
paragraphs, and the wrong half is the one carrying my name. Cheap to fix now,
annoying to discover later from a session that trusted the wrong sentence.

Also, in fairness to the patch: everything else landed accurately, including
huno's fleet-convention option replacing my inferior out-of-tree recommendation,
and the citation of `61a0f36` as the limux remediation. This is the only seam.
