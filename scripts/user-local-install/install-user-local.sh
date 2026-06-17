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
  --desktop-entry          Install a user desktop entry under ~/.local/share/applications
  --no-desktop-entry       Do not install a desktop entry (default)
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

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

mode="dry-run"
profile="${LIMUX_LOCAL_PROFILE:-release}"
prefix="${LIMUX_USER_PREFIX:-${HOME}/.local}"
install_id=""
desktop_entry="false"
manifest_out=""

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
        --desktop-entry)
            desktop_entry="true"
            shift
            ;;
        --no-desktop-entry)
            desktop_entry="false"
            shift
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

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
install_root="${prefix}/limux-reviewed/${install_id}"
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

[[ -x "$cli_src" ]] || die "missing executable CLI artifact: ${cli_src}"
[[ -x "$host_src" ]] || die "missing executable GTK host artifact: ${host_src}"
[[ -f "$ghostty_lib_src" ]] || die "missing Ghostty library: ${ghostty_lib_src}"

if [[ "$mode" == "apply" && -e "$install_root" ]]; then
    die "install root already exists; choose --install-id ${install_id}-${timestamp} if you want another copy: ${install_root}"
fi

ghostty_share_src=""
for candidate in \
    "${repo_root}/ghostty/zig-out/share/ghostty" \
    "/usr/local/share/ghostty" \
    "/usr/share/ghostty"
do
    if [[ -d "$candidate/themes" && -d "$candidate/shell-integration" ]]; then
        ghostty_share_src="$candidate"
        break
    fi
done

ghostty_terminfo_src=""
for candidate in \
    "${repo_root}/ghostty/zig-out/share/terminfo" \
    "/usr/local/share/terminfo" \
    "/usr/share/terminfo"
do
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
        run cp -a "$source" "$dest"
    else
        plan "skip missing ${label}: ${source}"
    fi
}

wrapper_content() {
    local root="$1"
    cat <<EOF_WRAPPER
#!/usr/bin/env bash
set -euo pipefail

INSTALL_ROOT="${root}"
export LD_LIBRARY_PATH="\${INSTALL_ROOT}/lib\${LD_LIBRARY_PATH:+:\${LD_LIBRARY_PATH}}"
export LIMUX_HOST_BIN="\${INSTALL_ROOT}/libexec/limux-host"
export XDG_DATA_DIRS="\${INSTALL_ROOT}/share\${XDG_DATA_DIRS:+:\${XDG_DATA_DIRS}}"

exec "\${INSTALL_ROOT}/libexec/limux-cli" "\$@"
EOF_WRAPPER
}

desktop_content() {
    local exec_path="$1"
    sed \
        -e "s|^Exec=.*|Exec=${exec_path}|" \
        -e "s|^TryExec=.*|TryExec=${exec_path}|" \
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
    run cp -a "${ghostty_share_src}/." "${install_root}/share/limux/ghostty/"
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

write_file "${install_root}/bin/limux" "$(wrapper_content "$install_root")"
write_file "${install_root}/bin/limux-cli" "$(wrapper_content "$install_root")"
run chmod 755 "${install_root}/bin/limux" "${install_root}/bin/limux-cli"
run chmod 755 "${install_root}/libexec/limux-cli" "${install_root}/libexec/limux-host"

if [[ -f "$desktop_src" ]]; then
    write_file "${install_root}/share/applications/dev.limux.linux.desktop" "$(desktop_content "${bin_link_dir}/limux")"
else
    plan "skip missing desktop entry source: ${desktop_src}"
fi
copy_if_present "$metainfo_src" "${install_root}/share/metainfo/dev.limux.linux.metainfo.xml" "metainfo"
if [[ -d "$icons_src" ]]; then
    run cp -a "${icons_src}/." "${install_root}/share/icons/"
else
    plan "skip missing icons source: ${icons_src}"
fi

run mkdir -p "$bin_link_dir"
install_symlink "${install_root}/bin/limux" "${bin_link_dir}/limux"
install_symlink "${install_root}/bin/limux-cli" "${bin_link_dir}/limux-cli"

if [[ "$desktop_entry" == "true" ]]; then
    run mkdir -p "$app_dir"
    archive_existing_path "${app_dir}/dev.limux.linux.desktop" "desktop entry"
    run cp "${install_root}/share/applications/dev.limux.linux.desktop" "${app_dir}/dev.limux.linux.desktop"
fi

manifest="$(
    cat <<EOF_MANIFEST
# Limux User-Local Install Manifest

Mode: ${mode}
Timestamp UTC: ${timestamp}
Repo: ${repo_root}
Install ID: ${install_id}
Install root: ${install_root}
Profile: ${profile}
Desktop entry: ${desktop_entry}

## Source Artifacts

- CLI: ${cli_src}
- Host: ${host_src}
- Ghostty library: ${ghostty_lib_src}
- Ghostty resources: ${ghostty_share_src:-not found}
- Ghostty terminfo: ${ghostty_terminfo_src:-not found}

## User Links

- ${bin_link_dir}/limux -> ${install_root}/bin/limux
- ${bin_link_dir}/limux-cli -> ${install_root}/bin/limux-cli

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
            bin/limux \
            bin/limux-cli \
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
