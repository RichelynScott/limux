#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: renderer-backend-preview.sh \
  --host <limux-host> \
  --cli <limux-cli> \
  --session-template <session.json> \
  --artifacts <new-directory> \
  [--start wsl-d3d12-gl|desktop-gl|software-gl|invalid-test] \
  [--polls <count>] [--poll-interval-ms <milliseconds>]

Runs a bounded, isolated process-level renderer fallback check. Every candidate
uses a unique preview socket, session directory, and XDG tree. The successful
candidate is also stopped after verification; this script never installs or
promotes a runtime.
USAGE
}

HOST=""
CLI=""
SESSION_TEMPLATE=""
ARTIFACTS=""
START_BACKEND="wsl-d3d12-gl"
POLLS=80
POLL_INTERVAL_MS=250
MAX_CAPTURE_BYTES=262144
BACKEND_TIMEOUT_SECONDS=30
CLI_TIMEOUT_SECONDS=2

while [[ $# -gt 0 ]]; do
    case "$1" in
        --host)
            HOST="${2:-}"
            shift 2
            ;;
        --cli)
            CLI="${2:-}"
            shift 2
            ;;
        --session-template)
            SESSION_TEMPLATE="${2:-}"
            shift 2
            ;;
        --artifacts)
            ARTIFACTS="${2:-}"
            shift 2
            ;;
        --start)
            START_BACKEND="${2:-}"
            shift 2
            ;;
        --polls)
            POLLS="${2:-}"
            shift 2
            ;;
        --poll-interval-ms)
            POLL_INTERVAL_MS="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'renderer-preview: unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

for required in HOST CLI SESSION_TEMPLATE ARTIFACTS; do
    if [[ -z "${!required}" ]]; then
        printf 'renderer-preview: --%s is required\n' \
            "$(printf '%s' "$required" | tr '[:upper:]_' '[:lower:]-')" >&2
        exit 2
    fi
done

[[ -x "$HOST" ]] || {
    printf 'renderer-preview: host is not executable: %s\n' "$HOST" >&2
    exit 2
}
[[ -x "$CLI" ]] || {
    printf 'renderer-preview: cli is not executable: %s\n' "$CLI" >&2
    exit 2
}
[[ -f "$SESSION_TEMPLATE" ]] || {
    printf 'renderer-preview: session template is not a file: %s\n' "$SESSION_TEMPLATE" >&2
    exit 2
}
[[ "$POLLS" =~ ^[1-9][0-9]*$ ]] || {
    printf 'renderer-preview: --polls must be a positive integer\n' >&2
    exit 2
}
[[ "$POLL_INTERVAL_MS" =~ ^[1-9][0-9]*$ ]] || {
    printf 'renderer-preview: --poll-interval-ms must be a positive integer\n' >&2
    exit 2
}
((POLLS <= 240)) || {
    printf 'renderer-preview: --polls must not exceed 240\n' >&2
    exit 2
}
((POLL_INTERVAL_MS <= 1000)) || {
    printf 'renderer-preview: --poll-interval-ms must not exceed 1000\n' >&2
    exit 2
}
command -v jq >/dev/null || {
    printf 'renderer-preview: jq is required\n' >&2
    exit 2
}
command -v timeout >/dev/null || {
    printf 'renderer-preview: timeout is required\n' >&2
    exit 2
}
for required_command in setsid ps pgrep pkill mkfifo; do
    command -v "$required_command" >/dev/null || {
        printf 'renderer-preview: %s is required\n' "$required_command" >&2
        exit 2
    }
done
if [[ -e "$ARTIFACTS" ]]; then
    printf 'renderer-preview: artifacts path already exists: %s\n' "$ARTIFACTS" >&2
    exit 2
fi
case "$START_BACKEND" in
    invalid-test|wsl-d3d12-gl|desktop-gl|software-gl) ;;
    *)
        printf 'renderer-preview: unknown start backend: %s\n' "$START_BACKEND" >&2
        exit 2
        ;;
esac

mkdir "$ARTIFACTS"
ATTEMPTS_FILE="$ARTIFACTS/attempted-backends.txt"
: > "$ATTEMPTS_FILE"

CURRENT_PID=""
CURRENT_SID=""
CURRENT_CAPTURE_PIDS=()
RUNNER_SID="$(ps -o sid= -p "$$" | tr -d '[:space:]')"
BACKEND_ENV=()
NEXT_BACKEND=""

wait_tick() {
    timeout "${POLL_INTERVAL_MS}ms" tail -f /dev/null >/dev/null 2>&1 || true
}

stop_tick() {
    timeout 100ms tail -f /dev/null >/dev/null 2>&1 || true
}

capture_bounded() {
    head -c "$MAX_CAPTURE_BYTES"
    cat >/dev/null
}

stop_capture_processes() {
    local pid
    for pid in "${CURRENT_CAPTURE_PIDS[@]}"; do
        for _ in {1..8}; do
            kill -0 "$pid" 2>/dev/null || break
            stop_tick
        done
        if kill -0 "$pid" 2>/dev/null; then
            kill -TERM "$pid" 2>/dev/null || true
        fi
        wait "$pid" 2>/dev/null || true
    done
    CURRENT_CAPTURE_PIDS=()
}

stop_current_host() {
    local pid="${CURRENT_PID:-}"
    local sid="${CURRENT_SID:-}"

    if [[ -n "$sid" && "$sid" != "$RUNNER_SID" ]]; then
        if pgrep --session "$sid" >/dev/null 2>&1; then
            pkill --signal INT --session "$sid" 2>/dev/null || true
            for _ in {1..8}; do
                pgrep --session "$sid" >/dev/null 2>&1 || break
                stop_tick
            done
        fi
        if pgrep --session "$sid" >/dev/null 2>&1; then
            pkill --signal TERM --session "$sid" 2>/dev/null || true
            for _ in {1..8}; do
                pgrep --session "$sid" >/dev/null 2>&1 || break
                stop_tick
            done
        fi
        if pgrep --session "$sid" >/dev/null 2>&1; then
            pkill --signal KILL --session "$sid" 2>/dev/null || true
        fi
    elif [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        kill -TERM "$pid" 2>/dev/null || true
    fi

    if [[ -n "$pid" ]]; then
        wait "$pid" 2>/dev/null || true
    fi
    CURRENT_PID=""
    CURRENT_SID=""
    stop_capture_processes
}

trap stop_current_host EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

configure_backend() {
    local backend="$1"
    BACKEND_ENV=()
    NEXT_BACKEND=""
    case "$backend" in
        invalid-test)
            BACKEND_ENV=("GSK_RENDERER=gl" "GALLIUM_DRIVER=definitely-missing")
            NEXT_BACKEND="wsl-d3d12-gl"
            ;;
        wsl-d3d12-gl)
            BACKEND_ENV=("GSK_RENDERER=gl" "GALLIUM_DRIVER=d3d12")
            NEXT_BACKEND="desktop-gl"
            ;;
        desktop-gl)
            BACKEND_ENV=("GSK_RENDERER=gl")
            NEXT_BACKEND="software-gl"
            ;;
        software-gl)
            BACKEND_ENV=(
                "GSK_RENDERER=gl"
                "LIBGL_ALWAYS_SOFTWARE=1"
                "GALLIUM_DRIVER=llvmpipe"
                "LP_NUM_THREADS=2"
            )
            ;;
        *)
            printf 'renderer-preview: unknown backend: %s\n' "$backend" >&2
            return 2
            ;;
    esac
}

backend_matches_diagnostics() {
    local backend="$1"
    local health_path="$2"
    jq -e --arg backend "$backend" '
        .renderer_diagnostics as $diagnostics |
        if $backend == "invalid-test" then
            false
        elif $backend == "wsl-d3d12-gl" then
            $diagnostics.status == "captured" and
            $diagnostics.selected_renderer == "GskGLRenderer" and
            $diagnostics.is_software_fallback == false and
            $diagnostics.requested_policy.gallium_driver == "d3d12" and
            $diagnostics.gpu_device_usage.dxg_open == true
        elif $backend == "desktop-gl" then
            $diagnostics.status == "captured" and
            ($diagnostics.selected_renderer == "GskGLRenderer" or
             $diagnostics.selected_renderer == "GskNglRenderer") and
            $diagnostics.is_software_fallback == false
        elif $backend == "software-gl" then
            $diagnostics.status == "captured" and
            ($diagnostics.selected_renderer == "GskGLRenderer" or
             $diagnostics.selected_renderer == "GskNglRenderer") and
            $diagnostics.is_software_fallback == true and
            $diagnostics.requested_policy.gallium_driver == "llvmpipe" and
            $diagnostics.requested_policy.lp_num_threads == "2" and
            $diagnostics.gpu_device_usage.dxg_open == false and
            (($diagnostics.fallback_indicators // []) |
                any(. == "thread:llvmpipe" or . == "renderer:llvmpipe"))
        else
            false
        end
    ' "$health_path" >/dev/null
}

attempt_backend() {
    local backend="$1"
    local attempt_dir="$ARTIFACTS/$backend"
    local socket="$attempt_dir/limux.sock"
    local workspace_ref=""
    local accepted=false
    local unhealthy_samples=0
    local started_at=$SECONDS
    local stdout_pipe="$attempt_dir/host.stdout.pipe"
    local stderr_pipe="$attempt_dir/host.stderr.pipe"

    configure_backend "$backend" || return $?
    mkdir -p \
        "$attempt_dir/session" \
        "$attempt_dir/config" \
        "$attempt_dir/state" \
        "$attempt_dir/data" \
        "$attempt_dir/cache" \
        "$attempt_dir/runtime" || return 2
    chmod 700 "$attempt_dir/runtime" || return 2
    cp "$SESSION_TEMPLATE" "$attempt_dir/session/session.json" || return 2
    printf '%s\n' "$backend" >> "$ATTEMPTS_FILE" || return 2
    mkfifo "$stdout_pipe" "$stderr_pipe" || return 2

    capture_bounded < "$stdout_pipe" > "$attempt_dir/host.stdout" &
    CURRENT_CAPTURE_PIDS+=("$!")
    printf '%s\n' "$!" > "$attempt_dir/host.stdout-capture.pid" || {
        stop_capture_processes
        return 2
    }
    capture_bounded < "$stderr_pipe" > "$attempt_dir/host.stderr" &
    CURRENT_CAPTURE_PIDS+=("$!")
    printf '%s\n' "$!" > "$attempt_dir/host.stderr-capture.pid" || {
        stop_capture_processes
        return 2
    }

    setsid env \
        -u LIMUX_WORKSPACE_ID \
        -u LIMUX_SURFACE_ID \
        -u LIMUX_PANE_ID \
        -u LIMUX_TAB_ID \
        -u LIMUX_SOCKET_PATH \
        -u GSK_RENDERER \
        -u GDK_DEBUG \
        -u GDK_DISABLE \
        -u GALLIUM_DRIVER \
        -u LIBGL_ALWAYS_SOFTWARE \
        -u LP_NUM_THREADS \
        -u MESA_GL_VERSION_OVERRIDE \
        -u MESA_LOADER_DRIVER_OVERRIDE \
        -u MESA_D3D12_DEFAULT_ADAPTER_NAME \
        "${BACKEND_ENV[@]}" \
        LIMUX_CHANNEL="preview:renderer-$backend" \
        LIMUX_SOCKET="$socket" \
        LIMUX_SESSION_DIR="$attempt_dir/session" \
        LIMUX_HOST_LOG=off \
        XDG_CONFIG_HOME="$attempt_dir/config" \
        XDG_STATE_HOME="$attempt_dir/state" \
        XDG_DATA_HOME="$attempt_dir/data" \
        XDG_CACHE_HOME="$attempt_dir/cache" \
        XDG_RUNTIME_DIR="$attempt_dir/runtime" \
        "$HOST" \
        > "$stdout_pipe" \
        2> "$stderr_pipe" &
    CURRENT_PID=$!
    printf '%s\n' "$CURRENT_PID" > "$attempt_dir/host.pid" || {
        stop_current_host
        return 2
    }

    for _ in {1..10}; do
        CURRENT_SID="$(ps -o sid= -p "$CURRENT_PID" 2>/dev/null | tr -d '[:space:]')"
        if [[ -n "$CURRENT_SID" && "$CURRENT_SID" != "$RUNNER_SID" ]]; then
            break
        fi
        stop_tick
    done
    if [[ -z "$CURRENT_SID" || "$CURRENT_SID" == "$RUNNER_SID" ]]; then
        stop_current_host
        return 2
    fi
    printf '%s\n' "$CURRENT_SID" > "$attempt_dir/host.sid" || {
        stop_current_host
        return 2
    }

    for ((poll = 1; poll <= POLLS; poll++)); do
        if ((SECONDS - started_at >= BACKEND_TIMEOUT_SECONDS)); then
            break
        fi
        if ! kill -0 "$CURRENT_PID" 2>/dev/null; then
            break
        fi
        if timeout --signal=TERM --kill-after=0.5s "${CLI_TIMEOUT_SECONDS}s" \
            env LIMUX_SOCKET="$socket" "$CLI" --json list-workspaces \
            > "$attempt_dir/list-workspaces.json" \
            2> "$attempt_dir/list-workspaces.stderr"; then
            workspace_ref="$(jq -r \
                '([(.workspaces // [])[] |
                    select(.focused == true or .selected == true)][0] //
                  (.workspaces // [])[0] // {}) |
                 (.workspace_ref // .ref // empty)' \
                "$attempt_dir/list-workspaces.json")"
        fi
        if [[ -n "$workspace_ref" ]] && \
            timeout --signal=TERM --kill-after=0.5s "${CLI_TIMEOUT_SECONDS}s" \
                env LIMUX_SOCKET="$socket" "$CLI" --json surface-health \
                --workspace "$workspace_ref" \
                > "$attempt_dir/surface-health.json" \
                2> "$attempt_dir/surface-health.stderr" && \
            jq -e '
                .renderer_diagnostics.status == "captured" and
                (.surfaces | type == "array" and length > 0) and
                all(.surfaces[];
                    .healthy == true and
                    .realized == true and
                    (.width_px // 0) > 0 and
                    (.height_px // 0) > 0)
            ' "$attempt_dir/surface-health.json" >/dev/null && \
            backend_matches_diagnostics "$backend" \
                "$attempt_dir/surface-health.json"; then
            accepted=true
            break
        fi
        if [[ -f "$attempt_dir/surface-health.json" ]] && \
            jq -e '
                .renderer_diagnostics.status == "captured"
            ' "$attempt_dir/surface-health.json" >/dev/null 2>&1 && \
            {
                ! jq -e '
                    (.surfaces | type == "array" and length > 0) and
                    all(.surfaces[];
                        .healthy == true and
                        .realized == true and
                        (.width_px // 0) > 0 and
                        (.height_px // 0) > 0)
                ' "$attempt_dir/surface-health.json" >/dev/null 2>&1 ||
                ! backend_matches_diagnostics "$backend" \
                    "$attempt_dir/surface-health.json"
            }; then
            ((unhealthy_samples += 1))
            if ((unhealthy_samples >= 3)); then
                break
            fi
        else
            unhealthy_samples=0
        fi
        wait_tick
    done

    stop_current_host
    if [[ "$accepted" == true ]]; then
        printf 'renderer-preview: backend accepted: %s\n' "$backend"
        return 0
    fi

    printf 'renderer-preview: backend rejected: %s; fallback=%s\n' \
        "$backend" "${NEXT_BACKEND:-none}" >&2
    return 1
}

write_result() {
    local selected="$1"
    local status="$2"
    local attempted_json
    attempted_json="$(jq -R -s 'split("\n") | map(select(length > 0))' "$ATTEMPTS_FILE")"
    jq -n \
        --arg status "$status" \
        --arg start "$START_BACKEND" \
        --arg selected "$selected" \
        --arg host "$HOST" \
        --arg cli "$CLI" \
        --arg captured_at "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
        --argjson attempted "$attempted_json" \
        '{
            status: $status,
            start_backend: $start,
            selected_backend: (if $selected == "" then null else $selected end),
            attempted_backends: $attempted,
            host: $host,
            cli: $cli,
            captured_at: $captured_at
        }' > "$ARTIFACTS/result.json"
}

backend="$START_BACKEND"
while [[ -n "$backend" ]]; do
    if attempt_backend "$backend"; then
        write_result "$backend" "passed"
        printf 'renderer-preview: PASS selected=%s artifacts=%s\n' "$backend" "$ARTIFACTS"
        exit 0
    else
        attempt_rc=$?
        if ((attempt_rc != 1)); then
            write_result "" "error"
            printf 'renderer-preview: ERROR backend setup failed: %s artifacts=%s\n' \
                "$backend" "$ARTIFACTS" >&2
            exit "$attempt_rc"
        fi
    fi
    backend="$NEXT_BACKEND"
done

write_result "" "failed"
printf 'renderer-preview: FAIL no healthy backend artifacts=%s\n' "$ARTIFACTS" >&2
exit 1
