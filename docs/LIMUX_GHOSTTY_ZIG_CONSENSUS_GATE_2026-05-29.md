# Limux Ghostty/Zig Consensus Gate

Date: 2026-05-29
Status: CONSENSUS GO FOR OPERATOR APPROVAL; EXECUTION STILL WAIT
Scope: `/home/riche/MCPs/limux`

## Target Artifact

Reviewed artifact:

```text
docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md
```

Frozen v2 SHA256:

```text
dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5
```

The original v1 artifact SHA was:

```text
257a522e0690aa5367cf422efa467253ef6f3fc84308150922b3f2a66d820f6b
```

## Consensus Summary

Round 1 result: `WAIT`.

No reviewer found a critical issue, but the first artifact had unresolved
provenance and evidence gaps:

- Trust anchor for `am-will/ghostty` was not explicit.
- Nested submodule behavior was not bounded.
- Existing extracted Zig cache could be trusted by version string alone.
- Archive path containment was informational, not enforced.
- Cargo network/cache behavior was not bounded.
- Final git status and build-fetch evidence were missing.

Round 2 result: `GO for operator approval consideration`.

The v2 artifact addressed those blockers. It does not authorize execution by
itself. It means the exact v2 command block is now suitable for explicit
operator approval under the mutation-script gate.

## Reviewer Verdicts

| Reviewer | Lens | V1 | V2 |
|---|---|---:|---:|
| `niru` | Platform semantics, shell safety, rollback/evidence | WAIT | GO |
| `zori` | Protocol convergence, hcom/workflow gate | WAIT | GO |
| `kazu` | Security/supply-chain, operator policy | WAIT | GO, conditional on explicit operator approval |
| Claude plugin | Cross-family adversarial tiebreaker | WAIT | GO for freezing the document; execution still needs approval |

## V2 Fixes

The v2 artifact added:

- Execution-time Zig `index.json` URL/SHA/size cross-check.
- Fresh per-run Zig extraction directory instead of trusting existing extracted
  cache contents.
- Programmatic archive member-name containment checks before extraction.
- `tar --no-same-owner --no-same-permissions`.
- Non-recursive Ghostty submodule initialization.
- Stop condition for non-empty `ghostty/.gitmodules`.
- Explicit `am-will/ghostty` fork trust-anchor section.
- Mutation surfaces for `.git/modules/ghostty`, local submodule config, Cargo
  cache reads, and `docs/evidence/`.
- `CARGO_NET_OFFLINE=true cargo test --locked ...`.
- Durable evidence logs for Zig build, `readelf`, `ldd`, cargo test, and final
  git status.

## Residual Risks

These are accepted only if the operator explicitly approves the exact v2 block:

- The command builds and executes external native build logic from the pinned
  `am-will/ghostty` commit on the host.
- The fork is reproducibly pinned, but reproducibility is not proof of
  benignity.
- Zig package dependencies are hash-governed by `ghostty/build.zig.zon`, but
  the dependency set is trusted transitively from the pinned fork commit.
- The no-sudo Zig provenance gate relies on official Zig HTTPS metadata and a
  pinned SHA/size; minisign verification is not used because `minisign` is not
  installed and installing it would be a separate package-manager mutation.
- `CARGO_NET_OFFLINE=true` may fail if local Cargo caches are incomplete. That
  is an acceptable stop condition, not a security blocker.

## Decision

Decision: `GO FOR EXPLICIT OPERATOR APPROVAL; WAIT FOR EXECUTION`.

The command block must not run until the operator approves this exact v2
artifact SHA:

```text
dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5
```

Any semantic edit to the command block reopens the affected review lens and
requires a new SHA.

## Approval Text

Use this exact approval if proceeding:

```text
I approve executing the exact Ghostty/Zig v2 command block in docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md with SHA256 dddf26db51d3d4a3f16ce9414f33497597ab2014c14a142b83ca4a3a1e7837e5.

Approved scope:
- download official Zig index metadata and Zig 0.15.2 tarball to /home/riche/.cache/limux-tools
- verify official index URL/SHA/size and tarball SHA256 02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239
- extract Zig into a fresh per-run directory only after archive containment checks
- initialize the pinned am-will/ghostty submodule at 81ab8ffa90185221782baf785e85387321e16f8d
- accept am-will/ghostty as the canonical vendored Ghostty trust anchor for this Limux fork for this bounded local build lane
- refuse nested Ghostty submodules unless separately reviewed
- build ghostty/zig-out/lib/libghostty.so
- capture build/readelf/ldd/cargo-test/final-git-status evidence under docs/evidence/
- run readelf and ldd on libghostty.so
- run CARGO_NET_OFFLINE=true cargo test --locked -p limux-host-linux surface_send_text_response

Stop on the listed stop conditions.
Do not use sudo, snap, apt, system-wide Zig install, package.sh, recursive submodule init, submodule update beyond the pinned commit, full scripts/check.sh, Xvfb smoke, rollback cleanup, or system-wide Limux install in this step.
```

## Sources

- Local target artifact:
  `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md`
- Local consensus request:
  `docs/LIMUX_GHOSTTY_ZIG_CONSENSUS_REVIEW_REQUEST_2026-05-29.md`
- hcom thread: `limux-ghostty-zig-gate`
- Official Zig download page and minisign note:
  `https://ziglang.org/download/`
- Official Zig JSON metadata:
  `https://ziglang.org/download/index.json`
- Pinned Ghostty `build.zig.zon`:
  `https://raw.githubusercontent.com/am-will/ghostty/81ab8ffa90185221782baf785e85387321e16f8d/build.zig.zon`
