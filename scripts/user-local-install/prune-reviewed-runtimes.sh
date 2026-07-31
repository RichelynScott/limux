#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF_USAGE'
Usage: prune-reviewed-runtimes.sh [--dry-run|--apply] --reviewed-root <path> [options]

Archives excess Limux user-local reviewed installs, keeping the newest N
installs independently in each runtime lane. Protected current, launcher-linked,
and active-process installs are retained even when they exceed the lane limit.

Options:
  --dry-run                    List retention decisions without changing files
  --apply                      Move excess installs into archive/ and write a TSV manifest
  --reviewed-root <path>       limux-reviewed directory to inspect
  --keep <count>               Newest installs retained per lane (default: 3)
  --current-install-root <path>
                               Newly installed root, always protected
  --current-created-utc <time> Creation timestamp for a prospective dry-run root
  --timestamp <time>           UTC archive batch timestamp
  -h, --help                   Show this help
EOF_USAGE
}

log() {
    printf '%s\n' "$*"
}

die() {
    printf 'prune-reviewed-runtimes: ERROR: %s\n' "$*" >&2
    exit 1
}

mode="dry-run"
reviewed_root=""
keep_count="3"
settle_proc_rescan_seconds="${SETTLE_PRUNE_PROC_RESCAN_SECONDS:-1}"
current_install_root=""
current_created_utc=""
archive_timestamp=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            mode="dry-run"
            shift
            ;;
        --apply)
            mode="apply"
            shift
            ;;
        --reviewed-root)
            [[ $# -ge 2 ]] || die "--reviewed-root requires a value"
            reviewed_root="$2"
            shift 2
            ;;
        --keep)
            [[ $# -ge 2 ]] || die "--keep requires a value"
            keep_count="$2"
            shift 2
            ;;
        --current-install-root)
            [[ $# -ge 2 ]] || die "--current-install-root requires a value"
            current_install_root="$2"
            shift 2
            ;;
        --current-created-utc)
            [[ $# -ge 2 ]] || die "--current-created-utc requires a value"
            current_created_utc="$2"
            shift 2
            ;;
        --timestamp)
            [[ $# -ge 2 ]] || die "--timestamp requires a value"
            archive_timestamp="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done

[[ -n "$reviewed_root" ]] || die "--reviewed-root is required"
[[ "$keep_count" =~ ^[1-9][0-9]*$ ]] || die "--keep must be a positive integer"
[[ "${#keep_count}" -le 6 ]] \
    || die "--keep must be at most 6 digits (got ${keep_count}); cap prevents bash (( )) overflow which would retain every install instead of the requested N"
[[ "$settle_proc_rescan_seconds" =~ ^[0-9]+$ ]] \
    || die "SETTLE_PRUNE_PROC_RESCAN_SECONDS must be a non-negative integer"

if [[ -z "$archive_timestamp" ]]; then
    archive_timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
fi
[[ "$archive_timestamp" =~ ^[0-9]{8}T[0-9]{6}Z$ ]] \
    || die "--timestamp must use YYYYMMDDTHHMMSSZ"

if [[ -z "$current_created_utc" ]]; then
    current_created_utc="$archive_timestamp"
fi
[[ "$current_created_utc" =~ ^[0-9]{8}T[0-9]{6}Z$ ]] \
    || die "--current-created-utc must use YYYYMMDDTHHMMSSZ"

reviewed_root="$(realpath -m -- "$reviewed_root")"
prefix="$(dirname -- "$reviewed_root")"

if [[ "$mode" == "apply" && ! -d "$reviewed_root" ]]; then
    die "reviewed root does not exist in apply mode: ${reviewed_root}"
fi

lane_for_relative_path() {
    local rel="$1"
    local remainder
    local channel
    local install_id

    case "$rel" in
        *$'\t'*|*$'\n'*|*$'\r'*)
            return 1
            ;;
        */*)
            ;;
        *)
            [[ "$rel" != "archive" ]] || return 1
            printf 'legacy\n'
            return 0
            ;;
    esac

    if [[ "$rel" == stable/* ]]; then
        remainder="${rel#stable/}"
        [[ -n "$remainder" && "$remainder" != */* ]] || return 1
        printf 'stable\n'
        return 0
    fi

    if [[ "$rel" == preview/* ]]; then
        remainder="${rel#preview/}"
        channel="${remainder%%/*}"
        [[ "$remainder" == */* ]] || return 1
        install_id="${remainder#*/}"
        [[ -n "$channel" && -n "$install_id" && "$install_id" != */* ]] || return 1
        printf 'preview/%s\n' "$channel"
        return 0
    fi

    return 1
}

created_utc_from_info() {
    local info_path="$1"
    LC_ALL=C sed -n \
        's/^[[:space:]]*"created_utc"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
        "$info_path" \
        | head -1
}

path_is_within() {
    local path="$1"
    local root="$2"
    [[ "$path" == "$root" || "$path" == "${root}/"* ]]
}

declare -a candidate_paths=()
declare -a candidate_rel_paths=()
declare -a candidate_lanes=()
declare -a candidate_created=()
declare -a candidate_bytes=()
declare -a invalid_candidates=()

add_candidate() {
    local path="$1"
    local rel="$2"
    local lane="$3"
    local created="$4"
    local bytes="$5"

    candidate_paths+=("$path")
    candidate_rel_paths+=("$rel")
    candidate_lanes+=("$lane")
    candidate_created+=("$created")
    candidate_bytes+=("$bytes")
}

if [[ -d "$reviewed_root" ]]; then
    while IFS= read -r -d '' info_path; do
        info_path="${reviewed_root}/${info_path#./}"
        candidate_path="$(dirname -- "$info_path")"
        candidate_path="$(realpath -e -- "$candidate_path" 2>/dev/null || true)"
        if [[ -z "$candidate_path" ]] || ! path_is_within "$candidate_path" "$reviewed_root"; then
            invalid_candidates+=("${info_path}:outside-reviewed-root")
            continue
        fi

        rel_path="${candidate_path#"${reviewed_root}/"}"
        lane="$(lane_for_relative_path "$rel_path" || true)"
        if [[ -z "$lane" ]]; then
            invalid_candidates+=("${rel_path}:unrecognized-lane-shape")
            continue
        fi

        created_utc="$(created_utc_from_info "$info_path")"
        if [[ ! "$created_utc" =~ ^[0-9]{8}T[0-9]{6}Z$ ]]; then
            invalid_candidates+=("${rel_path}:invalid-created-utc")
            continue
        fi

        allocated_bytes="$(du -s -B1 -- "$candidate_path" | awk '{print $1}')"
        add_candidate "$candidate_path" "$rel_path" "$lane" "$created_utc" "$allocated_bytes"
    done < <(
        cd "$reviewed_root"
        find . \
            -path './archive' -prune \
            -o -type f -name install-info.json -print0
    )
fi

current_rel_path=""
if [[ -n "$current_install_root" ]]; then
    current_install_root="$(realpath -m -- "$current_install_root")"
    path_is_within "$current_install_root" "$reviewed_root" \
        || die "--current-install-root must be inside --reviewed-root"
    [[ "$current_install_root" != "$reviewed_root" ]] \
        || die "--current-install-root cannot equal --reviewed-root"

    current_rel_path="${current_install_root#"${reviewed_root}/"}"
    current_lane="$(lane_for_relative_path "$current_rel_path" || true)"
    [[ -n "$current_lane" ]] \
        || die "--current-install-root does not match a supported lane shape"

    current_found="false"
    for index in "${!candidate_paths[@]}"; do
        if [[ "${candidate_paths[$index]}" == "$current_install_root" ]]; then
            current_found="true"
            break
        fi
    done
    if [[ "$current_found" != "true" ]]; then
        add_candidate \
            "$current_install_root" \
            "$current_rel_path" \
            "$current_lane" \
            "$current_created_utc" \
            "0"
    fi
fi

declare -A protected_reasons=()
if [[ -n "$current_rel_path" ]]; then
    protected_reasons["$current_rel_path"]="current-install"
fi

for link_dir in "${prefix}/bin" "${prefix}/libexec"; do
    [[ -d "$link_dir" ]] || continue
    while IFS= read -r -d '' link_path; do
        link_target="$(readlink -f -- "$link_path" 2>/dev/null || true)"
        [[ -n "$link_target" ]] || continue
        for index in "${!candidate_paths[@]}"; do
            if path_is_within "$link_target" "${candidate_paths[$index]}"; then
                protected_reasons["${candidate_rel_paths[$index]}"]="launcher-link:${link_path}"
            fi
        done
    done < <(find "$link_dir" -maxdepth 1 -type l -print0)
done

proc_scan_ambiguous="false"
current_uid="$(id -u)"

protect_candidate_from_process() {
    local proc_dir="$1"
    local proc_uid proc_state exe_target exe_basename proc_comm
    local inspect_cmdline="false"
    local index arg arg_target
    local -a proc_argv=()

    [[ -d "$proc_dir" ]] || return 0
    proc_uid="$(stat -c '%u' "$proc_dir" 2>/dev/null || true)"
    [[ "$proc_uid" == "$current_uid" ]] || return 0

    proc_state="$(awk '{print $3}' "${proc_dir}/stat" 2>/dev/null || true)"
    [[ "$proc_state" != "Z" ]] || return 0
    [[ -n "$proc_state" ]] || return 0

    exe_target="$(readlink -- "${proc_dir}/exe" 2>/dev/null || true)"
    exe_basename="${exe_target% (deleted)}"
    exe_basename="${exe_basename##*/}"
    if [[ -z "$exe_target" || "$exe_target" == *" (deleted)" ]]; then
        inspect_cmdline="true"
    else
        case "$exe_basename" in
            bash|sh|dash)
                inspect_cmdline="true"
                ;;
        esac
    fi

    # A wrapper process still points /proc/<pid>/exe at its shell until exec.
    # Match each absolute argv path against candidate roots before pruning.
    if [[ "$inspect_cmdline" == "true" && -r "${proc_dir}/cmdline" ]]; then
        mapfile -d "" -t proc_argv < "${proc_dir}/cmdline" 2>/dev/null || true
        for arg in "${proc_argv[@]}"; do
            [[ "$arg" == /* ]] || continue
            arg_target="$(realpath -m -- "$arg")"
            for index in "${!candidate_paths[@]}"; do
                if path_is_within "$arg_target" "${candidate_paths[$index]}"; then
                    protected_reasons["${candidate_rel_paths[$index]}"]="active-process:${proc_dir#/proc/}:cmdline-match"
                    return 0
                fi
            done
        done
    fi

    if [[ -z "$exe_target" ]]; then
        proc_comm="$(cat "${proc_dir}/comm" 2>/dev/null || true)"
        case "$proc_comm" in
            limux*)
                proc_scan_ambiguous="true"
                ;;
        esac
        return 0
    fi
    exe_target="${exe_target% (deleted)}"
    exe_target="$(realpath -m -- "$exe_target")"

    for index in "${!candidate_paths[@]}"; do
        if path_is_within "$exe_target" "${candidate_paths[$index]}"; then
            protected_reasons["${candidate_rel_paths[$index]}"]="active-process:${proc_dir#/proc/}"
        fi
    done
}

scan_reviewed_runtime_processes() {
    local proc_dir
    for proc_dir in /proc/[0-9]*; do
        protect_candidate_from_process "$proc_dir"
    done
}

scan_reviewed_runtime_processes
if [[ "$settle_proc_rescan_seconds" != "0" ]]; then
    sleep "$settle_proc_rescan_seconds"
    scan_reviewed_runtime_processes
fi

declare -A lane_seen=()
declare -a lanes=()
for lane in "${candidate_lanes[@]}"; do
    if [[ -z "${lane_seen[$lane]:-}" ]]; then
        lane_seen["$lane"]="true"
        lanes+=("$lane")
    fi
done

for lane in "${lanes[@]}"; do
    lane_indices=()
    for index in "${!candidate_paths[@]}"; do
        if [[ "${candidate_lanes[$index]}" == "$lane" ]]; then
            lane_indices+=("$index")
        fi
    done

    for ((left = 0; left < ${#lane_indices[@]}; left++)); do
        newest_pos="$left"
        for ((right = left + 1; right < ${#lane_indices[@]}; right++)); do
            newest_index="${lane_indices[$newest_pos]}"
            candidate_index="${lane_indices[$right]}"
            if [[ "${candidate_created[$candidate_index]}" > "${candidate_created[$newest_index]}" ]] \
                || { [[ "${candidate_created[$candidate_index]}" == "${candidate_created[$newest_index]}" ]] \
                    && [[ "${candidate_rel_paths[$candidate_index]}" > "${candidate_rel_paths[$newest_index]}" ]]; }
            then
                newest_pos="$right"
            fi
        done
        swap="${lane_indices[$left]}"
        lane_indices[$left]="${lane_indices[$newest_pos]}"
        lane_indices[$newest_pos]="$swap"
    done

    for ((position = 0; position < keep_count && position < ${#lane_indices[@]}; position++)); do
        index="${lane_indices[$position]}"
        rel_path="${candidate_rel_paths[$index]}"
        if [[ -z "${protected_reasons[$rel_path]:-}" ]]; then
            protected_reasons["$rel_path"]="retained-newest-${keep_count}"
        fi
    done
done

declare -a archive_indices=()
if [[ "$proc_scan_ambiguous" == "true" ]]; then
    for index in "${!candidate_paths[@]}"; do
        rel_path="${candidate_rel_paths[$index]}"
        if [[ -z "${protected_reasons[$rel_path]:-}" ]]; then
            protected_reasons["$rel_path"]="proc-scan-ambiguous"
        fi
    done
else
    for index in "${!candidate_paths[@]}"; do
        rel_path="${candidate_rel_paths[$index]}"
        if [[ -z "${protected_reasons[$rel_path]:-}" ]]; then
            archive_indices+=("$index")
        fi
    done
fi

archive_parent="${reviewed_root}/archive/${archive_timestamp}"
archive_root="${reviewed_root}/archive"
suffix=0
while [[ -e "$archive_parent" || -L "$archive_parent" ]]; do
    suffix=$((suffix + 1))
    archive_parent="${reviewed_root}/archive/${archive_timestamp}-${suffix}"
done
archive_batch_root="${archive_parent}/reviewed-runtimes"
manifest_path="${archive_batch_root}/MANIFEST.tsv"
global_manifest_path="${archive_root}/MANIFEST.tsv"
manifest_header=$'source_relative_directory\tarchive_relative_path\tarchived_utc\tallocated_bytes\treason'

log "RETENTION mode=${mode} root=${reviewed_root} keep=${keep_count}"
for invalid in "${invalid_candidates[@]}"; do
    log "SKIP_INVALID ${invalid}"
done
if [[ "$proc_scan_ambiguous" == "true" ]]; then
    log "SKIP_ARCHIVAL proc-scan-ambiguous"
fi

for index in "${!candidate_paths[@]}"; do
    rel_path="${candidate_rel_paths[$index]}"
    reason="${protected_reasons[$rel_path]:-retention-excess}"
    if [[ -n "${protected_reasons[$rel_path]:-}" ]]; then
        log "KEEP lane=${candidate_lanes[$index]} path=${rel_path} created=${candidate_created[$index]} reason=${reason}"
    fi
done

if [[ ${#archive_indices[@]} -eq 0 ]]; then
    log "NO_ARCHIVE retention already satisfied"
    exit 0
fi

if [[ "$mode" == "dry-run" ]]; then
    for index in "${archive_indices[@]}"; do
        log "WOULD_ARCHIVE lane=${candidate_lanes[$index]} path=${candidate_rel_paths[$index]} bytes=${candidate_bytes[$index]} reason=retention-excess destination=${archive_batch_root}/${candidate_rel_paths[$index]}"
    done
    log "WOULD_WRITE_MANIFEST ${manifest_path}"
    log "WOULD_APPEND_MANIFEST ${global_manifest_path}"
    exit 0
fi

mkdir -p "$archive_batch_root"
if [[ -L "$global_manifest_path" ]] \
    || { [[ -e "$global_manifest_path" ]] && [[ ! -f "$global_manifest_path" ]]; }
then
    die "global manifest must be a regular non-symlink file: ${global_manifest_path}"
fi
if [[ -f "$global_manifest_path" ]]; then
    existing_header="$(head -1 "$global_manifest_path")"
    [[ "$existing_header" == "$manifest_header" ]] \
        || die "global manifest has an unexpected header: ${global_manifest_path}"
else
    printf '%s\n' "$manifest_header" > "$global_manifest_path"
fi
printf '%s\n' "$manifest_header" > "$manifest_path"
for index in "${archive_indices[@]}"; do
    source_path="${candidate_paths[$index]}"
    rel_path="${candidate_rel_paths[$index]}"
    destination="${archive_batch_root}/${rel_path}"

    [[ -d "$source_path" ]] \
        || die "candidate disappeared before archival: ${source_path}"
    [[ ! -e "$destination" && ! -L "$destination" ]] \
        || die "archive destination already exists: ${destination}"

    mkdir -p "$(dirname -- "$destination")"
    mv "$source_path" "$destination"
    archive_relative_path="${destination#"${archive_root}/"}"
    printf '%s\t%s\t%s\t%s\tretention-excess\n' \
        "$rel_path" \
        "$archive_relative_path" \
        "$archive_timestamp" \
        "${candidate_bytes[$index]}" \
        >> "$global_manifest_path"
    printf '%s\t%s\t%s\t%s\tretention-excess\n' \
        "$rel_path" \
        "$archive_relative_path" \
        "$archive_timestamp" \
        "${candidate_bytes[$index]}" \
        >> "$manifest_path"
    log "ARCHIVED lane=${candidate_lanes[$index]} path=${rel_path} bytes=${candidate_bytes[$index]} destination=${destination}"
done
log "WROTE_MANIFEST ${manifest_path}"
log "APPENDED_MANIFEST ${global_manifest_path}"
