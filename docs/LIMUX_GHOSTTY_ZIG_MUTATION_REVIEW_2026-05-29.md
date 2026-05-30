# Limux Ghostty/Zig Mutation Review

Date: 2026-05-29
Status: DRAFT V2 ONLY - NOT APPROVED FOR EXECUTION
Scope: `/home/riche/MCPs/limux`
Target environment: Ubuntu 24.04.1 LTS Noble

## Decision Context

The bounded apt prerequisite lane is complete. The host test now reaches the
next expected gate:

```text
limux-ghostty-sys: libghostty not found
```

Current local state:

- `pkg-config --modversion gtk4 libadwaita-1 webkitgtk-6.0` succeeds.
- `ghostty/` is present but empty.
- `git submodule status --recursive` reports the pinned Ghostty commit as
  uninitialized: `-81ab8ffa90185221782baf785e85387321e16f8d ghostty`.
- `ghostty/zig-out/lib/libghostty.so` is missing.
- `zig` is not on `PATH`.

The repo README source-build path requires:

```bash
git submodule update --init --recursive
(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)
```

The pinned Ghostty commit's `build.zig.zon` declares
`minimum_zig_version = "0.15.2"`.

## Classification

Label: external source checkout + external tool download + native build.

Mutation surfaces:

- Network fetch from `https://github.com/am-will/ghostty.git` for the pinned
  `ghostty` submodule.
- Network fetch from official Zig download infrastructure:
  `https://ziglang.org/download/index.json` and
  `https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz`.
- Network fetches performed by `zig build` for Ghostty dependencies declared in
  `ghostty/build.zig.zon`, including `https://deps.files.ghostty.org/...`
  sources with Zig package hashes.
- Writes under `/home/riche/.cache/limux-tools/`.
- Writes under `/home/riche/MCPs/limux/ghostty/`,
  `/home/riche/MCPs/limux/.git/modules/ghostty`, local Git submodule config,
  `/home/riche/MCPs/limux/target/`, and Zig build cache/output paths.
- Writes under `/home/riche/MCPs/limux/docs/evidence/` for build/test logs.
- Cargo reads the locked dependency graph and may read local Cargo caches under
  `$CARGO_HOME`/`$HOME/.cargo`; the command uses `--locked` and
  `CARGO_NET_OFFLINE=true` to avoid Cargo network access in this lane.

## Trust Anchor

This repo pins `ghostty` to `https://github.com/am-will/ghostty.git`, not to
upstream `ghostty-org/ghostty`. That fork is the canonical vendored Ghostty
source for this Limux fork at the recorded gitlink. Approving this lane means
accepting host-native build execution from `am-will/ghostty` commit
`81ab8ffa90185221782baf785e85387321e16f8d` as the same trust root as this
Limux source tree. This is reproducible, but reproducibility is not a benignity
proof.

## Recommended Path

Use a project-scoped, non-sudo Zig binary pinned to the exact version and SHA
required by the pinned Ghostty commit. Avoid `snap install zig`, distro
packages with unknown version drift, and any `curl | bash` installer.

Official Zig metadata for `0.15.2` `x86_64-linux`:

| Field | Value |
|---|---|
| Index | `https://ziglang.org/download/index.json` |
| Tarball | `https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz` |
| SHA256 | `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239` |
| Size | `53733924` |

The command block cross-checks URL, SHA256, and size against the official
`index.json` at execution time before extraction. Zig also publishes minisign
signatures; this environment does not currently have `minisign` installed, and
installing it would be a separate package-manager mutation. The execution-time
metadata cross-check is the selected no-sudo provenance gate.

## Exact Draft Command Block

This block is not approved for execution yet.

```bash
set -euo pipefail

cd /home/riche/MCPs/limux

ROOT="$PWD"
ZIG_VERSION="0.15.2"
ZIG_INDEX_URL="https://ziglang.org/download/index.json"
ZIG_URL="https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz"
ZIG_SHA256="02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239"
ZIG_SIZE="53733924"
ZIG_CACHE_ROOT="$HOME/.cache/limux-tools"
ZIG_RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-$$"
ZIG_RUN_ROOT="$ZIG_CACHE_ROOT/runs"
ZIG_RUN_DIR="$ZIG_RUN_ROOT/$ZIG_RUN_ID"
ZIG_ARCHIVE="$ZIG_CACHE_ROOT/zig-x86_64-linux-$ZIG_VERSION.tar.xz"
ZIG_INDEX="$ZIG_CACHE_ROOT/zig-index-$ZIG_VERSION.json"
ZIG_MEMBERS="$ZIG_RUN_DIR/zig-archive-members.txt"
ZIG_DIR="$ZIG_RUN_DIR/zig-x86_64-linux-$ZIG_VERSION"
ZIG_GLOBAL_CACHE="$ZIG_CACHE_ROOT/zig-global-cache-$ZIG_VERSION"
EVIDENCE_DIR="$ROOT/docs/evidence/limux-ghostty-zig-$ZIG_RUN_ID"
GHOSTTY_COMMIT="81ab8ffa90185221782baf785e85387321e16f8d"

date -Is
git status --short --branch
git rev-parse HEAD
git submodule status --recursive
command -v cargo
command -v rustc
command -v curl
command -v jq
command -v tar
command -v awk
command -v tee
command -v readelf
command -v sha256sum
command -v pkg-config
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1
pkg-config --modversion webkitgtk-6.0
command -v zig || true

mkdir -p "$ZIG_CACHE_ROOT" "$ZIG_GLOBAL_CACHE" "$ZIG_RUN_ROOT" "$EVIDENCE_DIR"

if [ -e "$ZIG_RUN_DIR" ]; then
  echo "refusing to reuse existing Zig run directory: $ZIG_RUN_DIR" >&2
  exit 1
fi
mkdir -p "$ZIG_RUN_DIR"

curl --fail --proto '=https' --tlsv1.2 --location --max-redirs 3 --show-error \
  "$ZIG_INDEX_URL" --output "$ZIG_INDEX"

test "$(jq -r --arg v "$ZIG_VERSION" '.[$v]["x86_64-linux"].tarball' "$ZIG_INDEX")" = "$ZIG_URL"
test "$(jq -r --arg v "$ZIG_VERSION" '.[$v]["x86_64-linux"].shasum' "$ZIG_INDEX")" = "$ZIG_SHA256"
test "$(jq -r --arg v "$ZIG_VERSION" '.[$v]["x86_64-linux"].size' "$ZIG_INDEX")" = "$ZIG_SIZE"

if [ ! -f "$ZIG_ARCHIVE" ]; then
  curl --fail --proto '=https' --tlsv1.2 --location --max-redirs 3 --show-error \
    "$ZIG_URL" --output "$ZIG_ARCHIVE"
fi

echo "$ZIG_SHA256  $ZIG_ARCHIVE" | sha256sum --check -
test "$(wc -c < "$ZIG_ARCHIVE")" = "$ZIG_SIZE"

tar -tf "$ZIG_ARCHIVE" > "$ZIG_MEMBERS"
awk -v prefix="zig-x86_64-linux-$ZIG_VERSION/" '
  index($0, prefix) != 1 {
    print "unexpected archive member: " $0 > "/dev/stderr";
    bad = 1
  }
  $0 ~ /^\// || $0 ~ /(^|\/)\.\.(\/|$)/ {
    print "unsafe archive member: " $0 > "/dev/stderr";
    bad = 1
  }
  END { exit bad }
' "$ZIG_MEMBERS"
sed -n '1,20p' "$ZIG_MEMBERS"

tar --no-same-owner --no-same-permissions -xJf "$ZIG_ARCHIVE" -C "$ZIG_RUN_DIR"

test "$("$ZIG_DIR/zig" version)" = "$ZIG_VERSION"

git submodule update --init ghostty
test "$(git -C ghostty rev-parse HEAD)" = "$GHOSTTY_COMMIT"
if [ -s ghostty/.gitmodules ]; then
  echo "unexpected nested Ghostty submodules:" >&2
  cat ghostty/.gitmodules >&2
  exit 1
fi
git -C ghostty submodule status --recursive
test -f ghostty/build.zig
grep -F 'minimum_zig_version = "0.15.2"' ghostty/build.zig.zon

(
  cd ghostty
  "$ZIG_DIR/zig" build \
    --global-cache-dir "$ZIG_GLOBAL_CACHE" \
    -Dapp-runtime=none \
    -Doptimize=ReleaseFast 2>&1 | tee "$EVIDENCE_DIR/zig-build.log"
)

test -f ghostty/zig-out/lib/libghostty.so
readelf -d ghostty/zig-out/lib/libghostty.so | tee "$EVIDENCE_DIR/libghostty-readelf-dynamic.log"
ldd ghostty/zig-out/lib/libghostty.so | tee "$EVIDENCE_DIR/libghostty-ldd.log"

CARGO_NET_OFFLINE=true \
  LD_LIBRARY_PATH="$ROOT/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
  cargo test --locked -p limux-host-linux surface_send_text_response 2>&1 | tee "$EVIDENCE_DIR/cargo-test-host-send-text.log"

git status --short --branch | tee "$EVIDENCE_DIR/final-git-status.txt"
```

## Explicit Non-Goals

- Do not install Zig system-wide.
- Do not use `snap install zig`.
- Do not use `curl | bash`.
- Do not run `sudo`.
- Do not install Limux system-wide.
- Do not run `scripts/package.sh`.
- Do not run `apt autoremove`.
- Do not update the Ghostty submodule to a newer commit.
- Do not modify `/home/riche/.claude`.

## Success Criteria

The command block succeeds only if:

1. Official Zig `index.json` matches the pinned URL, SHA256, and size.
2. The official Zig tarball checksum and byte size match the pinned values.
3. The archive member list is contained under
   `zig-x86_64-linux-0.15.2/` and contains no absolute or parent-directory
   paths.
4. A freshly extracted local Zig binary reports version `0.15.2`.
5. The Ghostty submodule initializes at exactly
   `81ab8ffa90185221782baf785e85387321e16f8d`.
6. Ghostty has no nested git submodules, or the command stops.
7. `ghostty/build.zig.zon` declares `minimum_zig_version = "0.15.2"`.
8. `ghostty/zig-out/lib/libghostty.so` is produced.
9. `cargo test --locked -p limux-host-linux surface_send_text_response` runs
   offline with `LD_LIBRARY_PATH` pointing at `ghostty/zig-out/lib`.

## Stop Conditions

Stop before executing the build if:

- Official Zig `index.json` does not match the pinned tarball URL, SHA256, and
  size.
- The Zig SHA256 or byte-size check fails.
- Any archive member is outside the expected `zig-x86_64-linux-0.15.2/`
  directory, absolute, or parent-directory relative.
- The extracted Zig binary does not report version `0.15.2`.
- `git submodule update` checks out any Ghostty commit other than
  `81ab8ffa90185221782baf785e85387321e16f8d`.
- `ghostty/.gitmodules` is non-empty, indicating nested submodules that need a
  separate pin review.
- `ghostty/build.zig.zon` does not declare the expected minimum Zig version.
- Any command asks for sudo, credentials, or interactive confirmation.
- Network fetches fail with TLS, certificate, auth, redirect, or trust errors.
- The command block attempts writes outside the stated mutation surfaces.

Stop after the build if:

- `libghostty.so` is still missing.
- `ldd ghostty/zig-out/lib/libghostty.so` reports missing shared libraries.
- The host test fails for a new prerequisite not listed here.
- `CARGO_NET_OFFLINE=true cargo test --locked ...` attempts network access or
  fails due missing Cargo cache entries.
- `git status --short --branch` shows unexpected tracked-file changes outside
  normal submodule initialization state, or unexpected untracked files outside
  `docs/evidence/` and normal build/cache outputs.

## Rollback Plan

Do not run rollback automatically. Review state first.

Possible cleanup actions after review:

- Move `/home/riche/.cache/limux-tools/zig-x86_64-linux-0.15.2*` to an
  archive directory if the project-scoped Zig cache should be retired.
- Move `/home/riche/.cache/limux-tools/zig-global-cache-0.15.2` to an archive
  directory if the Zig package cache should be retired.
- Move `/home/riche/.cache/limux-tools/runs/*` to an archive directory if
  per-run extracted Zig directories should be retired.
- Leave `ghostty/` initialized if future Limux builds are expected.

Avoid destructive cleanup commands until the next verification state is known.

## Evidence Plan

Before mutation:

- Record `git status --short --branch`.
- Record `git rev-parse HEAD`.
- Record `git submodule status --recursive`.
- Record Zig absence/presence.
- Record GTK/WebKit `pkg-config` versions.

During mutation:

- Record Zig `index.json` URL/SHA/size cross-check result.
- Record Zig download URL, byte-size, and SHA check result.
- Record archive containment validation.
- Record extracted Zig version.
- Record Ghostty submodule commit.
- Record absence of nested Ghostty submodules.
- Record `ghostty/build.zig.zon` minimum Zig version.
- Capture full `zig build` output, including fetch URLs/hashes, under
  `docs/evidence/`.

After mutation:

- Record `test -f ghostty/zig-out/lib/libghostty.so`.
- Record `readelf -d ghostty/zig-out/lib/libghostty.so`.
- Record `ldd ghostty/zig-out/lib/libghostty.so`.
- Run `CARGO_NET_OFFLINE=true cargo test --locked -p limux-host-linux
  surface_send_text_response`.
- Record final `git status --short --branch`.
- If host test succeeds, next gated verification is `./scripts/check.sh`,
  followed by `LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh`.

Durable repo notes:

- Update `FYI.md` and `HANDOFF.md` with actual commands run, pass/fail
  results, and any new blocker.

## Review Lenses

### 1. Protocol Architecture / Convergence

The command block is scoped to the next known blocker only: produce
`libghostty.so` for local testing. It does not install Limux, run packaging,
or update Ghostty beyond the pinned submodule commit.

Finding: GO for shape, pending explicit approval.

### 2. Security / Hostile Input

The highest-risk inputs are the Zig binary tarball, GitHub submodule checkout,
and Ghostty's Zig package downloads. The Zig tarball is pinned by official
SHA256 and cross-checked against official Zig metadata at execution time. The
Ghostty submodule is pinned by commit, and nested submodules are refused unless
separately reviewed. Ghostty package downloads are governed by hashes in
`build.zig.zon`, but still execute native build logic from an external source
tree.

Finding: WAIT for explicit approval of this external-code build lane.

### 3. Platform Semantics

The command block avoids sudo and system package managers. Zig is staged under
`$HOME/.cache/limux-tools` in a fresh per-run extraction directory, and the
build writes into the repo submodule and normal Cargo/Zig build outputs.
`LD_LIBRARY_PATH` is set only for the offline locked host test.

Finding: GO if the operator accepts local cache writes and submodule checkout.

### 4. Operations / Rollback / Evidence

Rollback should be deliberate and archival, not destructive. The resulting
`ghostty/` checkout and `libghostty.so` are useful for ongoing Limux work.

Finding: GO with manual cleanup review if the build fails or the cache should
be removed later.

### 5. Domain-Specific: Limux Host Verification

This lane should clear the current `limux-ghostty-sys` blocker. It does not
prove the full GUI smoke path is healthy; that remains the next verification
stage after the host test.

Finding: WAIT for approval, then run host test and proceed to full gate only
if the Ghostty build succeeds.

## Mutation Wave Decision

Decision: WAIT

Reason:

- The lane is bounded and reviewable.
- Consensus review found no CRITICAL blockers, but the revised command block
  needs explicit approval as a new frozen artifact SHA before execution.
- Execution still requires explicit human approval of this exact draft command
  block because it downloads and builds external native code.

This review is evidence, not final mutation approval.

## Sources

- Repo README source-build instructions: `README.md`
- Repo packaging preflight/build logic: `scripts/package.sh`
- Pinned submodule metadata: `.gitmodules` and `git submodule status`
- Pinned Ghostty `build.zig.zon`:
  `https://raw.githubusercontent.com/am-will/ghostty/81ab8ffa90185221782baf785e85387321e16f8d/build.zig.zon`
- Official Zig release metadata:
  `https://ziglang.org/download/index.json`
