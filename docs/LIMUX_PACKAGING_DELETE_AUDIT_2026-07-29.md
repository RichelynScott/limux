# Packaging delete-call audit — `scripts/package.sh`

**Created by:** Claude Code (`fire` · limux lane · session `37f828e0` · Claude Fable 5)
**Date:** 2026-07-29 13:35 EST
**Purpose:** Durable record of the `scripts/package.sh` delete-call audit (backlog #12).
The findings reach **end-user machines**, not just this repo, so they are written down
rather than left in a chat transcript. Inventory + judgment only — **no fix is applied
here.** The shipped-behavior change is operator-gated.

## Why this outranks the rest of the 2026-07-29 cleanup

Every other delete-discipline fix this cycle was about our own disk hygiene. This one is
the same defect class — deleting where archiving was correct — **pointed outward at other
people's filesystems, executed as root.**

## The three-scope framing (the whole insight)

`scripts/package.sh` is one file containing three programs that run in different places
with different privileges. Reading it as a single build script hides the problem:

| Scope | Lines | Runs where | Privilege |
|---|---|---|---|
| **BUILD** | 1–460, 666–851 | developer / CI machine | user |
| **INSTALL** | 485–663 | heredoc → generated into the tarball as `install.sh`, shipped to users | **root** |
| **POSTINST** | 703–728 | heredoc → embedded in the `.deb` as `DEBIAN/postinst`, on every `dpkg -i` | **root** |

`scripts/appimage-webkit.sh` (sourced at L214) contains **zero** delete calls.

## Totals

**24 delete invocations: 9 SAFE, 8 RISKY, 7 UNBOUNDED-at-definition.**
Builder-scope deletes are essentially clean — everything lives under `/tmp/limux*`,
`$STAGE`, or `dist/`. **Every genuine violation is in the shipped installer/postinst.**

## The severe findings (source-verified by fire, not taken on report)

### 1. `.deb` postinst deletes in `/usr/local` as root — L718-724

```bash
rm -f /usr/libexec/limux/limux
rm -f /usr/local/libexec/limux/limux            # unconditional, NO guard
if is_legacy_limux_host /usr/local/bin/limux; then rm -f /usr/local/bin/limux; fi
rm -f /usr/share/applications/limux.desktop
rm -f /usr/local/share/applications/limux.desktop
```

A `.deb` maintainer script has no business writing to `/usr/local` — that is Debian Policy,
and `/usr/local` is exactly where a user's **source-built** install lives. An ordinary
`dpkg -i` silently destroys it.

### 2. The legacy-host heuristic is far too broad — L706-714 / L544-555

```bash
is_legacy_limux_host() {
    [ -x "$path" ] || return 1
    help="$("$path" --help 2>&1 || true)"
    echo "$help" | grep -q "limux CLI" && return 1
    echo "$help" | grep -q "GApplication" && return 0
    ...
}
```

Returns **true for any executable whose `--help` mentions `GApplication` and does not say
`limux CLI`** — i.e. essentially any GTK application named `limux`. The script then
**executes** the unknown binary as root before deleting it.

### 3. Generated `install.sh`: one guarded delete path, one unguarded — L565 / L569

`L565` at least probes with `is_legacy_limux_host`. The `elif` at `L569` bypasses the
heuristic entirely and deletes unconditionally across **foreign install prefixes**
(`/usr/local/libexec/...`, `/usr/libexec/...`) — including dpkg/rpm-owned locations.
`$PREFIX` comes from unvalidated `--prefix=` input, so `--uninstall --prefix=/usr` removes
the distro-packaged binary (L608), and `--prefix=` empty yields `/bin/limux`.

## Lesser findings worth fixing

| Line | Issue |
|---|---|
| L442 | `remove_tree "$rpmbuild_dir"` destroys `rpmbuild/BUILD` logs **on the failure path** (L438-440 warns "rpmbuild did not produce expected RPM") — deletes the only diagnostic evidence at the exact moment it is needed |
| L330 | wipes `$OUT_DIR` (`dist/`), which is gitignored — prior-version artifacts, checksums, signatures, and release notes parked there have **no git recovery** |
| L39-41, L600-602 | two duplicated copies of a generic recursive `remove_tree()` with **no path allowlist**; safety is entirely determined by 14 call sites, 4 of which depend on `VERSION` (`$1`, L7) or `PREFIX` (`--prefix=`, L494) |
| L612 | `rm -f /etc/ld.so.conf.d/limux.conf` unconditionally, without verifying it belongs to *this* prefix |
| L614, L642 | delete `limux.desktop` — a filename this installer **never writes** (it writes `dev.limux.linux.desktop`), i.e. removing a foreign/legacy file it did not create |
| L553, L713 | fixed-path root-owned logs `/tmp/limux-{installer,postinst}-probe.log` — classic `/tmp` symlink-follow targets |

## Proposed fix direction (NOT applied — operator-gated)

1. **Drop the `/usr/local` reach from the postinst entirely.** No legitimate case exists;
   it is a policy violation and the highest-severity item.
2. **Give L569 at minimum the same guard L565 has** — unconditional is strictly worse.
3. **Convert legacy-entrypoint removal to RENAME** (`<path>.limux-legacy-<ts>`) rather than
   `rm` — archive-not-delete, pointed outward. A user can undo a rename.
4. **Narrow the heuristic** so it cannot match arbitrary GTK binaries, or drop the
   execute-then-delete pattern altogether.
5. **Add a path-prefix assertion to `remove_tree()`** so the generic deleter cannot be
   pointed outside `/tmp/limux*`, `$STAGE`, `$ROOT_DIR/dist`, or `$PREFIX/*/limux` — this
   bounds all four `VERSION`/`PREFIX`-dependent call sites at once.
6. **L442**: keep `BUILD/` logs on the failure path.

## Provenance and limits

Enumeration + classification by a read-only audit subagent. The two highest-severity
findings (postinst `/usr/local` deletes; the `GApplication` heuristic) were **independently
re-read at source by fire** before being reported or recorded — per
`verify-before-claiming.md`, a subagent's claim is a lead, not a fact.

**Not verified:** no packaging run was executed, no `.deb` was installed, and no delete was
observed firing. This is static analysis. The `is_legacy_limux_host` false-positive breadth
is read from the code path, not demonstrated against a real foreign GTK binary.

**Lane:** limu (owns install/packaging). The write-up is authorized; the shipped-behavior
change requires operator sign-off because it alters what lands on user machines.
