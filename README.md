# Limux

A GPU-accelerated terminal workspace manager for Linux, powered by Ghostty's rendering engine. A special thanks to the cmux contributors who inspired this build. 

If you are on Mac, please visit https://github.com/manaflow-ai/cmux to download the original. 

https://github.com/user-attachments/assets/6f3047c2-e2b6-49f2-b536-570a1570d0f8

## Features

- **GPU-rendered terminals** via embedded Ghostty (OpenGL)
- **Workspaces** with folder-based naming, persistence across restarts, and sidebar management
- **Split panes** (horizontal/vertical) with keyboard navigation
- **Tabbed terminals** within each pane
- **Built-in browser** (WebKitGTK)
- **Right-click context menu** with copy, paste, split, clear
- **Drag-and-drop** workspace reordering with favorites/pinning and manual highlights
- **Animated sidebar** collapse/expand
- **CLI diagnostics** with `doctor`, `target-info`, runtime identity, and log triage
- **User-local stable/preview channels** for testing new Limux builds without
  replacing the daily-driver runtime
- **Agent hooks and pane orchestration** for Codex, Claude Code, Gemini, opt-in
  OpenCode hooks, Hermes receiver events, and hcom-launched sessions

## Install

Download the latest release from [GitHub Releases](https://github.com/am-will/limux/releases).

**Debian/Ubuntu (.deb)** — recommended:
```bash
sudo dpkg -i ./limux_0.2.0_amd64.deb
```

**AppImage** — portable across Ubuntu 24.04-era desktops and newer, no install needed:
```bash
chmod +x Limux-0.2.0-x86_64.AppImage
./Limux-0.2.0-x86_64.AppImage
```

Release AppImages are built and checked on the Ubuntu 24.04 `GLIBC_2.39`
floor. Limux still uses the host GTK4, libadwaita, and WebKitGTK runtime
libraries, so older distributions may need the `.deb`, tarball, or a source
build with matching system packages instead.

**Tarball** — manual install:
```bash
tar xzf limux-*-linux-x86_64.tar.gz
cd limux-*-linux-x86_64
sudo ./install.sh
```

**Arch Linux (unofficial AUR package)** — community-maintained by [antonbarchukov](https://github.com/antonbarchukov):
```bash
yay -S limux-bin
```

The AUR package is available at [`limux-bin`](https://aur.archlinux.org/packages/limux-bin). Thanks to [antonbarchukov](https://github.com/antonbarchukov) for packaging Limux for Arch users. Arch packaging is not currently maintained by upstream; please report AUR packaging issues to the package maintainer first. See [issue #5](https://github.com/am-will/limux/issues/5).

To uninstall:
```bash
# deb
sudo apt remove limux

# tarball
sudo ./install.sh --uninstall
```

### System dependencies

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-1 libadwaita-1-0 libwebkitgtk-6.0-4
```

## Runtime identity and diagnostics

Installed packages expose `limux` as the user-facing CLI. Running `limux` with
no arguments launches the GTK app; commands and diagnostics are handled by the
same CLI entrypoint.

```bash
limux --version
limux target-info
limux doctor --json
limux doctor --log-triage --lines 200
```

`--version` reports the CLI version and build identity. User-local installs also
read `install-info.json` beside the executable, so version output can include
the install id and channel.

Example:

```text
limux-cli 0.2.0 (abcdef123456, release) install-id=stable-abcdef123456 channel=stable
```

`target-info` / `socket-info` prints the resolved socket and channel without
connecting to a running host. Use it when checking whether a shell is targeting
the default runtime, stable runtime, or a preview runtime.

JSON flag placement is a parser gotcha: most commands use the global flag
before the subcommand, such as `limux --json identify`, while `doctor --json`
is a subcommand-local exception.

`doctor` checks launchers, running processes, control socket reachability,
stale sockets, Ghostty resource packaging, and optional log triage. Exit code
`0` means all checks passed, `1` means at least one check failed, and `2` means
warnings were found but no check failed. `--log-triage` summarizes common
runtime log warnings such as Mesa/GDK environment issues without requiring a
full manual log scrape.

## Build from source

### Prerequisites

- Rust toolchain (stable)
- Zig
- GTK4, libadwaita, WebKitGTK dev packages
- Initialized Ghostty submodule

```bash
# Install dev dependencies (Ubuntu/Debian)
sudo apt install libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev pkg-config build-essential

# Initialize the Ghostty submodule and build the embedded library
git submodule update --init --recursive
(cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast)

# Build limux
cargo build --release -p limux-cli --bin limux-cli
cargo build --release -p limux-host-linux

# Run from this checkout through the public CLI entrypoint.
./scripts/limux-dev
```

`scripts/limux-dev` launches the release CLI, points it at the sibling host
binary, and prepends the checkout's `ghostty/zig-out/lib` directory to
`LD_LIBRARY_PATH`. To make the checkout build available as normal shell
commands without a sudo install, symlink it from a directory on `PATH`:

```bash
mkdir -p "$HOME/.local/bin"
ln -s "$PWD/scripts/limux-dev" "$HOME/.local/bin/limux"
ln -s "$PWD/scripts/limux-dev" "$HOME/.local/bin/limux-cli"
```

Use `LIMUX_LOCAL_PROFILE=debug scripts/limux-dev` when you want to run the
debug binaries instead of the release binaries.

### User-local stable and preview channels

For development, prefer user-local channel installs over replacing the runtime
you are actively using for work. The installer can create isolated launcher
lanes:

```bash
# Traditional launcher names: ~/.local/bin/limux and ~/.local/bin/limux-cli
scripts/user-local-install/install-user-local.sh --apply --channel legacy --profile release

# Daily-driver candidate: ~/.local/bin/limux-stable and limux-stable-cli
scripts/user-local-install/install-user-local.sh --apply --channel stable --profile release

# Test runtime: ~/.local/bin/limux-preview and limux-preview-cli
scripts/user-local-install/install-user-local.sh --apply --channel preview --profile release

# Named test runtime: ~/.local/bin/limux-preview-lab and limux-preview-lab-cli
scripts/user-local-install/install-user-local.sh --apply --channel preview:lab --profile release
```

Each install records `install-info.json` with the install id, channel, source
SHA, and creation time. Channel-aware launchers pass the selected lane to the
CLI so preview runtimes can be launched and tested without disturbing the
stable/daily-driver socket and state.

### Package a release tarball

```bash
./scripts/package.sh
```

This builds the binary, bundles `libghostty.so`, icons, and an install script into a tarball.
`package.sh` also rebuilds `libghostty.so` with `ReleaseFast` and `-Dcpu=baseline`, so Zig and the initialized Ghostty submodule must be present.

## Development

Run the canonical local quality gate before committing:

```bash
./scripts/check.sh
```

Repository maintainability rules live in [`docs/maintainability.md`](docs/maintainability.md).

When validating user-local installs, also check
[`docs/terminal-input-regression-20260701.md`](docs/terminal-input-regression-20260701.md).
It records the June 2026 Ghostty resource packaging regression and the rule
that `ghostty/src` must not be installed or resolved as runtime resources.

Ghostty runtime resource packaging has its own regression check:

```bash
bash scripts/tests/validate-ghostty-resources.sh
```

For preview-to-stable promotion, use the checklist workflow in
[`docs/verification/post-install-checklist-v1.md`](docs/verification/post-install-checklist-v1.md)
and record each verification run with
[`docs/verification/run-template.md`](docs/verification/run-template.md).
Stable promotion should wait for a full PASS on the preview runtime.

## Agent integrations

Limux ships first-class hooks for coding agents (Codex, Claude Code, Gemini CLI,
and Hermes receiver events). Every terminal limux spawns auto-exports
`LIMUX_WORKSPACE_ID` / `LIMUX_SURFACE_ID` / `LIMUX_PANE_ID` /
`LIMUX_TAB_ID` / `LIMUX_SOCKET`, so the CLI auto-targets the right place
with no flags needed from inside the agent's own terminal.

```bash
# Fire a libadwaita toast + sidebar unread badge from any agent
limux notify --subtitle "needs review" --body "blocked on auth choice" "Input needed"

# Manually flag the current pane so you can return to it later
limux pane-action --action set_flag_color --color orange
limux pane-action --action clear_flag_color

# Install Limux session-restore hooks for supported agents
limux hooks setup
# Default setup covers Codex, Claude Code, and Gemini. OpenCode is opt-in:
limux hooks setup opencode

# Drop-in hook handlers translate hook JSON on stdin into notify/session state
echo '{"event":"stop"}' | limux claude-hook --event stop
echo '{"event":"finished"}' | limux gemini-hook --event finished
# Hermes lifecycle plugin payloads are receiver-only in Limux; installation is
# owned by Hermes/hcom, while Limux handles the notification/restore event.
echo '{"event":"pre_approval_request","extra":{"session_id":"h1","cwd":"'"$PWD"'"}}' \
  | limux hermes-hook --event pre_approval_request

# Spin up a multi-agent collaboration team in the active workspace.
# Limux launches each agent CLI, writes LIMUX_AGENTS.md describing
# the <agent-msg> XML protocol, seeds durable roster/ledger files
# when missing, then sends each peer a short bootstrap prompt:
limux agent-team --agents codex,claude --cwd "$PWD"
# Use --no-bootstrap if you want panes launched but no post-launch prompt.
# Use --launch-mode hcom when your normal entrypoint is hcom:
limux agent-team --agents codex,claude --launch-mode hcom --cwd "$PWD"
# This launches peer panes with `hcom codex --run-here` and
# `hcom claude --run-here`, keeping those sessions inside Limux panes.
# Hermes can be included too:
limux agent-team --agents codex,claude,hermes --launch-mode hcom --cwd "$PWD"
# → Codex and Claude can now do:
#   limux send --surface "<peer-surface-id>" \
#     $'<agent-msg from="codex" to="claude" id="…" ts="…">…</agent-msg>\n'
#   limux send-key --surface "<peer-surface-id>" enter
# Text typed into terminal panes allows printable Unicode plus tab/LF/CR.
# Use send-key for control keys; send/new-pane command text rejects ESC/BEL/CSI.

# Or split the current agent's pane and launch another terminal agent.
# Inside Limux, workspace/surface/pane default from LIMUX_*:
limux new-pane --direction right --command 'claude'
# Live GTK self-spawn currently supports terminal panes only.

# Explicit source targets are also accepted and serialized unchanged:
limux new-pane --workspace "$LIMUX_WORKSPACE_ID" --surface "$LIMUX_SURFACE_ID" \
  --pane "$LIMUX_PANE_ID" --direction down --command 'codex'

# Keep both agents in the same workspace on separate splits/tabs:
limux --json identify
limux list-panels --workspace "$LIMUX_WORKSPACE_ID"
limux send --workspace "$LIMUX_WORKSPACE_ID" --surface "<peer-surface-id>" \
  $'<agent-msg from="codex" to="claude" id="…" ts="…">…</agent-msg>\n'
```

See the auto-generated `LIMUX_AGENTS.md` (written into the shared cwd by
default) for the full protocol spec, peer table, durable coordination file
pointers, instruction-source pointers, and editable Policies section. Generated
files include a stable marker and an `Instruction Sources` table for detected
`AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` files, including path, modified time,
and deterministic content hash metadata. Limux points agents to read those files
directly; it does not copy or merge their contents.

Use `--protocol-path <path>` to write the generated protocol elsewhere.
Existing unmarked protocol files are not overwritten by default; use
`--force-protocol-overwrite` only when the target file is safe to replace.
`agent-team` no longer writes `AGENTS.md` by default, so existing repo
instructions are not clobbered. Put durable team policy notes that should
survive regeneration in `LIMUX_AGENTS.local.md`; Limux documents that sidecar
but does not create or overwrite it.

`agent-team` also seeds `LIMUX_TEAM_ROSTER.md` and `LIMUX_REVIEW_LEDGER.md`
when missing. The roster maps projects, owners, hcom names, related teams,
routing rules, and durable coordination files; live workspace/pane/surface IDs
stay in the freshly generated `LIMUX_AGENTS.md` runtime protocol to avoid stale
durable routing. The review ledger is for reviewer findings, consensus
decisions, accepted risks, and cross-team notifications that should not live
only in terminal scrollback. Existing roster and ledger files are preserved by
default; use `--roster-path <path>`, `--ledger-path <path>`, and
`--force-roster-overwrite` only when an intentional alternate path or marked
roster reset is needed. Symlink and non-regular roster/ledger targets are
refused.

If you normally start agents with hcom, add `--launch-mode hcom` to
`agent-team` or `review spawn`. Limux will create normal terminal panes, but the
pane command becomes `hcom <agent> --run-here` so hcom registers the session
without opening a separate external terminal window.

Keep the bus boundary clear:

- Limux's Unix socket is the local GUI control bus for panes, workspaces,
  terminal text, notifications, and screen reads.
- hcom is the cross-agent messaging and session bus for named agents,
  transcripts, durable messages, resume/fork, and multi-project coordination.
- `limux notify` creates user-visible Limux attention such as a toast/sidebar
  badge; `hcom send` sends a message to another hcom agent and is not a pane
  notification by itself.
- Agents launched inside Limux inherit `LIMUX_WORKSPACE_ID`,
  `LIMUX_SURFACE_ID`, `LIMUX_PANE_ID`, `LIMUX_TAB_ID`, and `LIMUX_SOCKET`.
  hcom-launched workers can use those values to call back into the correct
  Limux pane.

`--dry-run` does not contact a running Limux host, but it still materializes the
generated protocol and seeds missing roster/ledger files so agents can inspect
the exact outputs. Use temporary `--protocol-path`, `--roster-path`, and
`--ledger-path` values when you want a preview outside the repo root.

Prepare a durable review request without launching another agent:

```bash
limux review prepare \
  --artifact rust/limux-cli/src/main.rs \
  --reviewer claude \
  --lens security \
  --summary "Review the Phase 5D scaffold for blockers"
```

`review prepare` creates `reviews/<review-id>.md`, appends a pending entry to
`LIMUX_REVIEW_LEDGER.md`, and prints the exact reviewer prompt. It does not
contact the Limux host, split panes, or run reviewer CLIs. Use `--dry-run` to
preview paths, Markdown, and prompt text without writing files. Use
`--review-id`, `--reviews-dir`, and `--ledger-path` when you need deterministic
paths for a coordinated review. Existing request files, symlink targets, and
non-regular ledger paths are refused at the output leaf. Use trusted output
directories; Limux does not recursively audit every parent path component for
symlinks.

Launch a prepared reviewer request into a new pane:

```bash
limux review spawn --review-id <review-id>
```

`review spawn` reads the existing generated request, starts one reviewer
terminal pane beside the current pane, sends the prepared prompt after pane
creation, writes `reviews/<review-id>.evidence.md`, and updates the matching
pending ledger entry to `in-progress`. The evidence file is a pointer to the
live reviewer surface and a suggested `read-screen` capture command; it is not a
raw transcript dump. Use `--dry-run` to validate the request/ledger/evidence
paths without host contact, and `--no-launch` to create the pane without typing
the reviewer command or prompt.

Checked-in hook templates live in [`hooks/`](hooks/). They mirror
`limux hooks setup` for Codex, Claude Code, and Gemini CLI. OpenCode hook
installation is opt-in with `limux hooks setup opencode`. Hermes notification
receivers are supported through `limux hooks hermes <event>` / `limux
hermes-hook`, but Hermes-side lifecycle plugin installation remains external.

Coding agents working on **limux itself** should read [`AGENTS.md`](AGENTS.md)
and [`CLAUDE.md`](CLAUDE.md) in the repo root — those cover the build
loop, crate map, and the `feat/cmux-parity` roadmap tracked in
[`docs/cmux-parity-plan.md`](docs/cmux-parity-plan.md).

## Control bridge status

The live GTK bridge is the production path for user-visible CLI behavior. It
currently supports workspace, pane, surface, terminal send/key/read/health,
notification, and terminal pane-create commands. PRD-E live-bridge parity is
still partial: only `window.list` and `window.current` are read-only
state-mirror fallthrough methods today. Browser-pane bridge parity and broader
mirror API routing remain separate work until their PRD tasks are completed.

## Keyboard shortcuts

Most default shortcuts use `Ctrl`. Fullscreen defaults to `F11`. Custom remaps may also use `Cmd`, which Limux maps to either the Linux `Meta` or `Super` modifier. `Opt` maps to `Alt`.

### App

| Shortcut | Action |
|---|---|
| `Ctrl+Q` | Quit Limux |
| `Ctrl+Alt+N` | Open a new Limux instance |
| `F11` | Toggle fullscreen |

### Browser

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+L` | Open the focused browser page in a new split |
| `Ctrl+L` | Focus browser address bar |
| `Ctrl+[` | Browser back |
| `Ctrl+]` | Browser forward |
| `Ctrl+R` | Browser reload |
| `Ctrl+Alt+I` | Open Web Inspector |
| `Ctrl+Alt+C` | Open Web Inspector (console-only targeting is not exposed by WebKitGTK) |

### Find

| Shortcut | Action |
|---|---|
| `Ctrl+F` | Open find on the focused terminal or browser |
| `Ctrl+G` | Find next |
| `Ctrl+Shift+G` | Find previous |
| `Ctrl+Shift+F` | Hide find |
| `Ctrl+E` | Use selection for find |

### Terminal

| Shortcut | Action |
|---|---|
| `Ctrl+K` | Clear scrollback |
| `Ctrl+Shift+C` | Copy selection |
| `Ctrl+Shift+V` | Paste |
| `Ctrl++` | Increase font size |
| `Ctrl+-` | Decrease font size |
| `Ctrl+Shift+0` | Reset font size |

### Workspace And Pane

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+N` | New workspace (folder picker) |
| `Ctrl+Shift+W` | Close workspace |
| `Ctrl+Shift+Left/Right` | Cycle tabs in focused pane |
| `Ctrl+Shift+D` | Split down |
| `Ctrl+Shift+T` | New terminal tab in the focused pane |
| `Ctrl+D` | Split right |
| `Ctrl+W` | Close focused pane |
| `Ctrl+Shift+Z` | Toggle focused pane zoom |
| `Ctrl+M` | Toggle sidebar |
| `Ctrl+Shift+M` | Toggle top bar |
| `Ctrl+T` | New terminal tab |
| `Ctrl+Arrow` | Focus pane in direction |
| `Ctrl+PageDown/Up` | Next or previous workspace |
| `Ctrl+1-9` | Switch to workspace by number |

## Architecture

```
rust/
  limux-host-linux/    # GTK4/Adwaita UI (window, sidebar, panes, tabs)
  limux-ghostty-sys/   # FFI bindings to libghostty
  limux-core/          # Command dispatcher and state engine
  limux-protocol/      # Socket wire format types
  limux-control/       # Unix socket server
  limux-cli/           # CLI client
```

The terminal rendering is handled entirely by Ghostty's embedded library (`libghostty.so`), which provides GPU-accelerated OpenGL rendering. The UI layer is native GTK4 with libadwaita.

## License

MIT
