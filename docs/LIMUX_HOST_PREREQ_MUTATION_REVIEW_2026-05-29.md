# Limux Host Prerequisite Mutation Review

Date: 2026-05-29
Status: DRAFT ONLY - NOT APPROVED FOR EXECUTION
Scope: `/home/riche/MCPs/limux`
Target environment: Ubuntu 24.04.1 LTS Noble

## Decision Context

The selected path is a bounded host prerequisite install/build lane. This is
not a full Limux system install and not an upstream release install.

The immediate goal is to make host-side verification possible for this fork:

- `cargo test -p limux-host-linux surface_send_text_response`
- `./scripts/check.sh`
- `LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh`

## Current Read-Only Recon

Observed with read-only commands:

- `cargo` and `rustc` are installed.
- `build-essential` is installed.
- `xvfb` and `xvfb-run` are installed.
- `pkg-config` and `pkgconf` are not installed.
- `zig` is not on `PATH`.
- `ghostty/zig-out/lib/libghostty.so` is missing.
- The Ghostty submodule is not initialized in the current checkout:
  `-81ab8ffa90185221782baf785e85387321e16f8d ghostty`.

Current apt metadata reports candidates:

| Package | Candidate |
|---|---|
| `pkg-config` | `1.8.1-2build1` |
| `libgtk-4-dev` | `4.14.5+ds-0ubuntu0.10` |
| `libadwaita-1-dev` | `1.5.0-1ubuntu2` |
| `libwebkitgtk-6.0-dev` | `2.52.3-0ubuntu0.24.04.1` |

The apt simulation for the proposed install reported:

- 160 newly installed packages.
- 2 upgraded packages: `libpcre2-8-0`, `libselinux1`.
- 94 packages still not upgraded.

## Classification

Label: OS package install / build prerequisite mutation.

Mutation surfaces:

- APT package index state if `apt-get update` is executed.
- System package database under `/var/lib/dpkg`.
- System libraries and headers for GTK, libadwaita, WebKitGTK, GStreamer,
  graphics, and related transitive dependencies.
- Potential package upgrades for already-installed dependencies.

## Exact Draft Command Block

This block is not approved for execution yet.

```bash
set -euo pipefail

cd /home/riche/MCPs/limux

date -Is
sed -n '1,12p' /etc/os-release
git status --short --branch
git rev-parse HEAD

command -v cargo
command -v rustc
command -v xvfb-run
command -v pkg-config || true
command -v zig || true
test -f ghostty/zig-out/lib/libghostty.so && echo "libghostty present" || echo "libghostty missing"

dpkg-query -W -f='${binary:Package}\t${Status}\t${Version}\n' \
  pkg-config pkgconf build-essential xvfb \
  libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev \
  2>/dev/null || true

apt-cache policy \
  pkg-config pkgconf build-essential xvfb \
  libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev

apt-get -s install --no-install-recommends \
  pkg-config libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev

sudo apt-get update

sudo apt-get install --no-install-recommends \
  pkg-config libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev

command -v pkg-config
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1
pkg-config --modversion webkitgtk-6.0

cargo test -p limux-host-linux surface_send_text_response
```

## Explicit Non-Goals

- Do not install Limux system-wide.
- Do not install an upstream `.deb`, AppImage, AUR package, or tarball.
- Do not run `sudo ./install.sh`.
- Do not download Zig from the internet in this command block.
- Do not initialize or update git submodules in this command block.
- Do not build Ghostty in this command block.
- Do not run `apt autoremove` automatically.
- Do not modify `/home/riche/.claude`.

## Success Criteria

The command block succeeds only if:

1. APT installs the four requested top-level packages from Ubuntu repositories:
   `pkg-config`, `libgtk-4-dev`, `libadwaita-1-dev`, `libwebkitgtk-6.0-dev`.
2. `pkg-config --modversion gtk4`, `libadwaita-1`, and `webkitgtk-6.0`
   succeed.
3. The previously blocked host unit test compiles and runs, or fails past the
   prior `pkg-config` blocker with a new, clearly captured blocker.

## Stop Conditions

Stop before installing if:

- APT proposes removing packages.
- APT proposes upgrading security-sensitive packages beyond the two observed in
  simulation without a fresh review.
- APT cannot authenticate repositories or package signatures.
- APT proposes packages from an unexpected repository outside configured Ubuntu
  Noble sources.
- `sudo apt-get update` reports repository signature, TLS, or mirror errors.
- The install prompt materially differs from the simulated transaction.

Stop after installing if:

- `pkg-config` still cannot resolve `gtk4`, `libadwaita-1`, or `webkitgtk-6.0`.
- Host tests fail due to a new build prerequisite not listed here.
- Any command emits credential, auth, or repository trust errors.

## Rollback Plan

Do not run rollback automatically. Review the proposed removal list first.

Draft rollback commands:

```bash
sudo apt-get remove --purge \
  pkg-config libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev

sudo apt-get autoremove --purge
```

Rollback stop condition:

- Do not accept `autoremove` if it proposes removing unrelated packages that
  were already installed before this operation or are needed by other projects.

## Evidence Plan

Before mutation:

- Record `git status --short --branch`.
- Record `git rev-parse HEAD`.
- Record `/etc/os-release`.
- Record installed package state for top-level packages.
- Record `apt-cache policy` for top-level packages.
- Record `apt-get -s install` summary.

After mutation:

- Record installed versions for top-level packages.
- Record `pkg-config --modversion gtk4 libadwaita-1 webkitgtk-6.0`.
- Run `cargo test -p limux-host-linux surface_send_text_response`.
- If host test succeeds, proceed to the next gated verification step:
  `./scripts/check.sh` only after `ghostty/zig-out/lib/libghostty.so` exists.

Durable repo notes:

- Update `FYI.md` and `HANDOFF.md` with actual commands run, pass/fail
  results, and any new blocker.

## Review Lenses

### 1. Protocol Architecture / Convergence

The command block matches the selected lane: bounded host prerequisites only.
It does not install Limux, package this fork, build Ghostty, or start automatic
bootstrap. It should not be widened without reopening review.

Finding: GO for apt prerequisite lane if explicit human approval is given.

### 2. Security / Hostile Input

Inputs are configured Ubuntu apt repositories and local repo metadata. No
third-party binary download is included. WebKitGTK and JavaScriptCore are large
browser-engine surfaces, but this lane uses Ubuntu packaged versions from
configured Noble update/security repositories.

Finding: WAIT for final execution approval. Zig remains unresolved and should
not be acquired by ad hoc download in this lane.

### 3. Platform Semantics

The command block uses apt package names from README and read-only local apt
metadata. `--no-install-recommends` limits optional extras, but the transaction
still pulls a large GTK/WebKit development graph. `sudo apt-get update` mutates
apt lists and may change the transaction compared with the simulation.

Finding: GO if the operator accepts that the install may differ after metadata
refresh and agrees to stop if apt proposes removals or a materially different
transaction.

### 4. Operations / Rollback / Evidence

Rollback is possible through `apt-get remove --purge` of top-level packages,
but transitive dependencies require care because the system already reports
some auto-removable packages unrelated to this lane. `apt autoremove` must not
be accepted blindly.

Finding: GO with manual review of rollback/autoremove output.

### 5. Domain-Specific: Limux Build/Test

This lane should clear the current `pkg-config`/GTK sys-crate build blocker.
It will not clear the missing Ghostty library or missing Zig. A separate
review is needed for Zig installation/acquisition and Ghostty submodule/build.

Finding: WAIT for a second gate after apt prerequisites if Ghostty/Zig remain
blocked.

## Mutation Wave Decision

Decision: WAIT

Reason:

- The apt prerequisite lane is reviewable and bounded.
- No CRITICAL or HIGH blocker was found for the apt package lane itself.
- Execution still requires explicit human approval of this exact draft command
  block.
- Zig acquisition and Ghostty build are intentionally out of scope and remain a
  separate gate.

This review is evidence, not final mutation approval.
