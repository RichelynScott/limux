# ORPHAN STAGING MANIFEST — operator destroy list

**Created by:** Claude Code (`huno` · limux lane · session `f1e96226` · Claude Opus 5)
**Date:** 2026-07-29
**Purpose:** Single list the operator can read to destroy staged orphan build
snapshots. Staged under kazu-protocol: **agents `mv` only — the operator alone
destroys.** Nothing here has been deleted.

## Staging root

```
/home/riche/.local/limux-ORPHAN-STAGING-20260729
```

To destroy (OPERATOR ONLY, after reviewing this list):

```bash
rm -rf /home/riche/.local/limux-ORPHAN-STAGING-20260729
```

To restore instead (fully reversible — `mv` back preserving relative paths):

```bash
cd /home/riche/.local/limux-ORPHAN-STAGING-20260729 \
  && find . -mindepth 1 -maxdepth 2 -type d -exec sh -c \
     'mkdir -p "/home/riche/.local/limux-reviewed/$(dirname "$1")"' _ {} \;
```

## Safety verification performed BEFORE staging

| Check | Result |
|---|---|
| Every entry present on disk | 15/15 |
| Referenced by any installed launcher | **0** |
| Tier A `source_sha` identical to live install | confirmed byte-identical |
| Tier E referrer reachable on `$PATH` | **not on PATH** — referrer unreachable |
| All 7 launchers work after staging | 7/7 ok |
| `limux doctor` after staging | fully green |
| Saved profiles readable after staging | `main` + `second` both intact |

> A first verification pass returned a reassuring `live-referenced=0` from a loop
> that had run **once** — zsh does not word-split unquoted parameters the way bash
> does, so it checked a single concatenated bogus path. Re-run with a proper array,
> the real result was also 0. The number was right; the first measurement was not.

## Entries

| Tier | Path (relative to `limux-reviewed/`) | Bytes | Why it is dead |
|---|---|---:|---|
| A | `main-1005f58d-pane-timeout-live-20260716` | 37172325 | same `source_sha` as the LIVE install — provably zero information loss |
| B | `stable/main-1a26bda0-v0.2.3-20260719` | 37183257 | earlier point on the `stable` channel; superseded by live `c757056d2539` |
| B | `stable/main-3bf819f6a949-all5fixes-20260721` | 37271704 | earlier point on the `stable` channel; superseded by live `c757056d2539` |
| B | `stable/main-a5c0f9876b29-omp-scrollfix-20260721` | 37275292 | earlier point on the `stable` channel; superseded by live `c757056d2539` |
| C | `main-acabd3534899-full` | 36883513 | same-SHA pair `acabd3534899`, neither live; ≥1 is a pure duplicate |
| C | `main-acabd3534899-gladfix` | 36908001 | same-SHA pair `acabd3534899`, neither live; ≥1 is a pure duplicate |
| D | `copy-paste-drag-fix-20260622-1014d42` | 157587081 | one iteration series, all superseded; predates the provenance system |
| D | `copy-paste-drag-fix2-20260622-1e87406` | 157548439 | one iteration series, all superseded; predates the provenance system |
| D | `copy-paste-fix-20260622-8897272` | 157601459 | one iteration series, all superseded; predates the provenance system |
| D | `copy-paste-release-autocopy-20260622-29fd2ff` | 157574705 | one iteration series, all superseded; predates the provenance system |
| D | `copy-paste-toast-fix-20260622-4bfae87` | 157548983 | one iteration series, all superseded; predates the provenance system |
| D | `keyboard-modifier-fix-202607010828` | 72445842 | one iteration series, all superseded; predates the provenance system |
| D | `keyboard-ctrlv-hotfix-20260701075537` | 72436750 | one iteration series, all superseded; predates the provenance system |
| D | `resize-stability-60d9603` | 72439726 | one iteration series, all superseded; predates the provenance system |
| E | `raw-key-input-202607010843` | 72546538 | **phantom-live** — referenced only from an archived links dir NOT on `$PATH` |

## Per-tier totals

| Tier | Count | Bytes | GiB |
|---|---:|---:|---:|
| A | 1 | 37172325 | 0.035 |
| B | 3 | 111730253 | 0.104 |
| C | 2 | 73791514 | 0.069 |
| D | 8 | 1005182985 | 0.936 |
| E | 1 | 72546538 | 0.068 |
| **TOTAL** | **15** | **1300423615** | **1.211** |

## Tier E needs a judgment call before destroying

`raw-key-input-202607010843` is staged but is the one entry whose deadness rests on
a **reachability** argument rather than a duplication argument: its only referrer is
`~/.local/bin/archive-limux-links-20260701085347/`, which is not on `$PATH`. That
reasoning is sound but is a different class of evidence from Tiers A–D. If in doubt,
destroy A–D and restore E.

## Not included

The **untiered remainder** (23 snapshots, ~0.71 GiB) is deliberately NOT staged.
Those are unreferenced but I could not justify them by build history, and
"unreferenced" alone is not sufficient grounds under this protocol.

## Context

Staged after the 2026-07-29 VHDX compact reclaimed 150.79 GiB (C: 19.88 → 175.98
GiB free). **This 1.21 GiB is now marginal** — the space emergency it was queued
under is resolved. It is offered as cleanup, not as a space measure, and there is no
urgency to destroying it.
