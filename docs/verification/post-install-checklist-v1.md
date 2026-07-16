# Limux Post-Install Checklist v1

Version: v1
Status: frozen after first executed run; edits after that must create v2
Scope: operator-visible verification for preview builds before stable promotion

## Header

Fill this block in every run file.

| Field | Value |
|---|---|
| Checklist version | v1 |
| Checklist git SHA | `<git rev-parse HEAD:docs/verification/post-install-checklist-v1.md>` |
| Source SHA under test | `<git rev-parse HEAD>` |
| Install id | `<install-id>` |
| `limux-preview --version` | `<paste output>` |
| Date/time | `<YYYY-MM-DD HH:MM TZ>` |
| Operator | `<name>` |
| Overall verdict | `PASS` / `FAIL` |

## Setup

Run these from the Limux repo checkout that points at the source SHA being
verified. This installs an isolated preview lane and does not replace the
daily stable/legacy launcher.

```bash
git fetch origin
git switch main
git pull --ff-only origin main
git submodule update --init --recursive ghostty
ghostty_zig_args=(-Doptimize=ReleaseFast -Dcpu=baseline)
if ! pkg-config --exists gtk4-layer-shell-0 2>/dev/null; then
  ghostty_zig_args+=(-fno-sys=gtk4-layer-shell)
fi
(cd ghostty && zig build -Dapp-runtime=none "${ghostty_zig_args[@]}")
ghostty_resource_root="$(mktemp -d /tmp/limux-ghostty-resources.XXXXXX)"
(cd ghostty && DESTDIR="$ghostty_resource_root" zig build --prefix /usr "${ghostty_zig_args[@]}" -Demit-docs=false)
source_sha="$(git rev-parse --verify HEAD)"
install_id="preview-${source_sha:0:12}-$(date -u +%Y%m%dT%H%M%SZ)"
cargo build --release -p limux-cli --bin limux-cli
cargo build --release -p limux-host-linux --bin limux
scripts/user-local-install/install-user-local.sh --apply --profile release --channel preview --install-id "$install_id" --ghostty-share "$ghostty_resource_root/usr/share/ghostty" --ghostty-terminfo "$ghostty_resource_root/usr/share/terminfo"
~/.local/bin/limux-preview --version
```

Use `~/.local/bin/limux-preview` and `~/.local/bin/limux-preview-cli` for this
checklist. The preview wrapper exports `LIMUX_CHANNEL=preview:default`, so it
uses the preview socket/session namespace and must not interfere with stable.

Before launching preview, make sure a daily-driver Limux window is already
open. Prefer the stable lane when present; otherwise use the legacy daily
launcher so first-run verification does not dead-end before stable has ever
been promoted:

```bash
if [[ -x ~/.local/bin/limux-stable ]]; then
  daily_driver="$HOME/.local/bin/limux-stable"
elif [[ -x ~/.local/bin/limux ]]; then
  daily_driver="$HOME/.local/bin/limux"
else
  echo "No daily-driver Limux launcher found at ~/.local/bin/limux-stable or ~/.local/bin/limux" >&2
  exit 1
fi
nohup "$daily_driver" >/tmp/limux-daily-driver-checklist.log 2>&1 &
```

After the daily-driver window is open, launch preview:

```bash
~/.local/bin/limux-preview
```

Launcher resolution for promotion: if a full v1 run passes, promotion installs
the verified source to the stable lane and the operator relaunches with
`~/.local/bin/limux-stable` or the `Limux Stable` desktop entry. The legacy
`~/.local/bin/limux` launcher remains untouched unless a separate legacy-lane
update is explicitly requested.

## Symptom Split

| Symptom | Likely class |
|---|---|
| Typing corrupted / keys act as shortcuts | keyboard-modifier (#14) or Ghostty resource shape (PRD-B) |
| `?` / boxed glyphs in prompt | environmental font (Nerd Font / Powerlevel10k) — NOT a Limux input bug |
| `00~...01~` around pasted text | bracketed-paste shell mode — NOT a Limux input bug |

## Rules

Write-back rule: a fix PR that changes operator-visible behavior is not `done`
until a checklist run, full or relevant subset, records its verdict.

Versioning rule: v1 is frozen after the first run. Any checklist content change
after that creates `post-install-checklist-v2.md`; run files must name the
checklist version and git SHA they executed.

Promotion rule: subset runs may close individual tasks but never promote. If
all items pass on a full run, promote from the same source SHA with:

```bash
scripts/user-local-install/install-user-local.sh --apply --profile release --channel stable --install-id <verified-sha-id>
```

The stable install is a fresh stable-channel install of the verified source,
not a symlink hand edit. If any item fails, do not promote. Record verdict and
evidence, reopen or add TaskMaster tasks, and attach the run file.

Failure evidence rule: for #14-style typing, shortcut, or paste failures,
capture a focused rerun with `LIMUX_DEBUG_KEYS=1` and include the log path in
the run file.

## Checklist Items

Each item must be marked exactly one of `PASS`, `FAIL`, or `N/A`.

### 1. Build Identity And Doctor

Action:
1. In a preview pane, run:
   ```bash
   ~/.local/bin/limux-preview --version
   ~/.local/bin/limux-preview doctor --json
   ```
2. Confirm the version output includes source SHA, install id, and
   `channel=preview:default`.

Expected result:
- Build identity matches the source SHA and install id in the run header.
- `doctor` reports no stale launcher, resource, socket, or build drift for the
  preview lane.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 2. Fresh Pane Typing, Modifier Chords, And Paste

Action:
1. Open a fresh terminal pane in preview.
2. Type a normal sentence and several shell metacharacters.
3. With no terminal selection, press `Ctrl+C` and confirm the foreground shell
   or TUI receives its normal interrupt/cancel input.
4. Select terminal text, press `Ctrl+C`, and confirm the selection is copied
   without interrupting or clearing the foreground shell or TUI.
5. Use `Ctrl+D`, arrow keys, and Backspace normally.
6. Copy text outside Limux and paste with `Ctrl+Shift+V`.
7. Press plain `Ctrl+V` at a shell prompt.

Expected result:
- Plain typing stays literal and does not trigger Limux shortcuts.
- `Ctrl+C` copies and is consumed only while terminal text is selected; without
  a selection it retains normal terminal interrupt/cancel behavior.
- `Ctrl+Shift+V` performs Limux terminal paste.
- Plain `Ctrl+V` is intentionally unclaimed by Limux and passes through to the
  terminal/native shell behavior.
- No unexpected `?`, `00~...01~`, repeated text, or shortcut storm appears.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 3. Mouse Selection Copy And Stuck-Click Watch

Action:
1. Drag-select a sentence in a Limux pane.
2. Release the mouse button.
3. Paste into another pane or external editor.
4. Move the mouse across panes and click normal UI controls.

Expected result:
- Selection completes only after release.
- Auto-copy behavior, if enabled by the runtime, copies the final intended
  selection.
- No pane remains stuck in left-click selection mode.
- Other panes and UI controls remain clickable.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 4. Window Controls And Edge Hitbox

Action:
1. Confirm the window exposes close, minimize, and maximize/fullscreen controls
   according to the current Limux chrome policy.
2. Click each window control.
3. Trace the right and bottom window edges with the pointer.
4. Resize from each edge and corner.
5. Try clicking a visible window behind Limux near the right and bottom edges.

Expected result:
- Window controls work and do not leave the app in a broken chrome state.
- Edge hitbox is not visibly far outside the window border.
- Corners are practical to grab.
- Limux does not block clicks on underlying windows far outside its visible
  border.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 5. Drag-Resize Soak With Live Agent TUI

Action:
1. Start a live agent TUI in one pane, such as `codex` or `claude`.
2. Split the workspace into at least two panes.
3. Drag-resize pane splitters repeatedly for at least 30 seconds.
4. Continue typing in the live TUI after resizing.

Expected result:
- Terminal content reflows without durable mangling.
- The TUI remains usable.
- No resize storm, freeze, stuck input, or persistent visual corruption occurs.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 6. Sidebar Resize, Collapse, And Restore

Action:
1. Drag the workspace sidebar width handle left and right.
2. Collapse the sidebar.
3. Restore it with the visible ribbon/button.
4. Repeat after switching workspaces.

Expected result:
- Sidebar width changes predictably within configured limits.
- Titles and paths stay readable until the minimum readable size, then truncate.
- Collapse and restore work repeatedly.
- The sidebar does not steal pane space after collapse.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 7. Multi-Workspace Session Restore

Action:
1. Create at least two workspaces with different pane layouts.
2. Open at least one split pane.
3. Set a flag color on one pane and record the color in Evidence.
4. Close the preview Limux window.
5. Relaunch with `~/.local/bin/limux-preview`.

Expected result:
- Workspaces restore.
- Pane splits restore.
- The flagged pane restores with the same flag color.
- Active tabs and saved operator-visible state restore.
- No duplicate `terminal-0` stack warning causes broken UI.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 8. Notification Toast, Sidebar Dot, And Pane Attention

Action:
1. From a non-focused preview pane or another terminal targeting preview, run:
   ```bash
   ~/.local/bin/limux-preview notify --title "Checklist ping" --body "Preview notification"
   ```
2. Observe desktop toast and sidebar row state.
3. In a right-hand preview pane, run a delayed pane-originating desktop
   notification:
   ```bash
   sleep 2; printf '\033]777;notify;Limux checklist;Pane attention\a'
   ```
4. Before the notification fires, focus a different pane.
5. Observe the pane attention border on the pane that emitted the notification.

Expected result:
- Toast appears when configured.
- Workspace row gets the expected unread/sidebar marker.
- The CLI `notify` command is treated as workspace-only and is not required to
  draw a pane border.
- The pane-originating OSC 777 notification gets a visible blue border overlay.
- The marker clears according to configured hover/focus behavior.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 9. Runtime Channel Isolation

Action:
1. Ensure a daily-driver Limux runtime is open before launching preview. Use
   `~/.local/bin/limux-stable` when present, or fall back to the legacy
   `~/.local/bin/limux` launcher on the first verification cycle before stable
   promotion exists.
2. Run the scripted smoke:
   ```bash
   bash scripts/tests/runtime-isolation-smoke.sh
   ```
3. Launch the preview runtime with `~/.local/bin/limux-preview`.
4. Use both stable and preview windows briefly.

Expected result:
- The smoke prints `runtime-isolation-smoke: PASS`.
- Daily-driver and preview sockets/session dirs are distinct.
- Preview does not target or mutate the daily-driver socket.
- Daily-driver and preview windows can coexist without copy/paste, border, or
  workspace interference.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

### 10. Pane Attention Overlay And Per-Pane Flags

Action:
1. In a right-hand split pane, run a delayed pane-originating desktop
   notification:
   ```bash
   sleep 2; printf '\033]777;notify;Limux checklist;Pane flag attention\a'
   ```
2. Before the notification fires, focus a different pane.
3. Right-click a tab in the right-hand pane and set a flag color.
4. In the flagged right-hand pane, run a second delayed pane-originating
   desktop notification:
   ```bash
   sleep 2; printf '\033]777;notify;Limux checklist;Flagged pane attention\a'
   ```
5. Before the second notification fires, focus a different pane.
6. Clear the flag color.

Expected result:
- The blue attention border is visible around the actual pane, including
  right-hand split panes.
- A durable per-pane flag color appears without hiding unread/attention state.
- Clearing the flag removes the flag border and leaves unrelated unread state
  alone.

Verdict: `PASS` / `FAIL` / `N/A`
Evidence:

## Run Closeout

If all ten items pass, record `Overall verdict: PASS` in the run file, then
promote from the same source SHA to the stable lane with the command in the
Promotion rule. If any item fails, record `Overall verdict: FAIL`, do not
promote, and create or reopen TaskMaster tasks with evidence.
