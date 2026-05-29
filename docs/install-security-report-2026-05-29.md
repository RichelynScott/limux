# Limux Upstream Sync And Install Security Report

Date: 2026-05-29

Scope: `/home/riche/MCPs/limux`

Assumption: the blank in "install ____ normally" refers to **Limux** in this
repo. If the intended target was Multica or another tool, rerun this report for
that repository/package because the dependency graph and threat model change.

## Executive Summary

Your fork does **not** need an upstream-main update right now.

After fetching all remotes:

| Item | Result |
|---|---|
| `origin` | `https://github.com/RichelynScott/limux.git` |
| `upstream` | `https://github.com/am-will/limux.git` |
| Local branch | `main` |
| Local `HEAD` | `0fe6984` |
| `upstream/main` | `9ffc934` |
| Ahead of upstream main | 2 commits |
| Behind upstream main | 0 commits |

The upstream branches that looked relevant are already ancestors of this fork,
except `upstream/fix/render-throttling`, which is an old divergent branch, not
a clean security update to merge into `main`.

## Upstream Branch Review

| Upstream ref | Status against this fork | Action |
|---|---|---|
| `upstream/main` | Fork is ahead 2, behind 0. | Nothing to merge. |
| `upstream/fix/issue-66-appimage-webkitgtk` | Already contained in this fork. | No action. |
| `upstream/docs/agent-hooks` | Already contained in this fork. | No action. |
| `upstream/save-sesh` | Already contained in this fork. | No action. |
| `upstream/fix/render-throttling` | Not merged; branch is divergent from an older base. | Do not merge without a separate code review. |

The AppImage/WebKit branch is the one that most closely matches a security or
runtime-packaging concern. It is already included in this fork's history.

## Can Limux Be Installed "Normally"?

Yes, but "normal" has different meanings:

| Install path | What happens | Security posture |
|---|---|---|
| `.deb` release | `sudo dpkg -i ./limux_0.1.19_amd64.deb`; package depends on host GTK/libadwaita/WebKitGTK packages. | Best normal path if the artifact is from a trusted release and checksums/signatures are verified. Root install and maintainer scripts still matter. |
| AppImage | `chmod +x` then run the AppImage. | Lower install friction, but larger bundled runtime surface and artifact trust still matter. |
| Tarball `install.sh` | Runs a shell installer with sudo and copies binaries/libraries/icons. | More manual and harder to audit than a distro package; inspect before running. |
| Source build | Builds Rust crates plus Ghostty with Zig and system dev packages. | Most reproducible for your fork, but largest build-time supply-chain surface. |

The reason not to blindly "just install normally" is not that normal install is
impossible. It is that a normal upstream release may not include your fork
customizations, and any root-level installer/package path expands the trust
surface.

## Runtime Dependencies For Normal `.deb` Install

The package metadata in `scripts/package.sh` declares:

```text
Depends: libgtk-4-1, libadwaita-1-0, libwebkitgtk-6.0-4
```

On this Ubuntu 24.04/Noble environment, `apt-cache policy` reports candidate
versions:

| Package | Candidate version | Notes |
|---|---:|---|
| `libgtk-4-1` | `4.14.5+ds-0ubuntu0.10` | From `noble-updates/main`. |
| `libadwaita-1-0` | `1.5.0-1ubuntu2` | From `noble/main`. |
| `libwebkitgtk-6.0-4` | `2.52.3-0ubuntu0.24.04.1` | From `noble-updates` and `noble-security`. |

Important transitive runtime packages include:

- graphics/windowing: `libwayland-client0`, `libwayland-egl1`, `libx11-6`,
  `libxrandr2`, `libxdamage1`, `libxext6`, `libxfixes3`, `libxi6`,
  `libxkbcommon0`, `libvulkan1`, `libgles2`
- media/browser: `libjavascriptcoregtk-6.0-1`, `gstreamer1.0-plugins-base`,
  `gstreamer1.0-plugins-good`, `libsoup-3.0-0`, `libsecret-1-0`,
  `libsqlite3-0`, `libxml2`, `libxslt1.1`
- WebKit sandboxing support: `bubblewrap`, `xdg-dbus-proxy`, `libseccomp2`
- image/text/rendering: `libcairo2`, `libpango-1.0-0`, `libharfbuzz0b`,
  `libgdk-pixbuf-2.0-0`, `libpng16-16t64`, `libjpeg8`, `libtiff6`,
  `libwebp7`

## Source Build Dependencies

The README source-build path requires:

- Rust stable toolchain
- Zig
- initialized Ghostty submodule
- `libgtk-4-dev`
- `libadwaita-1-dev`
- `libwebkitgtk-6.0-dev`
- `pkg-config`
- `build-essential`

The source build also runs:

```bash
git submodule update --init --recursive
(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)
cargo build --release
```

That means source builds trust three major supply-chain surfaces:

1. crates from `crates.io`
2. the Ghostty git submodule and its Zig build graph
3. OS package repositories for GTK/WebKit dev libraries

## Rust Dependency Scan

Static dependency inventory from `cargo metadata --locked`:

| Metric | Result |
|---|---:|
| crates.io packages in lock graph | 144 |
| local workspace crates | 6 |
| direct runtime Rust dependencies | `gtk4`, `gdk4-wayland`, `libadwaita`, `webkit6`, `tokio`, `serde`, `serde_json`, `clap`, `dirs`, `uuid`, `shell-quote`, `anyhow`, `thiserror`, `libc` |
| build dependency of note | `cc` for `limux-ghostty-sys` |

OSV query result for locked `crates.io` packages:

```text
0 vulnerabilities returned by https://api.osv.dev/v1/querybatch
```

Limitations:

- `cargo audit` and `cargo deny` are not installed in this environment.
- I did not install those tools because installing scanners is itself package
  execution and should be approved deliberately.
- OSV result is advisory-database coverage, not a proof that code is safe.

## Highest Security-Relevant Areas

| Area | Concern | Practical posture |
|---|---|---|
| WebKitGTK / JavaScriptCore | Browser engine CVEs are common and can be high impact if untrusted web content is loaded. | Prefer distro security packages; keep `libwebkitgtk-6.0-4` current. |
| AppImage runtime bundling | Bundled browser/runtime libraries can drift from distro security updates. | Prefer `.deb` for security-managed hosts unless AppImage is needed. |
| Tarball installer | `install.sh` runs with root privileges and copies binaries/libraries. | Inspect before running; prefer package manager when possible. |
| GitHub release artifacts | Unsigned or unchecked artifacts are a supply-chain risk. | Verify checksums/signatures when available; pin exact release. |
| Rust crates | 144 external crates, including GTK/WebKit bindings and proc macros. | Keep `Cargo.lock`; scan with OSV/RustSec before release. |
| Ghostty submodule | External native/FFI dependency built through Zig. | Pin submodule commit, review updates separately, avoid floating submodules. |
| GitHub Actions | Release workflows fetch tools and AppImageKit artifacts. | Pin actions and verify external binaries where feasible. |

## Why WebKitGTK Matters

Limux embeds a browser surface via WebKitGTK. That is useful, but it means the
runtime inherits the browser engine's security profile. Ubuntu's security
notices show WebKitGTK updates regularly fix memory corruption, arbitrary code
execution, cross-origin, and related browser-class issues. The current Noble
candidate seen locally is `2.52.3-0ubuntu0.24.04.1`, which comes from the
security/update pocket.

## Recommended Install Policy

For daily use on this machine:

1. Use the `.deb` path for release builds when possible.
2. Pin the exact release artifact and verify checksum/signature if upstream
   publishes one.
3. Let Ubuntu manage GTK/libadwaita/WebKitGTK security updates.
4. Avoid `curl | bash` install flows.
5. Avoid unsigned desktop/AppImage artifacts for sensitive work unless there is
   a concrete reason to prefer them.
6. For this fork's unreleased features, build locally from the pinned repo and
   locked dependencies instead of installing upstream release binaries.

For release/build work:

1. Run `cargo metadata --locked` and an advisory scan before packaging.
2. Install `cargo-audit` or `cargo-deny` only after approving the tool install
   path.
3. Build Ghostty from the pinned submodule commit.
4. Run `./scripts/check.sh` after `ghostty/zig-out/lib/libghostty.so` exists.
5. Treat AppImage packaging as a separate security review because it bundles or
   copies browser/runtime components.

## Commands Run

```bash
git fetch --all --prune
git rev-list --count HEAD ^upstream/main
git rev-list --count upstream/main ^HEAD
git branch -r --no-merged HEAD
cargo metadata --locked --format-version 1
cargo tree --locked --all-features --prefix none
apt-cache policy libgtk-4-1 libadwaita-1-0 libwebkitgtk-6.0-4
apt-cache depends libgtk-4-1 libadwaita-1-0 libwebkitgtk-6.0-4
curl -sS -X POST https://api.osv.dev/v1/querybatch
```

## Sources

- Local repo: `README.md`, `scripts/package.sh`, `Cargo.toml`,
  `rust/limux-host-linux/Cargo.toml`, `Cargo.lock`
- Upstream remote: `https://github.com/am-will/limux.git`
- OSV API: <https://google.github.io/osv.dev/api/>
- WebKitGTK security page: <https://webkitgtk.org/security.html>
- Ubuntu Security Notices: <https://ubuntu.com/security/notices>
