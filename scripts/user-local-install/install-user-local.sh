#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF_USAGE'
Usage: scripts/user-local-install/install-user-local.sh [--dry-run|--apply] [options]

Stages the already-built Limux artifacts into a no-sudo user-local install:
  ~/.local/limux-reviewed/<git-sha>/

The script does not build, fetch packages, use sudo, write /etc, or run the
root/global installer. It is dry-run by default.

Options:
  --dry-run                Print planned actions and manifest output (default)
  --apply                  Perform the user-local install
  --profile <release|debug>
                           Select target/<profile> artifacts (default: release)
  --prefix <path>          User prefix (default: ~/.local)
  --install-id <id>        Install id under limux-reviewed (default: git sha)
  --channel <channel>      Install launcher lane: legacy, stable, preview, preview:<id>
                           (default: legacy)
  --desktop-entry          Install a user desktop entry under ~/.local/share/applications
  --no-desktop-entry       Do not install a desktop entry (default)
  --ghostty-share <path>   Ghostty runtime share dir containing shell-integration
  --ghostty-terminfo <path>
                           Terminfo dir containing g/ghostty or x/xterm-ghostty
  --manifest-out <path>    Also write the dry-run manifest to this path
  -h, --help               Show this help
EOF_USAGE
}

log() {
    printf '%s\n' "$*"
}

die() {
    printf 'install-user-local: ERROR: %s\n' "$*" >&2
    exit 1
}

assert_split_icon_svg() {
    local path="$1"
    local prefix

    if [[ ! -s "$path" ]]; then
        die "split icon source is missing or empty: ${path}"
    fi

    prefix="$(LC_ALL=C head -c 5 "$path")"
    case "$prefix" in
        "<svg "|"<?xml") ;;
        *) die "split icon source does not start with SVG/XML bytes: ${path}" ;;
    esac

    LC_ALL=C grep -Eq '<svg([[:space:]>])' "$path" \
        || die "split icon source is missing an <svg> element: ${path}"
    LC_ALL=C grep -q '</svg>' "$path" \
        || die "split icon source is missing a closing </svg>: ${path}"
}

validate_split_icon_sources() {
    local icons_dir="$1"
    local rel

    for rel in \
        "limux-split-horizontal-symbolic.svg" \
        "limux-split-vertical-symbolic.svg" \
        "hicolor/scalable/actions/limux-split-horizontal-symbolic.svg" \
        "hicolor/scalable/actions/limux-split-vertical-symbolic.svg"
    do
        assert_split_icon_svg "${icons_dir}/${rel}"
    done
}

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

mode="dry-run"
profile="${LIMUX_LOCAL_PROFILE:-release}"
prefix="${LIMUX_USER_PREFIX:-${HOME}/.local}"
install_id=""
runtime_channel="legacy"
desktop_entry="false"
manifest_out=""
ghostty_share_override="${LIMUX_GHOSTTY_SHARE_DIR:-}"
ghostty_terminfo_override="${LIMUX_GHOSTTY_TERMINFO_DIR:-}"

parse_runtime_channel() {
    local raw="$1"
    local id

    case "$raw" in
        legacy|stable)
            printf '%s\n' "$raw"
            ;;
        preview)
            printf 'preview:default\n'
            ;;
        preview:*|preview/*)
            id="${raw#preview:}"
            id="${id#preview/}"
            if [[ -z "$id" || "$id" == "." || "$id" == ".." || ! "$id" =~ ^[A-Za-z0-9_-]+$ ]]; then
                die "unsafe preview channel id: ${id:-<empty>}"
            fi
            printf 'preview:%s\n' "$id"
            ;;
        *)
            die "--channel must be legacy, stable, preview, or preview:<id>, got: ${raw}"
            ;;
    esac
}

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
        --profile)
            [[ $# -ge 2 ]] || die "--profile requires a value"
            profile="$2"
            shift 2
            ;;
        --prefix)
            [[ $# -ge 2 ]] || die "--prefix requires a value"
            prefix="$2"
            shift 2
            ;;
        --install-id)
            [[ $# -ge 2 ]] || die "--install-id requires a value"
            install_id="$2"
            shift 2
            ;;
        --channel)
            [[ $# -ge 2 ]] || die "--channel requires a value"
            runtime_channel="$(parse_runtime_channel "$2")"
            shift 2
            ;;
        --desktop-entry)
            desktop_entry="true"
            shift
            ;;
        --no-desktop-entry)
            desktop_entry="false"
            shift
            ;;
        --ghostty-share)
            [[ $# -ge 2 ]] || die "--ghostty-share requires a value"
            ghostty_share_override="$2"
            shift 2
            ;;
        --ghostty-terminfo)
            [[ $# -ge 2 ]] || die "--ghostty-terminfo requires a value"
            ghostty_terminfo_override="$2"
            shift 2
            ;;
        --manifest-out)
            [[ $# -ge 2 ]] || die "--manifest-out requires a value"
            manifest_out="$2"
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

case "$profile" in
    release|debug) ;;
    *) die "--profile must be release or debug, got: ${profile}" ;;
esac

if [[ -z "$install_id" ]]; then
    install_id="$(git -C "$repo_root" rev-parse --short=12 HEAD 2>/dev/null || true)"
fi
if [[ -z "$install_id" ]]; then
    install_id="$(date -u +%Y%m%dT%H%M%SZ)"
fi
if [[ "$install_id" == *"/"* || "$install_id" == "." || "$install_id" == ".." ]]; then
    die "unsafe install id: ${install_id}"
fi

channel_kind="$runtime_channel"
channel_id=""
install_subdir="$install_id"
launcher_name="limux"
cli_launcher_name="limux-cli"
desktop_file_name="dev.limux.linux.desktop"
desktop_display_name="Limux"
wrapper_cli_args=()

case "$runtime_channel" in
    legacy)
        ;;
    stable)
        install_subdir="stable/${install_id}"
        launcher_name="limux-stable"
        cli_launcher_name="limux-stable-cli"
        desktop_file_name="dev.limux.linux.stable.desktop"
        desktop_display_name="Limux Stable"
        wrapper_cli_args=("--channel" "stable")
        ;;
    preview:*)
        channel_kind="preview"
        channel_id="${runtime_channel#preview:}"
        install_subdir="preview/${channel_id}/${install_id}"
        if [[ "$channel_id" == "default" ]]; then
            launcher_name="limux-preview"
            cli_launcher_name="limux-preview-cli"
            desktop_file_name="dev.limux.linux.preview.desktop"
            desktop_display_name="Limux Preview"
        else
            launcher_name="limux-preview-${channel_id}"
            cli_launcher_name="limux-preview-${channel_id}-cli"
            desktop_file_name="dev.limux.linux.preview-${channel_id}.desktop"
            desktop_display_name="Limux Preview ${channel_id}"
        fi
        wrapper_cli_args=("--channel" "$runtime_channel")
        ;;
esac

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
install_root="${prefix}/limux-reviewed/${install_subdir}"
archive_dir="${prefix}/limux-reviewed/archive/${timestamp}"
bin_link_dir="${prefix}/bin"
app_dir="${prefix}/share/applications"

target_dir="${repo_root}/target/${profile}"
cli_src="${target_dir}/limux-cli"
host_src="${target_dir}/limux"
ghostty_lib_src="${repo_root}/ghostty/zig-out/lib/libghostty.so"
desktop_src="${repo_root}/rust/limux-host-linux/dev.limux.linux.desktop"
metainfo_src="${repo_root}/rust/limux-host-linux/dev.limux.linux.metainfo.xml"
icons_src="${repo_root}/rust/limux-host-linux/icons"

validate_split_icon_sources "$icons_src"

[[ -x "$cli_src" ]] || die "missing executable CLI artifact: ${cli_src}"
[[ -x "$host_src" ]] || die "missing executable GTK host artifact: ${host_src}"
[[ -f "$ghostty_lib_src" ]] || die "missing Ghostty library: ${ghostty_lib_src}"

if [[ "$mode" == "apply" && -e "$install_root" ]]; then
    die "install root already exists; choose --install-id ${install_id}-${timestamp} if you want another copy: ${install_root}"
fi

ghostty_share_src=""
for candidate in \
    "$ghostty_share_override" \
    "${repo_root}/ghostty/zig-out/share/ghostty" \
    "/usr/local/share/ghostty" \
    "/usr/share/ghostty"
do
    [[ -n "$candidate" ]] || continue
    if [[ -d "$candidate/shell-integration" ]]; then
        ghostty_share_src="$candidate"
        break
    fi
done

ghostty_terminfo_src=""
for candidate in \
    "$ghostty_terminfo_override" \
    "${repo_root}/ghostty/zig-out/share/terminfo" \
    "/usr/local/share/terminfo" \
    "/usr/share/terminfo"
do
    [[ -n "$candidate" ]] || continue
    if [[ -f "$candidate/g/ghostty" || -f "$candidate/x/xterm-ghostty" ]]; then
        ghostty_terminfo_src="$candidate"
        break
    fi
done

declare -a planned_actions=()

plan() {
    planned_actions+=("$*")
}

run() {
    plan "$*"
    if [[ "$mode" == "apply" ]]; then
        "$@"
    fi
}

write_file() {
    local path="$1"
    local content="$2"

    plan "write ${path}"
    if [[ "$mode" == "apply" ]]; then
        printf '%s' "$content" > "$path"
    fi
}

archive_existing_path() {
    local path="$1"
    local label="$2"

    if [[ ! -e "$path" && ! -L "$path" ]]; then
        return 0
    fi
    if [[ -d "$path" && ! -L "$path" ]]; then
        die "refusing to replace existing directory ${label}: ${path}"
    fi

    run mkdir -p "$archive_dir"
    run mv "$path" "${archive_dir}/$(basename "$path")"
}

install_symlink() {
    local target="$1"
    local link="$2"

    archive_existing_path "$link" "symlink"
    run ln -s "$target" "$link"
}

copy_if_present() {
    local source="$1"
    local dest="$2"
    local label="$3"

    if [[ -e "$source" ]]; then
        if [[ -d "$source" ]]; then
            run cp -R "$source" "$dest"
        else
            run cp "$source" "$dest"
        fi
    else
        plan "skip missing ${label}: ${source}"
    fi
}

wrapper_content() {
    local root="$1"
    local channel="$2"
    shift 2
    local cli_args=("$@")
    local quoted_cli_args=""
    local arg

    for arg in "${cli_args[@]}"; do
        quoted_cli_args+=" \"${arg//\"/\\\"}\""
    done

    cat <<EOF_WRAPPER
#!/usr/bin/env bash
set -euo pipefail

INSTALL_ROOT="${root}"
export LD_LIBRARY_PATH="\${INSTALL_ROOT}/lib\${LD_LIBRARY_PATH:+:\${LD_LIBRARY_PATH}}"
export LIMUX_HOST_BIN="\${INSTALL_ROOT}/libexec/limux-host"
export XDG_DATA_DIRS="\${INSTALL_ROOT}/share\${XDG_DATA_DIRS:+:\${XDG_DATA_DIRS}}:/usr/local/share:/usr/share"
$(if [[ "$channel" != "legacy" ]]; then printf 'export LIMUX_CHANNEL="%s"\n' "$channel"; fi)

exec "\${INSTALL_ROOT}/libexec/limux-cli"${quoted_cli_args} "\$@"
EOF_WRAPPER
}

desktop_content() {
    local exec_path="$1"
    local display_name="$2"
    sed \
        -e "s|^Exec=.*|Exec=${exec_path}|" \
        -e "s|^TryExec=.*|TryExec=${exec_path}|" \
        -e "s|^Name=.*|Name=${display_name}|" \
        "$desktop_src"
}

run mkdir -p \
    "${install_root}/bin" \
    "${install_root}/libexec" \
    "${install_root}/lib" \
    "${install_root}/share/limux" \
    "${install_root}/share/applications" \
    "${install_root}/share/metainfo" \
    "${install_root}/share/icons"

run cp "$cli_src" "${install_root}/libexec/limux-cli"
run cp "$host_src" "${install_root}/libexec/limux-host"
run cp "$ghostty_lib_src" "${install_root}/lib/libghostty.so"

if [[ -n "$ghostty_share_src" ]]; then
    run mkdir -p "${install_root}/share/limux/ghostty"
    run cp -R "${ghostty_share_src}/." "${install_root}/share/limux/ghostty/"
else
    plan "warning: no Ghostty resource directory found; installed host will fall back to built-in/default resource behavior"
fi

if [[ -n "$ghostty_terminfo_src" ]]; then
    run mkdir -p "${install_root}/share/limux/terminfo"
    if [[ -f "${ghostty_terminfo_src}/g/ghostty" ]]; then
        run mkdir -p "${install_root}/share/limux/terminfo/g"
        run cp "${ghostty_terminfo_src}/g/ghostty" "${install_root}/share/limux/terminfo/g/ghostty"
    fi
    if [[ -f "${ghostty_terminfo_src}/x/xterm-ghostty" ]]; then
        run mkdir -p "${install_root}/share/limux/terminfo/x"
        run cp "${ghostty_terminfo_src}/x/xterm-ghostty" "${install_root}/share/limux/terminfo/x/xterm-ghostty"
    fi
else
    plan "warning: no Ghostty terminfo entries found"
fi

write_file "${install_root}/bin/${launcher_name}" "$(wrapper_content "$install_root" "$runtime_channel" "${wrapper_cli_args[@]}")"
write_file "${install_root}/bin/${cli_launcher_name}" "$(wrapper_content "$install_root" "$runtime_channel" "${wrapper_cli_args[@]}")"
run chmod 755 "${install_root}/bin/${launcher_name}" "${install_root}/bin/${cli_launcher_name}"
run chmod 755 "${install_root}/libexec/limux-cli" "${install_root}/libexec/limux-host"

if [[ -f "$desktop_src" ]]; then
    write_file "${install_root}/share/applications/${desktop_file_name}" "$(desktop_content "${bin_link_dir}/${launcher_name}" "$desktop_display_name")"
else
    plan "skip missing desktop entry source: ${desktop_src}"
fi
copy_if_present "$metainfo_src" "${install_root}/share/metainfo/dev.limux.linux.metainfo.xml" "metainfo"
if [[ -d "$icons_src" ]]; then
    run cp -R "${icons_src}/." "${install_root}/share/icons/"
else
    plan "skip missing icons source: ${icons_src}"
fi

run mkdir -p "$bin_link_dir"
install_symlink "${install_root}/bin/${launcher_name}" "${bin_link_dir}/${launcher_name}"
install_symlink "${install_root}/bin/${cli_launcher_name}" "${bin_link_dir}/${cli_launcher_name}"

if [[ "$desktop_entry" == "true" ]]; then
    run mkdir -p "$app_dir"
    archive_existing_path "${app_dir}/${desktop_file_name}" "desktop entry"
    run cp "${install_root}/share/applications/${desktop_file_name}" "${app_dir}/${desktop_file_name}"
fi

manifest="$(
    cat <<EOF_MANIFEST
# Limux User-Local Install Manifest

Mode: ${mode}
Timestamp UTC: ${timestamp}
Repo: ${repo_root}
Install ID: ${install_id}
Runtime channel: ${runtime_channel}
Runtime kind: ${channel_kind}
Preview channel id: ${channel_id:-n/a}
Install root: ${install_root}
Profile: ${profile}
Desktop entry: ${desktop_entry}
Launcher: ${bin_link_dir}/${launcher_name}
CLI launcher: ${bin_link_dir}/${cli_launcher_name}

## Source Artifacts

- CLI: ${cli_src}
- Host: ${host_src}
- Ghostty library: ${ghostty_lib_src}
- Ghostty resources: ${ghostty_share_src:-not found}
- Ghostty terminfo: ${ghostty_terminfo_src:-not found}

## User Links

- ${bin_link_dir}/${launcher_name} -> ${install_root}/bin/${launcher_name}
- ${bin_link_dir}/${cli_launcher_name} -> ${install_root}/bin/${cli_launcher_name}

## Archive Directory For Replaced Links

${archive_dir}

## Safety Boundary

- No sudo.
- No package manager.
- No build step.
- No /etc writes.
- Existing link/file targets are moved into the archive directory, not deleted.
- Browser/WebKit use remains gated separately.
EOF_MANIFEST
)"

write_file "${install_root}/MANIFEST.md" "${manifest}"$'\n'

if [[ "$mode" == "apply" ]]; then
    (
        cd "$install_root"
        sha256sum \
            "bin/${launcher_name}" \
            "bin/${cli_launcher_name}" \
            libexec/limux-cli \
            libexec/limux-host \
            lib/libghostty.so \
            MANIFEST.md \
            > SHA256SUMS
    )
fi

if [[ -n "$manifest_out" ]]; then
    printf '%s\n' "$manifest" > "$manifest_out"
fi

log "Limux user-local install lane (${mode})"
log ""
log "$manifest"
log ""
log "Planned actions:"
for action in "${planned_actions[@]}"; do
    log "- ${action}"
done

if [[ "$mode" == "dry-run" ]]; then
    log ""
    log "Dry-run only. Re-run with --apply to install."
fi
