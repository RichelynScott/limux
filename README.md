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
- **Drag-and-drop** workspace reordering with favorites/pinning
- **Animated sidebar** collapse/expand

## Install

Download the latest release from [GitHub Releases](https://github.com/am-will/limux/releases).

**Debian/Ubuntu (.deb)** — recommended:
```bash
sudo dpkg -i ./limux_0.1.19_amd64.deb
```

**AppImage** — portable across Ubuntu 24.04-era desktops and newer, no install needed:
```bash
chmod +x Limux-0.1.19-x86_64.AppImage
./Limux-0.1.19-x86_64.AppImage
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
cargo build --release

# Run (point to libghostty.so location)
LD_LIBRARY_PATH=../ghostty/zig-out/lib:$LD_LIBRARY_PATH ./target/release/limux
```

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

## Agent integrations

Limux ships first-class hooks for coding agents (Codex, Claude Code, and
Gemini CLI). Every terminal limux spawns auto-exports
`LIMUX_WORKSPACE_ID` / `LIMUX_SURFACE_ID` / `LIMUX_PANE_ID` /
`LIMUX_TAB_ID` / `LIMUX_SOCKET`, so the CLI auto-targets the right place
with no flags needed from inside the agent's own terminal.

```bash
# Fire a libadwaita toast + sidebar unread badge from any agent
limux notify --subtitle "needs review" --body "blocked on auth choice" "Input needed"

# Install Limux session-restore hooks for supported agents
limux hooks setup

# Drop-in hook handlers translate hook JSON on stdin into notify/session state
echo '{"event":"stop"}' | limux claude-hook --event stop
echo '{"event":"finished"}' | limux gemini-hook --event finished

# Spin up a multi-agent collaboration team in the active workspace.
# Limux launches each agent CLI, writes LIMUX_AGENTS.md describing
# the <agent-msg> XML protocol, seeds durable roster/ledger files
# when missing, then sends each peer a short bootstrap prompt:
limux agent-team --agents codex,claude --cwd "$PWD"
# Use --no-bootstrap if you want panes launched but no post-launch prompt.
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
limux identify --json
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

Checked-in hook templates live in [`hooks/`](hooks/). They mirror
`limux hooks setup` for Codex, Claude Code, and Gemini CLI; OpenCode is
omitted until its hook integration is ready.

Coding agents working on **limux itself** should read [`AGENTS.md`](AGENTS.md)
and [`CLAUDE.md`](CLAUDE.md) in the repo root — those cover the build
loop, crate map, and the `feat/cmux-parity` roadmap tracked in
[`docs/cmux-parity-plan.md`](docs/cmux-parity-plan.md).

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
