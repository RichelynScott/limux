# Limux Ghostty/Zig Mutation Review

Date: 2026-05-29
Status: DRAFT ONLY - NOT APPROVED FOR EXECUTION
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
  `https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz`.
- Network fetches performed by `zig build` for Ghostty dependencies declared in
  `ghostty/build.zig.zon`, including `https://deps.files.ghostty.org/...`
  sources with Zig package hashes.
- Writes under `/home/riche/.cache/limux-tools/`.
- Writes under `/home/riche/MCPs/limux/ghostty/`,
  `/home/riche/MCPs/limux/target/`, and Zig build cache/output paths.

## Recommended Path

Use a project-scoped, non-sudo Zig binary pinned to the exact version and SHA
required by the pinned Ghostty commit. Avoid `snap install zig`, distro
packages with unknown version drift, and any `curl | bash` installer.

Official Zig metadata for `0.15.2` `x86_64-linux`:

| Field | Value |
|---|---|
| Tarball | `https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz` |
| SHA256 | `02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239` |
| Size | `53733924` |

## Exact Draft Command Block

This block is not approved for execution yet.

```bash
set -euo pipefail

cd /home/riche/MCPs/limux

ROOT="$PWD"
ZIG_VERSION="0.15.2"
ZIG_URL="https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz"
ZIG_SHA256="02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239"
ZIG_CACHE_ROOT="$HOME/.cache/limux-tools"
ZIG_ARCHIVE="$ZIG_CACHE_ROOT/zig-x86_64-linux-$ZIG_VERSION.tar.xz"
ZIG_DIR="$ZIG_CACHE_ROOT/zig-x86_64-linux-$ZIG_VERSION"
ZIG_GLOBAL_CACHE="$ZIG_CACHE_ROOT/zig-global-cache-$ZIG_VERSION"
GHOSTTY_COMMIT="81ab8ffa90185221782baf785e85387321e16f8d"

date -Is
git status --short --branch
git rev-parse HEAD
git submodule status --recursive
command -v cargo
command -v rustc
command -v curl
command -v tar
command -v sha256sum
command -v pkg-config
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1
pkg-config --modversion webkitgtk-6.0
command -v zig || true

mkdir -p "$ZIG_CACHE_ROOT" "$ZIG_GLOBAL_CACHE"

if [ ! -f "$ZIG_ARCHIVE" ]; then
  curl --fail --location --show-error "$ZIG_URL" --output "$ZIG_ARCHIVE"
fi

echo "$ZIG_SHA256  $ZIG_ARCHIVE" | sha256sum --check -
tar -tf "$ZIG_ARCHIVE" | sed -n '1,20p'

if [ ! -x "$ZIG_DIR/zig" ]; then
  tar -xJf "$ZIG_ARCHIVE" -C "$ZIG_CACHE_ROOT"
fi

test "$("$ZIG_DIR/zig" version)" = "$ZIG_VERSION"

git submodule update --init --recursive ghostty
test "$(git -C ghostty rev-parse HEAD)" = "$GHOSTTY_COMMIT"
test -f ghostty/build.zig
grep -F 'minimum_zig_version = "0.15.2"' ghostty/build.zig.zon

(
  cd ghostty
  "$ZIG_DIR/zig" build \
    --global-cache-dir "$ZIG_GLOBAL_CACHE" \
    -Dapp-runtime=none \
    -Doptimize=ReleaseFast
)

test -f ghostty/zig-out/lib/libghostty.so
ldd ghostty/zig-out/lib/libghostty.so

LD_LIBRARY_PATH="$ROOT/ghostty/zig-out/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
  cargo test -p limux-host-linux surface_send_text_response
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

1. The official Zig tarball checksum matches the pinned SHA256.
2. The local Zig binary reports version `0.15.2`.
3. The Ghostty submodule initializes at exactly
   `81ab8ffa90185221782baf785e85387321e16f8d`.
4. `ghostty/build.zig.zon` declares `minimum_zig_version = "0.15.2"`.
5. `ghostty/zig-out/lib/libghostty.so` is produced.
6. `cargo test -p limux-host-linux surface_send_text_response` runs with
   `LD_LIBRARY_PATH` pointing at `ghostty/zig-out/lib`.

## Stop Conditions

Stop before executing the build if:

- The Zig SHA256 check fails.
- The tarball listing does not start with the expected
  `zig-x86_64-linux-0.15.2/` directory.
- The extracted Zig binary does not report version `0.15.2`.
- `git submodule update` checks out any Ghostty commit other than
  `81ab8ffa90185221782baf785e85387321e16f8d`.
- `ghostty/build.zig.zon` does not declare the expected minimum Zig version.
- Any command asks for sudo, credentials, or interactive confirmation.
- Network fetches fail with TLS, certificate, auth, redirect, or trust errors.
- The command block attempts writes outside the stated mutation surfaces.

Stop after the build if:

- `libghostty.so` is still missing.
- `ldd ghostty/zig-out/lib/libghostty.so` reports missing shared libraries.
- The host test fails for a new prerequisite not listed here.
- `git status --short --branch` shows unexpected tracked-file changes outside
  normal submodule initialization state.

## Rollback Plan

Do not run rollback automatically. Review state first.

Possible cleanup actions after review:

- Move `/home/riche/.cache/limux-tools/zig-x86_64-linux-0.15.2*` to an
  archive directory if the project-scoped Zig cache should be retired.
- Move `/home/riche/.cache/limux-tools/zig-global-cache-0.15.2` to an archive
  directory if the Zig package cache should be retired.
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

- Record Zig download URL and SHA check result.
- Record extracted Zig version.
- Record Ghostty submodule commit.
- Record `ghostty/build.zig.zon` minimum Zig version.

After mutation:

- Record `test -f ghostty/zig-out/lib/libghostty.so`.
- Record `ldd ghostty/zig-out/lib/libghostty.so`.
- Run `cargo test -p limux-host-linux surface_send_text_response`.
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
SHA256. The Ghostty submodule is pinned by commit. Ghostty package downloads
are governed by hashes in `build.zig.zon`, but still execute native build
logic from an external source tree.

Finding: WAIT for explicit approval of this external-code build lane.

### 3. Platform Semantics

The command block avoids sudo and system package managers. Zig is staged under
`$HOME/.cache/limux-tools`, and the build writes into the repo submodule and
normal Cargo/Zig build outputs. `LD_LIBRARY_PATH` is set only for the host test.

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
- No CRITICAL or HIGH blocker was found in the proposed scope.
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
