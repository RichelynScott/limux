# Limux Security Dependency And Vulnerability Review - 2026-06-17

Date: 2026-06-17 19:40 EDT / 2026-06-17 23:40 UTC
Repo: `/home/riche/MCPs/limux`
Branch: `main`
Remote: `origin=https://github.com/RichelynScott/limux.git`, `upstream=https://github.com/am-will/limux.git`
Authoring session: `worker-limux-halo` / Codex
Status: review report only; no install, package build, package refresh, or privileged mutation performed

Post-review update: after this review, the operator approved the no-sudo
terminal-first user-local lane. `scripts/user-local-install/install-user-local.sh
--apply --desktop-entry` installed the existing release artifacts under
`/home/riche/.local/limux-reviewed/7c8eac72965f`, archived the previous
`~/.local/bin/limux` and `~/.local/bin/limux-cli` symlinks under
`/home/riche/.local/limux-reviewed/archive/20260617T235653Z`, and flipped both
PATH entries to reviewed wrappers. No package manager, build step, sudo,
`/etc` write, or root/global installer was used. Verification after the install:
`sha256sum -c SHA256SUMS`, `limux --help`, `limux-cli --help`, and
`limux --json identify` all exited 0. Browser/WebKit and root/global install
remain gated. Ghostty resources and terminfo were not present in the current
artifact tree, so this install improves layout/rollback but does not yet add a
full packaged resource build.

## Executive Verdict

| Surface | Verdict | Reason |
|---|---:|---|
| Current PATH launcher (`~/.local/bin/limux` -> `scripts/limux-dev`) | GO | It is a repo-local exec wrapper around already-built artifacts. It does not run Cargo, installers, package managers, or root writes. `limux --json identify` works against the live app. |
| Current terminal workspace / panes / agent orchestration | GO with caveats | The terminal/control path has real usable features: GTK app, Ghostty terminals, control socket, `agent-team`, hcom launch mode, read/send/notify. Treat it as a repo-local product build, not a full packaged install. |
| Browser/WebKit tabs and browser automation | WAIT | The host is built with the default `webkit` feature and has browser-tab UI code, but the local Ubuntu WebKitGTK runtime is `2.52.3-0ubuntu0.24.04.1`. Upstream WebKitGTK fixed June 2026 CVEs in `2.52.4`; the local Ubuntu changelog does not show those June 2 fixes yet. |
| Full/global install from `scripts/package.sh` tarball installer on trusted host | NO-GO today | The generated installer re-execs with `sudo`, writes `/usr/local`/`/etc/ld.so.conf.d`, runs cache/loader updates, and has cleanup/uninstall `rm`/`remove_tree` logic. That is not the same risk as the current repo-local launcher. |
| Near-full user-local install | WAIT, recommended next lane | This is the pragmatic next step if you want a "real installed" feel fast: no `sudo`, no `/etc`, no root cleanup, explicit manifest, artifact hashes, and rollback. It should not be treated as the same as system install. |
| `.deb` / package-manager system install | WAIT | Better than a raw tarball installer once built and reviewed, but it still mutates system package/runtime locations and should be tested first in the Project Isolation full disposable VM lane. |

Bottom line: do not impulse-run a full global install from this checkout on the trusted WSL host. The current Limux launcher is usable for terminal/agent work. The fastest safer upgrade path is a reviewed user-local install wrapper using the already-built artifacts, then a disposable-VM `.deb` trial for true system install.

## What Is Actually Installed Right Now

The current `limux` on PATH is not a system/global install:

- `/home/riche/.local/bin/limux -> /home/riche/MCPs/limux/scripts/limux-dev`
- `/home/riche/.local/bin/limux-cli -> /home/riche/MCPs/limux/scripts/limux-dev`
- No `/home/riche/.local/libexec/limux/limux-host` found.
- No `/home/riche/.local/lib/limux/libghostty.so` found.

`scripts/limux-dev` resolves the repo root, selects `target/release` by default, checks for existing `limux-cli`, host binary, and `ghostty/zig-out/lib/libghostty.so`, sets:

- `LD_LIBRARY_PATH=/home/riche/MCPs/limux/ghostty/zig-out/lib`
- `LIMUX_HOST_BIN=/home/riche/MCPs/limux/target/release/limux`

Then it execs `target/release/limux-cli`. It does not build, install, or fetch packages.

Current live runtime check:

```text
limux --json identify -> exit 0
name=limux-control
protocol=v1+v2
version=0.1.19
focused workspace cwd=/home/riche/Proj/SUPPLY_CHAIN_SECURITY
```

`limux --json health` returned `unknown command: health`; this is a CLI surface fact, not a runtime failure.

Current artifact hashes:

```text
b6073d450ffd40eb356206371a5d4dd5c1e7d8ff315175396689c434219c5f3c  scripts/limux-dev
d139b333f29ac545f7dadd3e2df8fff341872746e3fbe92bf7039c5cfc6ce6e3  target/release/limux-cli
7998f255e99ab55832e886231aaeede2714f8f7751fcdb5991b828999d8a38fd  target/release/limux
4f75d48afdb30bce61bc2f00adc02f1cb3f2ede0753c792e1dd189e27a908265  ghostty/zig-out/lib/libghostty.so
```

## Dependency Inventory

Workspace crates:

- `limux-protocol`
- `limux-core`
- `limux-control`
- `limux-ghostty-sys`
- `limux-host-linux`
- `limux-cli`

`Cargo.lock` package count: 153.

Direct Rust dependencies from `cargo tree --locked -e normal --workspace --depth 1`:

| Area | Direct dependencies |
|---|---|
| CLI | `anyhow 1.0.102`, `clap 4.6.0`, `dirs 6.0.0`, `serde 1.0.228`, `serde_json 1.0.149`, `tokio 1.50.0`, local `limux-control`, local `limux-protocol` |
| Core | `anyhow 1.0.102`, `serde 1.0.228`, `serde_json 1.0.149`, local `limux-protocol` |
| Control | `libc 0.2.x`, `tokio 1.50.0`, local protocol/core surfaces |
| Host | `gtk4 0.11.1`, `gdk4-wayland 0.11.0`, `libadwaita 0.9.1`, `webkit6 0.6.1`, `shell-quote 0.7.2`, `uuid 1.22.0`, `dirs 6.0.0`, local control/protocol/ghostty |
| Ghostty FFI | local `limux-ghostty-sys` plus build dependency `cc 1.2.57` |

Native/runtime dependencies observed locally:

```text
libwebkitgtk-6.0-4: 2.52.3-0ubuntu0.24.04.1 installed/candidate
libgtk-4-1:          4.14.5+ds-0ubuntu0.10 installed/candidate
libadwaita-1-0:      1.5.0-1ubuntu2 installed/candidate
```

Ghostty submodule:

```text
remote: https://github.com/am-will/ghostty.git
commit: 81ab8ffa90185221782baf785e85387321e16f8d
describe: xcframework-404a3f175ba6baafabc46cac807194883e040980-10-g81ab8ffa9
version marker: 1.3.0-dev
HEAD subject: embedded: gate paste on text clipboard availability
```

## Vulnerability And Advisory Results

### Rust Crates

An OSV batch query was run against all 153 locked crates with ecosystem `crates.io` and exact locked versions.

Result:

```text
queried 153 locked packages
packages_with_vulns 0
```

This means OSV returned no known advisories for the exact locked Rust crates at query time. It does not prove that every crate is uncompromised, and it does not cover native WebKitGTK/Ghostty runtime risk.

`cargo audit` is not installed on this host:

```text
cargo audit --version -> failed, no cargo-audit installed
```

I did not install `cargo-audit` because this review kept package execution and package installs at Level 0.

### Ghostty

GitHub lists five public Ghostty security advisories. The relevant current one is:

| Advisory | Severity | Affected | Patched | Local status |
|---|---:|---|---|---|
| `GHSA-4jxv-xgrp-5m3r` / `CVE-2026-26982` | Medium | `<= 1.2.3` | `1.3.0` | Current source is `1.3.0-dev`, and the fix commit `fe7427ed2 input: paste encoding replaces unsafe control characters with spaces (#10746)` is an ancestor of the Limux submodule HEAD. |
| `GHSA-5hcq-3j4q-4v6p` / `CVE-2024-56803` | Medium | `1.0.0` | `1.0.1` | Current source is past patched range. |
| `GHSA-98wc-794w-gjx3` | Low | `<= 1.0.1` | `1.1.0` | Current source is past patched range. |
| `GHSA-q9fg-cpmh-c78x` | Low | `< 1.2.0` | `1.2.0` | Current source is past patched range. |
| `GHSA-hfg5-8q2c-crhc` | Low | `1.0.0` | `1.0.1` | Current source is past patched range. |

Residual Ghostty risk: this repo uses an `am-will/ghostty` fork at a detached commit, not a clean upstream release tag. The current source appears to include the known upstream advisory fixes, but the already-built `libghostty.so` should still be treated as a local artifact whose provenance is "built in this repo" rather than a signed vendor package.

### WebKitGTK

This is the clearest current vulnerability flag.

Upstream WebKitGTK published `WSA-2026-0003` on 2026-06-02. It lists multiple CVEs affecting WebKitGTK/WPE WebKit before `2.52.4`, including memory handling, use-after-free, CSP enforcement, and sensitive-data access issues. Upstream stable `2.52.4` was released the same day.

Local Ubuntu state:

```text
apt-cache policy libwebkitgtk-6.0-4
Installed: 2.52.3-0ubuntu0.24.04.1
Candidate: 2.52.3-0ubuntu0.24.04.1
```

Local Ubuntu changelog top entry:

```text
webkit2gtk (2.52.3-0ubuntu0.24.04.1) noble-security; urgency=medium
Thu, 23 Apr 2026
```

That changelog includes earlier 2025/2026 CVEs, but not the June 2 `WSA-2026-0003` CVE set. Therefore, as of this review, the local WebKitGTK runtime should be treated as behind upstream security state.

Impact for Limux:

- Terminal/agent-pane use does not require opening web content.
- The host does include browser-tab UI code behind the default `webkit` feature (`rust/limux-host-linux/src/pane.rs`).
- The live CLI bridge still rejects browser `pane.create` requests (`rust/limux-host-linux/src/control_bridge.rs:421`), but the GTK UI has "new browser tab" paths.
- Recommendation: avoid browser/WebKit features until Ubuntu provides `2.52.4` or an equivalent backport is verified, or build/use a terminal-only host profile.

### Current Supply-Chain Threat Scan

Per the `SUPPLY_CHAIN_SECURITY` methodology, I used its local watcher in stdout-only mode against the configured source subset:

```text
python3 /home/riche/Proj/SUPPLY_CHAIN_SECURITY/security_posture/supply_chain_watch.py \
  --config /home/riche/Proj/SUPPLY_CHAIN_SECURITY/security_posture/config/sources.json \
  --state /tmp/limux-scs-watch-state.json \
  --out /tmp \
  --stdout \
  --include-seen \
  --min-score 20 \
  --timeout 20
```

The watcher checked 22 configured sources, including Socket, StepSecurity, Microsoft Security Blog, OX Security, SafeDep, Aikido, JFrog, Zscaler, OpenSourceMalware, Cybersecurity News, and The Hacker News. It surfaced many active npm/PyPI/AUR/VS Code/Open VSX/package-manager attacks, including Miasma/Shai-Hulud variants, compromised npm packages, PyPI packages, AUR hijacks, and developer-machine secret stealers.

No direct hit for `limux`, the locked Rust crate set, or the local Ghostty fork surfaced in the watcher output. This is not a "clean" attestation under the SCS mandatory registry, because the full post-by-post 12-month mandatory-source table was not reproduced here. It is enough to say the current public threat landscape reinforces the decision to avoid first-touch package builds/installers/global mutations on a secrets-bearing host.

## Code And Installer Findings

### F1 - WebKitGTK runtime is behind upstream security release

Severity: High for browser/WebKit features; Low for terminal-only use.

Evidence:

- Local `libwebkitgtk-6.0-4` is `2.52.3-0ubuntu0.24.04.1`.
- Upstream `WSA-2026-0003` affects versions before `2.52.4`.
- Browser tab code exists in `rust/limux-host-linux/src/pane.rs`.

Recommendation:

- Do not use Limux browser tabs for untrusted web content until WebKitGTK is updated/backported.
- If the immediate goal is terminal/session management, keep using terminal-only workflows.
- Consider a terminal-only host build/profile if WebKit packaging remains behind.

### F2 - Global installer is root-mutating and deletion-capable

Severity: High for trusted-host install operation; not a runtime RCE finding.

Evidence:

- `scripts/package.sh:461` generated installer re-execs through `sudo`.
- `scripts/package.sh:589-607` installs into `$PREFIX/bin`, `$PREFIX/libexec`, `$PREFIX/lib`, `$PREFIX/share`, writes `/etc/ld.so.conf.d/limux.conf`, and runs cache/loader updates.
- `scripts/package.sh:549-558` defines `remove_tree` using `find ... rm -f` and `rmdir`.
- `scripts/package.sh:561-583` uninstall path removes bins, libexec, libs, share files, desktop files, metainfo, icons, and loader config.

Recommendation:

- Do not run this installer on the trusted host as the next step.
- If a global install is still desired, first test the generated package in a disposable full VM.
- For host convenience now, create a no-root user-local install lane with a manifest and rollback, not this installer.

### F3 - Agent hook argument sanitizer preserves dangerous execution flags

Severity: Medium.

Evidence:

- `rust/limux-cli/src/agent_hooks.rs:180-239` sanitizes saved agent launch arguments.
- The test at `rust/limux-cli/src/agent_hooks.rs:500-524` confirms `--dangerously-bypass-approvals-and-sandbox` is intentionally preserved.
- The sanitizer also preserves other high-risk flags through `option_is_safe_flag_or_assignment`.

Impact:

Limux hook resume behavior can preserve an agent's unsafe launch posture. If an agent was started with sandbox/approval bypass flags, Limux can help rehydrate that mode rather than force a safer default.

Recommendation:

- Treat this as a product security fix candidate.
- Default-drop dangerous approval/sandbox bypass flags unless the user explicitly opts in with a separate Limux config setting.
- Update tests to assert the safer behavior.

### F4 - Ghostty FFI thread-safety invariant is asserted but not proven locally

Severity: Medium for maintainability/safety review; no observed exploit.

Evidence:

- `rust/limux-host-linux/src/terminal.rs:25-32` stores raw `ghostty_app_t` and declares `unsafe impl Send` and `unsafe impl Sync` with a brief comment.
- `limux-ghostty-sys` is raw FFI to `ghostty/include/ghostty.h`.

Impact:

The host relies on a native terminal engine and raw C/Zig FFI. That is expected for this product, but the thread-safety contract deserves a stronger local invariant or upstream reference before a system install is promoted.

Recommendation:

- Document which thread owns Ghostty calls and which calls are allowed cross-thread.
- Prefer confining Ghostty app/surface operations to the GTK/main-thread model where possible.

### F5 - Restored startup commands are logged to stderr

Severity: Low to Medium, depending on command contents.

Evidence:

- `rust/limux-host-linux/src/pane.rs:1245-1249` logs restored agent terminal commands.
- `rust/limux-host-linux/src/terminal.rs:1350-1355` logs `limux: starting restored terminal command=...`.

Impact:

If a restored command ever contains tokens, secrets, or sensitive paths, logs may capture them.

Recommendation:

- Log a redacted command summary or executable name only.
- Keep full command details in in-memory state where needed, not stderr.

### F6 - Socket fallback uses shared `/tmp` path when `XDG_RUNTIME_DIR` is absent

Severity: Low under current default; higher if `allowAll` is used.

Evidence:

- `rust/limux-control/src/socket_path.rs:14` fallback runtime socket is `/tmp/limux.sock`.
- `rust/limux-control/src/auth.rs:24-32` defaults to `LocalUser`.
- `rust/limux-control/src/auth.rs:50-68` enforces `SO_PEERCRED` uid checks.
- `rust/limux-control/src/request_io.rs:7-9` limits request length, connections, and idle timeout.

Impact:

The default same-user auth reduces risk. The shared `/tmp` fallback is still less clean than `$XDG_RUNTIME_DIR/limux/limux.sock`, especially if a user sets `LIMUX_SOCKET_MODE=allowAll` or uses explicit shared socket paths.

Recommendation:

- Prefer fail-closed or warn loudly if `XDG_RUNTIME_DIR` is missing for normal runtime sockets.
- Keep `allowAll` out of normal user docs except for tightly scoped testing.

## Positive Security Controls Observed

- Control socket uses `SO_PEERCRED` same-user authorization by default.
- Socket file permissions are set owner-only when owner-only modes are active.
- Request framing has `MAX_REQUEST_LEN=1MiB`, `MAX_CONNECTIONS=64`, and idle timeout.
- Terminal text payload validation rejects control characters except tab/newline/carriage return.
- File-drop shell payload generation uses `shell-quote`.
- Generated coordination/review files refuse symlink targets and use temp/create-new patterns in several paths.
- Host launch uses `Command::new`, not a shell, for key paths.
- Live GTK bridge rejects browser pane creation through `pane.create`; browser automation parity remains a separate surface.

## Practical Recommendation

For your immediate goal of using Limux to manage multiple sessions:

1. Keep using the current repo-local launcher for terminal/session work.
2. Avoid browser/WebKit tabs until WebKitGTK is updated or the risk is explicitly accepted.
3. Do not run `scripts/package.sh` or its generated `install.sh` globally on the trusted host.
4. Make the next implementation step a reviewed user-local install wrapper:
   - install only under `$HOME/.local/limux-reviewed/<limux-sha>/`
   - no `sudo`
   - no `/etc/ld.so.conf.d`
   - no `rm` cleanup of legacy paths
   - explicit manifest and SHA256 file
   - rollback by moving one symlink
5. Use the Project Isolation full disposable VM lane for `.deb` and true global/system install testing.

This is not saying Limux is unusable. It is saying the current safe lane is a real repo-local app launcher, while the risky lane is the global installer/system packaging path plus currently-lagging WebKitGTK.

## Post-Remediation Update - 2026-06-18

The remaining repo-controlled limitations identified in this review were narrowed:

- Agent hook resume safety: fixed. Limux no longer preserves dangerous agent launch flags such as `--dangerously-bypass-approvals-and-sandbox`, `--dangerously-skip-permissions`, `--full-auto`, or `--yolo` when capturing/restoring agent resume commands. Benign search toggles remain preservable. Covered by CLI and host restore tests.
- Ghostty resource recognition/staging: improved. Themes are now treated as optional; a valid bundle only needs shell integration plus Ghostty terminfo. The user-local installer also accepts explicit `--ghostty-share` and `--ghostty-terminfo` paths so a generated resource bundle can be staged without a global install.
- Browser/WebKit exposure: reduced by default. Embedded browser tabs are now runtime opt-in via `LIMUX_ENABLE_WEBKIT_BROWSER=1`; without that explicit opt-in, the browser button is hidden and browser tab creation is skipped. This keeps terminal/session management usable while WebKitGTK remains on the separate risk gate.

What remains genuinely gated:

- This machine currently has no `zig` binary and no installed `xterm-ghostty` terminfo database, so Ghostty terminfo could not be generated or copied during this remediation without a package-manager/toolchain install. The installer path is ready for a generated bundle, but the real bundle still requires either Zig availability or another trusted Ghostty terminfo source.
- WebKitGTK was not globally updated or replaced. Until the system package catches up or the operator explicitly accepts the browser risk, embedded browser use should stay disabled.

## Verification Commands Run

```text
git status --short --branch
git remote -v
git log --oneline --decorate -8
rg --files
cargo metadata --locked --format-version 1
cargo tree --locked -e normal --workspace --depth 1
cargo tree --locked -e normal --workspace --depth 2
cargo tree --locked -e normal -p limux-host-linux --depth 3
cargo tree --locked -e normal -p limux-control --depth 3
cargo tree --locked -e normal -p limux-cli --depth 3
cargo tree --locked -e build --workspace
cargo tree --locked --duplicates
cargo audit --version
limux --help
limux-cli --help
limux --json identify
limux --json health
limux-cli --json identify
rg -c '^name = ' Cargo.lock
git submodule status --recursive
apt-cache policy libwebkitgtk-6.0-4 libgtk-4-1 libadwaita-1-0
apt changelog libwebkitgtk-6.0-4
sha256sum scripts/limux-dev target/release/limux-cli target/release/limux ghostty/zig-out/lib/libghostty.so
python3 OSV batch query against 153 Cargo.lock package/version pairs
python3 /home/riche/Proj/SUPPLY_CHAIN_SECURITY/security_posture/supply_chain_watch.py --stdout --include-seen --min-score 20
GitHub Ghostty security advisory API query
Web search / source checks for WebKitGTK, Ubuntu packages, OSV API docs, Ghostty advisories, exact Limux/Ghostty/WebKit terms
```

Failures / limits:

- `cargo tree --locked -e normal,no-dev --workspace` failed because that event selector combination is invalid for Cargo.
- `cargo audit` is unavailable locally and was not installed.
- `limux --json health` is not a supported command.
- Full `./scripts/check.sh` was not run because this was a security/dependency review and the user is managing RAM pressure; the current runtime was verified with CLI help and identify instead.
- No full SCS "clean" attestation is claimed because that would require reproducing the mandatory post-by-post source table for the complete lookback window.

## Sources

Local project evidence:

- `AGENTS.md`
- `Cargo.toml`
- `Cargo.lock`
- `scripts/limux-dev`
- `scripts/package.sh`
- `rust/limux-cli/src/agent_hooks.rs`
- `rust/limux-control/src/auth.rs`
- `rust/limux-control/src/socket_path.rs`
- `rust/limux-control/src/request_io.rs`
- `rust/limux-host-linux/src/control_bridge.rs`
- `rust/limux-host-linux/src/pane.rs`
- `rust/limux-host-linux/src/terminal.rs`
- `rust/limux-protocol/src/lib.rs`
- `docs/install-security-report-2026-05-29.md`
- `docs/LIMUX_HOST_PREREQ_MUTATION_REVIEW_2026-05-29.md`
- `docs/LIMUX_GHOSTTY_ZIG_MUTATION_REVIEW_2026-05-29.md`
- `docs/project-isolation-lab-goal.md`

Supply-chain methodology sources:

- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/WHOLESALE_INSTALL_SECURITY_VETTING_METHOD_2026-06-09.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/PACKAGE_SURFACE_MANDATORY_SOURCE_REGISTRY.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/docs/LIMUX_INSTALL_VM_LAB_STRATEGY_2026-06-10.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/security_posture/README.md`
- `/home/riche/Proj/SUPPLY_CHAIN_SECURITY/security_posture/config/sources.json`

External primary/advisory sources checked:

- WebKitGTK security advisories: https://webkitgtk.org/security.html
- WebKitGTK WSA-2026-0003: https://webkitgtk.org/security/WSA-2026-0003.html
- WebKitGTK 2.52.4 release: https://webkitgtk.org/2026/06/02/webkitgtk2.52.4-released.html
- Ubuntu package page for `libwebkitgtk-6.0-4`: https://packages.ubuntu.com/noble/libwebkitgtk-6.0-4
- OSV API docs: https://google.github.io/osv.dev/api/
- Ghostty advisory `GHSA-4jxv-xgrp-5m3r`: https://github.com/ghostty-org/ghostty/security/advisories/GHSA-4jxv-xgrp-5m3r
- Ghostty advisory `GHSA-5hcq-3j4q-4v6p`: https://github.com/ghostty-org/ghostty/security/advisories/GHSA-5hcq-3j4q-4v6p
- Ghostty advisory `GHSA-98wc-794w-gjx3`: https://github.com/ghostty-org/ghostty/security/advisories/GHSA-98wc-794w-gjx3
