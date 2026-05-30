# Limux Ghostty/Zig Consensus Review Request

Date: 2026-05-29
Requester: `tipi` Codex session in `/home/riche/MCPs/limux`
Thread: `limux-ghostty-zig-gate`

## Review Target

Review this draft-only mutation artifact:

```text
docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md
```

Original reviewed artifact SHA256:

```text
257a522e0690aa5367cf422efa467253ef6f3fc84308150922b3f2a66d820f6b
```

Revised v2 artifact SHA256 after consensus WAIT fixes:

```text
dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5
```

## Goal

Decide whether the proposed Ghostty/Zig command block is safe enough to run as
the next bounded verification gate for Limux.

The active blocker is:

```text
limux-ghostty-sys: libghostty not found
```

The apt prerequisite lane is already complete. GTK/WebKit pkg-config checks
pass. The remaining lane would:

- download official Zig `0.15.2` to `/home/riche/.cache/limux-tools`
- verify Zig SHA256
  `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239`
- initialize only the pinned Ghostty submodule commit
  `81ab8ffa90185221782baf785e85387321e16f8d`
- build `ghostty/zig-out/lib/libghostty.so`
- run `ldd` on the produced shared library
- run `cargo test -p limux-host-linux surface_send_text_response`

## Non-Goals

- Do not execute the command block.
- Do not edit files.
- Do not run sudo, apt, snap, package managers, package scripts, Zig build,
  submodule update, or Ghostty build.
- Do not suggest system-wide Zig install unless you are explicitly rejecting
  the local pinned Zig path and giving a stronger reason.
- Do not review unrelated Limux architecture.

## Required Lenses

Use the lens that best fits your role, but cover concrete file/line or command
evidence:

- Kazu: security/supply-chain and operator-policy lens.
- Zori: protocol architecture/convergence and hcom/workflow gate lens.
- Niru: platform semantics, shell safety, rollback/evidence lens.
- Tipi/Claude plugin: adversarial hidden-failure and tiebreaker lens.

## Questions To Answer

1. Is the reviewed artifact SHA exactly the expected value?
2. Are the mutation surfaces complete and bounded?
3. Is the Zig acquisition path sufficiently pinned and safer than `snap` or a
   system install for this local build lane?
4. Are Ghostty submodule checkout and Zig package fetches adequately bounded by
   pinned commit/hash checks?
5. Are stop conditions sufficient?
6. Are rollback/evidence steps sufficient and non-destructive?
7. Should the decision be `GO`, `WAIT`, `NO-GO`, or `DEFER`?

## Output Format

Respond in hcom with:

```text
VERDICT: GO | WAIT | NO-GO | DEFER
ROLE/LENS:
CRITICAL:
HIGH:
MEDIUM:
MUST-FIX BEFORE EXECUTE:
RESIDUAL RISKS:
RECOMMENDATION:
```

Only list findings that should affect execution of this specific gate.

## V2 Narrow Re-Review Request

The first consensus round converged on `WAIT` with no critical findings. The
review artifact was revised to address the reported blockers:

- Added execution-time Zig `index.json` URL/SHA/size cross-check.
- Added fresh per-run Zig extraction instead of trusting an existing extracted
  cache.
- Added archive prefix/absolute/parent-path containment validation before
  extraction.
- Dropped recursive Ghostty submodule initialization and added a stop condition
  for non-empty `ghostty/.gitmodules`.
- Added `.git/modules/ghostty`, local Git submodule config, Cargo cache reads,
  and `docs/evidence/` to mutation surfaces.
- Changed host test to `CARGO_NET_OFFLINE=true cargo test --locked ...`.
- Added durable `zig build`, `readelf`, `ldd`, cargo-test, and final git-status
  evidence logs.
- Recorded the `am-will/ghostty` fork trust anchor explicitly.

For v2, only re-check whether the above changes resolve your prior blockers
and whether any new blocker was introduced by the v2 command block.
