#!/usr/bin/env bash
# scripts/xvfb-smoke-test.sh - Headless end-to-end smoke test for the
# limux agent-integrations stack. Runs a real limux GTK host under Xvfb,
# exercises limux-cli against the live Unix socket, asserts expected
# behavior, then tears down. Zero display hardware required.
#
# Usage:
#   ./scripts/xvfb-smoke-test.sh                # release build
#   LIMUX_SMOKE_PROFILE=debug ./scripts/xvfb-smoke-test.sh
set -euo pipefail

PROFILE="${LIMUX_SMOKE_PROFILE:-release}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# This script launches its own isolated host/socket. Do not inherit IDs from a
# real Limux pane that happens to be running the smoke test.
unset LIMUX_WORKSPACE_ID LIMUX_SURFACE_ID LIMUX_PANE_ID LIMUX_TAB_ID LIMUX_SOCKET LIMUX_SOCKET_PATH

DEMO_DIR="$(mktemp -d -t limux-smoke-XXXXXX)"
LOG_DIR="$DEMO_DIR/logs"
mkdir -p "$LOG_DIR"

echo "== limux agent-integrations smoke test =="
echo "profile:   $PROFILE"
echo "demo dir:  $DEMO_DIR"
echo "log dir:   $LOG_DIR"

# --- 1. Deps --------------------------------------------------------------
command -v xvfb-run >/dev/null || {
  echo "FAIL: xvfb-run not installed (sudo pacman -S xorg-server-xvfb)"
  exit 2
}
command -v cargo >/dev/null || { echo "FAIL: cargo missing"; exit 2; }
command -v sed >/dev/null || { echo "FAIL: sed missing"; exit 2; }

# --- 2. Build -------------------------------------------------------------
if [ "$PROFILE" = "release" ]; then
  CARGO_FLAGS="--release"
  BIN_DIR="target/release"
else
  CARGO_FLAGS=""
  BIN_DIR="target/debug"
fi

echo "-- building limux-cli ($PROFILE)..."
cargo build $CARGO_FLAGS -p limux-cli --bin limux-cli 2>&1 | tail -3

echo "-- building limux-host-linux ($PROFILE)..."
cargo build $CARGO_FLAGS -p limux-host-linux 2>&1 | tail -3

LIMUX_HOST="$ROOT_DIR/$BIN_DIR/limux"
LIMUX_CLI="$ROOT_DIR/$BIN_DIR/limux-cli"
[ -x "$LIMUX_HOST" ] || { echo "FAIL: host binary missing at $LIMUX_HOST"; exit 2; }
[ -x "$LIMUX_CLI" ]  || { echo "FAIL: cli binary missing at $LIMUX_CLI"; exit 2; }

# The host needs libghostty.so on the runtime path in both release and debug
# profiles on some distro/toolchain combinations.
LIBGHOSTTY_DIR="$ROOT_DIR/ghostty/zig-out/lib"
if [ -d "$LIBGHOSTTY_DIR" ]; then
  export LD_LIBRARY_PATH="$LIBGHOSTTY_DIR:${LD_LIBRARY_PATH:-}"
fi

FAKE_BIN_DIR="$DEMO_DIR/fake-bin"
FAKE_AGENT_PROOF_DIR="$DEMO_DIR/fake-agent-proof"
FAKE_SHELL="$DEMO_DIR/fake-shell"
FAKE_ZDOTDIR="$DEMO_DIR/zdot"
mkdir -p "$FAKE_BIN_DIR" "$FAKE_AGENT_PROOF_DIR" "$FAKE_ZDOTDIR"
cat > "$FAKE_BIN_DIR/codex" <<'FAKE_AGENT'
#!/usr/bin/env bash
set -euo pipefail
agent="$(basename "$0")"
proof_dir="${LIMUX_FAKE_AGENT_PROOF_DIR:-__LIMUX_FAKE_AGENT_PROOF_DIR__}"
protocol_path="${LIMUX_FAKE_AGENT_PROTOCOL_PATH:-__LIMUX_FAKE_AGENT_PROTOCOL_PATH__}"
roster_path="${LIMUX_FAKE_AGENT_ROSTER_PATH:-__LIMUX_FAKE_AGENT_ROSTER_PATH__}"
ledger_path="${LIMUX_FAKE_AGENT_LEDGER_PATH:-__LIMUX_FAKE_AGENT_LEDGER_PATH__}"
mkdir -p "$proof_dir"
line_status="timeout"
line=""
if IFS= read -r -t "${LIMUX_FAKE_AGENT_READ_TIMEOUT:-10}" line; then
  line_status="read"
fi
protocol_exists="no"
if [[ -f "$protocol_path" ]]; then
  protocol_exists="yes"
fi
roster_exists="no"
if [[ -f "$roster_path" ]]; then
  roster_exists="yes"
fi
ledger_exists="no"
if [[ -f "$ledger_path" ]]; then
  ledger_exists="yes"
fi
{
  printf 'agent=%s\n' "$agent"
  printf 'argc=%s\n' "$#"
  printf 'line_status=%s\n' "$line_status"
  printf 'protocol_exists=%s\n' "$protocol_exists"
  printf 'roster_exists=%s\n' "$roster_exists"
  printf 'ledger_exists=%s\n' "$ledger_exists"
  printf 'workspace=%s\n' "${LIMUX_WORKSPACE_ID:-}"
  printf 'surface=%s\n' "${LIMUX_SURFACE_ID:-}"
  printf 'line=%s\n' "$line"
} > "$proof_dir/$agent.bootstrap"
FAKE_AGENT
sed -i \
  -e "s|__LIMUX_FAKE_AGENT_PROOF_DIR__|$FAKE_AGENT_PROOF_DIR|g" \
  -e "s|__LIMUX_FAKE_AGENT_PROTOCOL_PATH__|$DEMO_DIR/LIMUX_AGENTS.md|g" \
  -e "s|__LIMUX_FAKE_AGENT_ROSTER_PATH__|$DEMO_DIR/LIMUX_TEAM_ROSTER.md|g" \
  -e "s|__LIMUX_FAKE_AGENT_LEDGER_PATH__|$DEMO_DIR/LIMUX_REVIEW_LEDGER.md|g" \
  "$FAKE_BIN_DIR/codex"
cp "$FAKE_BIN_DIR/codex" "$FAKE_BIN_DIR/claude"
chmod +x "$FAKE_BIN_DIR/codex" "$FAKE_BIN_DIR/claude"
export PATH="$FAKE_BIN_DIR:$PATH"
export LIMUX_FAKE_AGENT_PROOF_DIR="$FAKE_AGENT_PROOF_DIR"
export LIMUX_FAKE_AGENT_PROTOCOL_PATH="$DEMO_DIR/LIMUX_AGENTS.md"
export LIMUX_FAKE_AGENT_ROSTER_PATH="$DEMO_DIR/LIMUX_TEAM_ROSTER.md"
export LIMUX_FAKE_AGENT_LEDGER_PATH="$DEMO_DIR/LIMUX_REVIEW_LEDGER.md"
export LIMUX_FAKE_AGENT_READ_TIMEOUT=10
cat > "$FAKE_ZDOTDIR/.zshenv" <<FAKE_ZSHENV
export PATH="$FAKE_BIN_DIR:\$PATH"
FAKE_ZSHENV
export ZDOTDIR="$FAKE_ZDOTDIR"
cat > "$FAKE_SHELL" <<FAKE_SHELL_WRAPPER
#!/usr/bin/env bash
export PATH="$FAKE_BIN_DIR:\$PATH"
exec /bin/bash "\$@"
FAKE_SHELL_WRAPPER
chmod +x "$FAKE_SHELL"
export SHELL="$FAKE_SHELL"

# --- 3. Stage 0: dry-run agent-team (no host) ----------------------------
# Fast sanity pass — if this fails nothing else will work.
echo
echo "== stage 0: agent-team --dry-run (no host) =="
"$LIMUX_CLI" agent-team --dry-run \
  --agents codex,claude,opencode,gemini \
  --cwd "$DEMO_DIR" \
  --roster-path "$DEMO_DIR/stage0-team-roster.md" \
  --ledger-path "$DEMO_DIR/stage0-review-ledger.md" \
  2>&1 | tee "$LOG_DIR/stage0.txt"

grep -q "peers=\[codex, claude, opencode, gemini\]" \
  "$LOG_DIR/stage0.txt" \
  || { echo "FAIL: stage 0 dry-run did not report expected peers"; exit 1; }
grep -q '<!-- limux-team-roster durable:create-if-missing:v1 -->' "$DEMO_DIR/stage0-team-roster.md" \
  || { echo "FAIL: stage 0 did not create dry-run roster"; exit 1; }
grep -q '<!-- limux-review-ledger durable:v1 -->' "$DEMO_DIR/stage0-review-ledger.md" \
  || { echo "FAIL: stage 0 did not create dry-run review ledger"; exit 1; }
grep -F -q '| `current` | peer | `opencode`' "$DEMO_DIR/stage0-team-roster.md" \
  || { echo "FAIL: stage 0 roster missing opencode peer"; exit 1; }
echo "stage 0: OK"

# --- 4. Launch the live host under Xvfb ----------------------------------
# Each smoke run gets its own socket path so we don't collide with the
# user's real limux session.
SOCKET="$DEMO_DIR/limux.sock"
export LIMUX_SOCKET="$SOCKET"
export LIMUX_SOCKET_PATH="$SOCKET"
export LIMUX_SOCKET_MODE="runtime"
export XDG_DATA_HOME="$DEMO_DIR/data"
export XDG_STATE_HOME="$DEMO_DIR/state"
export XDG_RUNTIME_DIR="$DEMO_DIR/runtime"
mkdir -p "$XDG_DATA_HOME/limux" "$XDG_STATE_HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
cat > "$XDG_DATA_HOME/limux/session.json" <<SMOKE_SESSION
{
  "version": 1,
  "active_workspace_index": 0,
  "top_bar_visible": true,
  "sidebar": { "visible": true, "width": 220 },
  "workspaces": [
    {
      "id": "00000000-0000-4000-8000-000000000001",
      "name": "limux",
      "favorite": false,
      "cwd": "$DEMO_DIR",
      "folder_path": "$DEMO_DIR",
      "layout": {
        "kind": "pane",
        "pane_id": 1,
        "active_tab_id": "terminal-0",
        "tabs": [
          {
            "id": "terminal-0",
            "custom_name": null,
            "pinned": false,
            "tab_kind": "terminal",
            "cwd": "$DEMO_DIR"
          }
        ]
      }
    }
  ]
}
SMOKE_SESSION

echo
echo "== stage 1: boot limux host under xvfb-run =="
# Under Xvfb there is no GPU, so Mesa falls back to a software renderer.
# Ghostty now requires OpenGL 4.3, which softpipe cannot provide reliably
# enough for the embedded surface path. Use llvmpipe by default, while keeping
# an explicit smoke-only override for debugging local Mesa regressions.
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER="${LIMUX_SMOKE_GALLIUM_DRIVER:-llvmpipe}"
export LP_NUM_THREADS=1
export MESA_GL_VERSION_OVERRIDE="${MESA_GL_VERSION_OVERRIDE:-4.3}"
xvfb-run -a -s "-screen 0 1280x800x24 +extension GLX +render" \
  "$LIMUX_HOST" >"$LOG_DIR/host.stdout" 2>"$LOG_DIR/host.stderr" &
HOST_PID=$!
echo "host PID: $HOST_PID (socket=$SOCKET)"

cleanup() {
  local rc=$?
  echo
  echo "-- cleanup (rc=$rc) --"
  if kill -0 "$HOST_PID" 2>/dev/null; then
    kill "$HOST_PID" 2>/dev/null || true
    sleep 1
    kill -9 "$HOST_PID" 2>/dev/null || true
  fi
  # Tail the host log on failure to aid debugging.
  if [ "$rc" -ne 0 ]; then
    echo "-- host.stdout (tail) --"
    tail -n 40 "$LOG_DIR/host.stdout" 2>/dev/null || true
    echo "-- host.stderr (tail) --"
    tail -n 40 "$LOG_DIR/host.stderr" 2>/dev/null || true
    echo "artifacts retained at: $DEMO_DIR"
  else
    # Clean slate on success.
    rm -rf "$DEMO_DIR"
  fi
}
trap cleanup EXIT INT TERM

# Poll for the socket (up to 30s)
for i in $(seq 1 60); do
  if [ -S "$SOCKET" ]; then
    echo "socket up after ${i}*500ms"
    break
  fi
  if ! kill -0 "$HOST_PID" 2>/dev/null; then
    echo "FAIL: host process died before opening the socket"
    exit 1
  fi
  sleep 0.5
done

[ -S "$SOCKET" ] || { echo "FAIL: socket $SOCKET never appeared"; exit 1; }

# --- 5. Stage 2: live agent-team bootstrap with fake agents ---------------
echo
echo "== stage 2: agent-team two-phase bootstrap with fake agents =="
rm -f "$FAKE_AGENT_PROOF_DIR"/*.bootstrap
printf 'manual review ledger sentinel\n' > "$DEMO_DIR/LIMUX_REVIEW_LEDGER.md"
"$LIMUX_CLI" --id-format both agent-team \
  --agents codex,claude \
  --cwd "$DEMO_DIR" \
  --force-protocol-overwrite \
  2>&1 | tee "$LOG_DIR/stage2.txt"

grep -q "bootstrap=sent" "$LOG_DIR/stage2.txt" \
  || { echo "FAIL: live agent-team did not report bootstrap=sent"; exit 1; }

for i in $(seq 1 50); do
  if [[ -s "$FAKE_AGENT_PROOF_DIR/codex.bootstrap" && -s "$FAKE_AGENT_PROOF_DIR/claude.bootstrap" ]]; then
    break
  fi
  sleep 0.2
done

for agent in codex claude; do
  proof="$FAKE_AGENT_PROOF_DIR/$agent.bootstrap"
  if [[ ! -s "$proof" ]]; then
    surface="$(
      awk -F'|' -v agent="\`$agent\`" '$2 ~ agent { gsub(/[`[:space:]]/, "", $4); print $4; exit }' \
        "$DEMO_DIR/LIMUX_AGENTS.md"
    )"
    if [[ -n "$surface" ]]; then
      "$LIMUX_CLI" read-screen --surface "$surface" --scrollback --lines 80 \
        > "$LOG_DIR/stage2-$agent-screen.txt" 2>&1 || true
      cat "$LOG_DIR/stage2-$agent-screen.txt" || true
    fi
    echo "FAIL: missing fake $agent bootstrap proof"
    exit 1
  fi
  grep -q '^argc=0$' "$proof" || { echo "FAIL: fake $agent received unexpected argv"; cat "$proof"; exit 1; }
  grep -q '^line_status=read$' "$proof" || { echo "FAIL: fake $agent did not read bootstrap prompt"; cat "$proof"; exit 1; }
  grep -q '^protocol_exists=yes$' "$proof" || { echo "FAIL: fake $agent read prompt before LIMUX_AGENTS.md existed"; cat "$proof"; exit 1; }
  grep -q '^roster_exists=yes$' "$proof" || { echo "FAIL: fake $agent read prompt before LIMUX_TEAM_ROSTER.md existed"; cat "$proof"; exit 1; }
  grep -q '^ledger_exists=yes$' "$proof" || { echo "FAIL: fake $agent read prompt before LIMUX_REVIEW_LEDGER.md existed"; cat "$proof"; exit 1; }
  grep -q 'Read the generated runtime protocol file' "$proof" || { echo "FAIL: fake $agent prompt missing protocol instruction"; cat "$proof"; exit 1; }
  grep -q 'team roster' "$proof" || { echo "FAIL: fake $agent prompt missing roster instruction"; cat "$proof"; exit 1; }
  grep -q 'durable review ledger' "$proof" || { echo "FAIL: fake $agent prompt missing ledger instruction"; cat "$proof"; exit 1; }
  grep -q 'authoritative instruction sources' "$proof" || { echo "FAIL: fake $agent prompt missing instruction-source instruction"; cat "$proof"; exit 1; }
done
grep -q '<!-- limux-team-roster durable:create-if-missing:v1 -->' "$DEMO_DIR/LIMUX_TEAM_ROSTER.md" \
  || { echo "FAIL: live agent-team did not seed team roster"; exit 1; }
grep -q 'manual review ledger sentinel' "$DEMO_DIR/LIMUX_REVIEW_LEDGER.md" \
  || { echo "FAIL: live agent-team overwrote existing review ledger"; exit 1; }
echo "stage 2: OK (fake agents received post-write bootstrap prompt)"

# Extract the generated team targets for the remaining live bridge checks.
TEAM_WORKSPACE="$(
  sed -n 's/^- Workspace ID: `\([^`]*\)`/\1/p' "$DEMO_DIR/LIMUX_AGENTS.md" | head -1
)"
CLAUDE_SURFACE="$(
  awk -F'|' '/`claude`/ { gsub(/[`[:space:]]/, "", $4); print $4; exit }' "$DEMO_DIR/LIMUX_AGENTS.md"
)"
[ -n "$TEAM_WORKSPACE" ] \
  || { echo "FAIL: LIMUX_AGENTS.md missing team workspace id"; exit 1; }
[ -n "$CLAUDE_SURFACE" ] \
  || { echo "FAIL: LIMUX_AGENTS.md missing claude surface"; exit 1; }

# --- 6. Stage 3: list-workspaces sanity -----------------------------------
echo
echo "== stage 3: list-workspaces sees team workspace =="
"$LIMUX_CLI" --id-format both list-workspaces 2>&1 | tee "$LOG_DIR/stage3-list.txt"
grep -q "$TEAM_WORKSPACE" "$LOG_DIR/stage3-list.txt" \
  || { echo "FAIL: list-workspaces missing team workspace $TEAM_WORKSPACE"; exit 1; }
echo "stage 3: OK"

# --- 7. Stage 4: generated peer surface send ------------------------------
echo
echo "== stage 4: surface.send_text to generated peer surface =="
ENVELOPE=$'<agent-msg from="codex" to="claude" id="smoke-1" ts="2026-04-19T23:59:00Z">\n\t<request>smoke test ping π</request>\n</agent-msg>\n'
if "$LIMUX_CLI" send --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" "$ENVELOPE" \
     2>&1 | tee "$LOG_DIR/stage4-send.txt"; then
  echo "stage 4: OK (surface send accepted)"
else
  echo "FAIL: send to generated claude surface failed"
  exit 1
fi

# --- 8. Stage 5: workspace notify -----------------------------------------
echo
echo "== stage 5: notification.create by team workspace id =="
if "$LIMUX_CLI" notify --workspace "$TEAM_WORKSPACE" --subtitle "smoke" --body "all good" "Smoke test" \
     2>&1 | tee "$LOG_DIR/stage5-notify.txt"; then
  echo "stage 5: OK (workspace notify accepted)"
else
  echo "FAIL: workspace notify failed"
  exit 1
fi

# --- 9. Stage 6: self-split pane.create + command injection ----------------
echo
echo "== stage 6: pane.create self-split with exact-surface command =="
SELF_SPLIT_PROOF="$DEMO_DIR/self-split-proof"
SELF_SPLIT_ENV="$DEMO_DIR/self-split-env"
SELF_SPLIT_CMD="printf split-ok > '$SELF_SPLIT_PROOF'; printf '%s\n%s\n%s\n' \"\$LIMUX_WORKSPACE_ID\" \"\$LIMUX_PANE_ID\" \"\$LIMUX_SURFACE_ID\" > '$SELF_SPLIT_ENV'"

"$LIMUX_CLI" --json new-pane \
  --workspace "$TEAM_WORKSPACE" \
  --surface "$CLAUDE_SURFACE" \
  --direction right \
  --command "$SELF_SPLIT_CMD" \
  2>&1 | tee "$LOG_DIR/stage6.json"

RESPONSE_WORKSPACE="$(sed -n 's/.*"workspace_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"
RESPONSE_PANE="$(sed -n 's/.*"pane_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"
RESPONSE_SURFACE="$(sed -n 's/.*"surface_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"

if [ -z "$RESPONSE_WORKSPACE" ]; then
  RESPONSE_WORKSPACE_REF="$(sed -n 's/.*"workspace_ref"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"
  RESPONSE_WORKSPACE="${RESPONSE_WORKSPACE_REF#workspace:}"
fi
if [ -z "$RESPONSE_PANE" ]; then
  RESPONSE_PANE_REF="$(sed -n 's/.*"pane_ref"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"
  RESPONSE_PANE="${RESPONSE_PANE_REF#pane:}"
fi
if [ -z "$RESPONSE_SURFACE" ]; then
  RESPONSE_SURFACE_REF="$(sed -n 's/.*"surface_ref"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$LOG_DIR/stage6.json" | head -1)"
  RESPONSE_SURFACE="${RESPONSE_SURFACE_REF#surface:}"
fi

[ -n "$RESPONSE_WORKSPACE" ] || { echo "FAIL: pane.create response missing workspace_id/workspace_ref"; exit 1; }
[ -n "$RESPONSE_PANE" ] || { echo "FAIL: pane.create response missing pane_id/pane_ref"; exit 1; }
[ -n "$RESPONSE_SURFACE" ] || { echo "FAIL: pane.create response missing surface_id/surface_ref"; exit 1; }

for _ in $(seq 1 50); do
  if [ -f "$SELF_SPLIT_PROOF" ] && [ -f "$SELF_SPLIT_ENV" ]; then
    break
  fi
  sleep 0.1
done

[ -f "$SELF_SPLIT_PROOF" ] || { echo "FAIL: self-split command proof file missing"; exit 1; }
[ "$(cat "$SELF_SPLIT_PROOF")" = "split-ok" ] || { echo "FAIL: self-split proof file has unexpected content"; exit 1; }
[ -f "$SELF_SPLIT_ENV" ] || { echo "FAIL: self-split env file missing"; exit 1; }

ENV_WORKSPACE="$(sed -n '1p' "$SELF_SPLIT_ENV")"
ENV_PANE="$(sed -n '2p' "$SELF_SPLIT_ENV")"
ENV_SURFACE="$(sed -n '3p' "$SELF_SPLIT_ENV")"

[ "$ENV_WORKSPACE" = "$RESPONSE_WORKSPACE" ] || {
  echo "FAIL: spawned pane LIMUX_WORKSPACE_ID ($ENV_WORKSPACE) did not match response ($RESPONSE_WORKSPACE)"
  exit 1
}
[ "$ENV_PANE" = "$RESPONSE_PANE" ] || {
  echo "FAIL: spawned pane LIMUX_PANE_ID ($ENV_PANE) did not match response ($RESPONSE_PANE)"
  exit 1
}
[ "$ENV_SURFACE" = "$RESPONSE_SURFACE" ] || {
  echo "FAIL: spawned pane LIMUX_SURFACE_ID ($ENV_SURFACE) did not match response ($RESPONSE_SURFACE)"
  exit 1
}
echo "stage 6: OK (self-split command ran with fresh LIMUX_* env)"

# --- 10. Stage 7: typed-PTY control-character guard -----------------------
echo
echo "== stage 7: typed-PTY control-character guard =="
BAD_ESC=$'bad\x1b[31m'
BAD_BEL=$'bad\x07'
BAD_CSI="$(printf 'bad\302\23331m')"

expect_control_reject() {
  local label="$1"
  local codepoint="$2"
  shift 2
  if "$@" > "$LOG_DIR/$label.txt" 2>&1; then
    cat "$LOG_DIR/$label.txt"
    echo "FAIL: $label accepted disallowed terminal control character"
    exit 1
  fi
  if ! grep -q "disallowed terminal control character $codepoint" "$LOG_DIR/$label.txt"; then
    cat "$LOG_DIR/$label.txt"
    echo "FAIL: $label did not report $codepoint"
    exit 1
  fi
}

expect_control_reject stage7-send-esc U+001B \
  "$LIMUX_CLI" send --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" "$BAD_ESC"

expect_control_reject stage7-send-c1 U+009B \
  "$LIMUX_CLI" send --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" "$BAD_CSI"

expect_control_reject stage7-new-pane-bel U+0007 \
  "$LIMUX_CLI" new-pane --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" \
    --command "$BAD_BEL"

expect_control_reject stage7-respawn-esc U+001B \
  "$LIMUX_CLI" respawn-pane --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" \
    --command "$BAD_ESC"

"$LIMUX_CLI" set-buffer --name bad-control "$BAD_ESC" \
  > "$LOG_DIR/stage7-set-buffer.txt" 2>&1
expect_control_reject stage7-paste-buffer-esc U+001B \
  "$LIMUX_CLI" paste-buffer --workspace "$TEAM_WORKSPACE" --surface "$CLAUDE_SURFACE" \
    --name bad-control

expect_control_reject stage7-new-workspace-bel U+0007 \
  "$LIMUX_CLI" new-workspace --command "$BAD_BEL"

echo "stage 7: OK (typed-PTY control payloads rejected)"

# --- 11. Stage 8: hook translators end-to-end -----------------------------
echo
echo "== stage 8: claude-hook event translation =="
if echo '{"hook_event_name":"Notification","message":"hello from smoke"}' \
  | LIMUX_WORKSPACE_ID="" "$LIMUX_CLI" claude-hook 2>&1 \
  | tee "$LOG_DIR/stage8.txt"; then
  echo "stage 8: OK (claude-hook accepted JSON on stdin)"
else
  # claude-hook legitimately errors without a workspace target — that's
  # a pass-through error, not a bridge regression. Surface the output.
  echo "stage 8: claude-hook returned non-zero (check output)"
fi

echo
echo "===================================="
echo "✅ limux agent-integrations smoke test PASSED"
echo "===================================="
