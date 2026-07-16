#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/renderer-backend-preview/renderer-backend-preview.sh"
TEST_ROOT="$(mktemp -d -t limux-renderer-runner-test-XXXXXX)"
FAKE_HOST="$TEST_ROOT/fake-host"
FAKE_CLI="$TEST_ROOT/fake-cli"
BACKEND_LOG="$TEST_ROOT/backends.log"
DESCENDANT_LOG="$TEST_ROOT/descendants.log"
SESSION_TEMPLATE="$TEST_ROOT/session.json"
ARTIFACTS="$TEST_ROOT/artifacts"

printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': "${FAKE_BACKEND_LOG:?}"' \
    ': "${FAKE_DESCENDANT_LOG:?}"' \
    'printf "%s\n" "${GALLIUM_DRIVER:-automatic}" >> "$FAKE_BACKEND_LOG"' \
    'timeout 300s tail -f /dev/null >/dev/null 2>&1 &' \
    'printf "%s\n" "$!" >> "$FAKE_DESCENDANT_LOG"' \
    'trap "exit 0" INT TERM' \
    'while :; do timeout 1s tail -f /dev/null >/dev/null 2>&1 || true; done' \
    > "$FAKE_HOST"

printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    ': "${FAKE_BACKEND_LOG:?}"' \
    'args=" $* "' \
    'if [[ "$args" == *" list-workspaces "* ]]; then' \
    '  printf "%s\n" '\''{"workspaces":[{"workspace_ref":"workspace:inactive","focused":false,"selected":false},{"workspace_ref":"workspace:test","focused":true,"selected":true}]}'\''' \
    '  exit 0' \
    'fi' \
    'if [[ "$args" == *" surface-health "* ]]; then' \
    '  [[ "$args" == *" --workspace workspace:test "* ]] || exit 3' \
    '  backend="$(tail -n 1 "$FAKE_BACKEND_LOG")"' \
    '  if [[ "$backend" == "definitely-missing" ]]; then' \
    '    printf "%s\n" '\''{"renderer_diagnostics":{"status":"captured","selected_renderer":"GskCairoRenderer","is_software_fallback":true,"requested_policy":{"gallium_driver":"definitely-missing","lp_num_threads":null},"gpu_device_usage":{"dxg_open":false}},"surfaces":[{"healthy":false,"realized":false,"width_px":0,"height_px":0}]}'\''' \
    '  elif [[ "$backend" == "d3d12" && "${FAKE_D3D12_WRONG:-0}" == "1" ]]; then' \
    '    printf "%s\n" '\''{"renderer_diagnostics":{"status":"captured","selected_renderer":"GskGLRenderer","is_software_fallback":true,"requested_policy":{"gallium_driver":"d3d12","lp_num_threads":null},"gpu_device_usage":{"dxg_open":false}},"surfaces":[{"healthy":true,"realized":true,"width_px":1200,"height_px":800}]}'\''' \
    '  elif [[ "$backend" == "d3d12" ]]; then' \
    '    printf "%s\n" '\''{"renderer_diagnostics":{"status":"captured","selected_renderer":"GskGLRenderer","is_software_fallback":false,"requested_policy":{"gallium_driver":"d3d12","lp_num_threads":null},"gpu_device_usage":{"dxg_open":true}},"surfaces":[{"healthy":true,"realized":true,"width_px":1200,"height_px":800}]}'\''' \
    '  elif [[ "$backend" == "llvmpipe" ]]; then' \
    '    printf "%s\n" '\''{"renderer_diagnostics":{"status":"captured","selected_renderer":"GskGLRenderer","is_software_fallback":true,"requested_policy":{"gallium_driver":"llvmpipe","lp_num_threads":"2"},"gpu_device_usage":{"dxg_open":false}},"surfaces":[{"healthy":true,"realized":true,"width_px":1200,"height_px":800}]}'\''' \
    '  else' \
    '    printf "%s\n" '\''{"renderer_diagnostics":{"status":"captured","selected_renderer":"GskGLRenderer","is_software_fallback":false,"requested_policy":{"gallium_driver":null,"lp_num_threads":null},"gpu_device_usage":{"dxg_open":false}},"surfaces":[{"healthy":true,"realized":true,"width_px":1200,"height_px":800}]}'\''' \
    '  fi' \
    '  exit 0' \
    'fi' \
    'exit 2' \
    > "$FAKE_CLI"

chmod +x "$FAKE_HOST" "$FAKE_CLI"
printf '%s\n' '{}' > "$SESSION_TEMPLATE"

FAKE_BACKEND_LOG="$BACKEND_LOG" \
FAKE_DESCENDANT_LOG="$DESCENDANT_LOG" \
    "$RUNNER" \
    --host "$FAKE_HOST" \
    --cli "$FAKE_CLI" \
    --session-template "$SESSION_TEMPLATE" \
    --artifacts "$ARTIFACTS" \
    --start invalid-test \
    --polls 20 \
    --poll-interval-ms 25

jq -e '.selected_backend == "wsl-d3d12-gl"' "$ARTIFACTS/result.json" >/dev/null
jq -e '.attempted_backends == ["invalid-test", "wsl-d3d12-gl"]' \
    "$ARTIFACTS/result.json" >/dev/null
[[ "$(wc -l < "$BACKEND_LOG")" -eq 2 ]]
while IFS= read -r pid_file; do
    pid="$(< "$pid_file")"
    ! kill -0 "$pid" 2>/dev/null
done < <(find "$ARTIFACTS" -name host.pid -type f -print)

WRONG_ARTIFACTS="$TEST_ROOT/wrong-renderer-artifacts"
FAKE_BACKEND_LOG="$BACKEND_LOG" \
FAKE_DESCENDANT_LOG="$DESCENDANT_LOG" \
FAKE_D3D12_WRONG=1 \
    "$RUNNER" \
    --host "$FAKE_HOST" \
    --cli "$FAKE_CLI" \
    --session-template "$SESSION_TEMPLATE" \
    --artifacts "$WRONG_ARTIFACTS" \
    --start wsl-d3d12-gl \
    --polls 20 \
    --poll-interval-ms 25
jq -e '.selected_backend == "desktop-gl"' "$WRONG_ARTIFACTS/result.json" >/dev/null
jq -e '.attempted_backends == ["wsl-d3d12-gl", "desktop-gl"]' \
    "$WRONG_ARTIFACTS/result.json" >/dev/null

FAIL_BIN="$TEST_ROOT/fail-bin"
FAIL_ARTIFACTS="$TEST_ROOT/setup-failure-artifacts"
mkdir "$FAIL_BIN"
printf '%s\n' '#!/usr/bin/env bash' 'exit 73' > "$FAIL_BIN/cp"
chmod +x "$FAIL_BIN/cp"
backend_count_before="$(wc -l < "$BACKEND_LOG")"
set +e
PATH="$FAIL_BIN:$PATH" \
FAKE_BACKEND_LOG="$BACKEND_LOG" \
FAKE_DESCENDANT_LOG="$DESCENDANT_LOG" \
    "$RUNNER" \
    --host "$FAKE_HOST" \
    --cli "$FAKE_CLI" \
    --session-template "$SESSION_TEMPLATE" \
    --artifacts "$FAIL_ARTIFACTS" \
    --start wsl-d3d12-gl \
    > "$TEST_ROOT/setup-failure.stdout" \
    2> "$TEST_ROOT/setup-failure.stderr"
setup_rc=$?
set -e
[[ "$setup_rc" -eq 2 ]]
[[ "$(wc -l < "$BACKEND_LOG")" -eq "$backend_count_before" ]]
jq -e '.status == "error" and .attempted_backends == []' \
    "$FAIL_ARTIFACTS/result.json" >/dev/null

UNKNOWN_ARTIFACTS="$TEST_ROOT/unknown-artifacts"
set +e
FAKE_BACKEND_LOG="$BACKEND_LOG" \
    "$RUNNER" \
    --host "$FAKE_HOST" \
    --cli "$FAKE_CLI" \
    --session-template "$SESSION_TEMPLATE" \
    --artifacts "$UNKNOWN_ARTIFACTS" \
    --start unknown-backend \
    > "$TEST_ROOT/unknown.stdout" \
    2> "$TEST_ROOT/unknown.stderr"
unknown_rc=$?
set -e
[[ "$unknown_rc" -eq 2 ]]
[[ ! -e "$UNKNOWN_ARTIFACTS" ]]

while IFS= read -r descendant_pid; do
    ! kill -0 "$descendant_pid" 2>/dev/null
done < "$DESCENDANT_LOG"
while IFS= read -r pid_file; do
    pid="$(< "$pid_file")"
    ! kill -0 "$pid" 2>/dev/null
done < <(find "$TEST_ROOT" -name '*.pid' -type f -print)

printf 'PASS artifacts=%s\n' "$TEST_ROOT"
