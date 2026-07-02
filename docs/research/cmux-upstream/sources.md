# Source Snapshot

Snapshot date: 2026-07-02

## GitHub Sources

### manaflow-ai/cmux

- Repository: <https://github.com/manaflow-ai/cmux>
- Description: Open source Ghostty-based macOS terminal with vertical tabs and
  notifications for AI coding agents.
- Default branch: `main`
- Latest release observed: `v0.64.17`, published 2026-06-23.
  <https://github.com/manaflow-ai/cmux/releases/tag/v0.64.17>
- Repository activity observed: pushed on 2026-07-02.
- Commands used:
  - `gh repo view manaflow-ai/cmux --json nameWithOwner,description,url,defaultBranchRef,pushedAt,latestRelease`
  - `gh release list --repo manaflow-ai/cmux --limit 20`
  - `gh pr list --repo manaflow-ai/cmux --state all --limit 60 --json ...`
  - `gh issue list --repo manaflow-ai/cmux --state all --limit 60 --json ...`

Recent themes observed from primary GitHub data:

- iOS/mobile terminal rendering, keyboard, and scroll correctness.
- Remote tmux and SSH persistence.
- Workspace/sidebar colors, statuses, groups, and notification attribution.
- Browser automation and domain allowlisting.
- Session resume, cwd inheritance, split/new-tab correctness.
- Git observation without optional locks.
- Update UX and multi-runtime channel hygiene.
- Performance problems with many active/hidden agent panes.

### am-will/limux

- Repository: <https://github.com/am-will/limux>
- Description: GPU-accelerated terminal multiplexer for Linux.
- Default branch: `main`
- Latest release observed: `v0.1.19`, published 2026-05-13.
  <https://github.com/am-will/limux/releases/tag/v0.1.19>
- Repository activity observed: pushed on 2026-05-13.
- Upstream `main` observed at `9ffc9341de2ad649f99a85df7c05b7eafb4a6236`.
- Commands used:
  - `gh repo view am-will/limux --json nameWithOwner,description,url,defaultBranchRef,pushedAt,latestRelease`
  - `gh pr list --repo am-will/limux --state all --limit 80 --json ...`
  - `gh issue list --repo am-will/limux --state all --limit 80 --json ...`
  - `git ls-remote --heads upstream`

Observed upstream branches:

- `main` `9ffc934`: already ancestor of this fork.
- `docs/agent-hooks` `8625a4c`: already ancestor of this fork.
- `fix/issue-66-appimage-webkitgtk` `e4e010a`: already ancestor of this fork.
- `save-sesh` `aae3265`: already ancestor of this fork.
- `fix/render-throttling` `4c50b6c`: old divergent branch; do not merge
  directly.

Recent upstream Limux themes observed from primary GitHub data:

- HiDPI/fractional scaling and physical-pixel Ghostty sizing.
- Dead-key/IME input on Wayland and international keyboard correctness.
- Ctrl+W close-tab behavior.
- UI font/settings work.
- SVG loader/AppImage packaging issues.
- Platform packaging for ARM, NixOS, Flatpak, and AUR.
- Runtime/socket ownership when launching Limux twice.

## Local Sources

- `docs/cmux-parity-plan.md`
- `docs/future-improvements/limux-runtime-channel-contract-20260702.md`
- `docs/future-improvements/limux-runtime-isolation-surface-audit-20260702.md`
- `docs/future-improvements/limux-cursor-ide-integration-plan-20260630.md`
- `docs/future-improvements/limux-pane-attention-border-and-color-flags-20260701.md`
- `docs/future-improvements/limux-runtime-isolation-and-window-ui-plan-20260701.md`
- `.taskmaster/tasks/tasks.json`
- `rust/limux-control/src/socket_path.rs`
- `rust/limux-host-linux/src/control_bridge.rs`
- `rust/limux-host-linux/src/window.rs`
- `rust/limux-host-linux/src/terminal.rs`
- `rust/limux-cli/src/main.rs`

## Subagent Lenses

Three read-only Codex subagents contributed compressed summaries:

- cmux upstream product/feed lens.
- upstream Limux PR/issue/branch lens.
- local Limux docs/code taxonomy lens.

The database below integrates those reports with the main session's GitHub API
queries. The subagent reports were used as synthesis input; this directory is
the durable record.
