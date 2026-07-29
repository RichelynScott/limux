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
                           (default: legacy). Stable also promotes the plain
                           limux/limux-cli aliases; legacy remains explicit.
  --keep-reviewed <count>  Keep newest reviewed installs per lane (default: 3;
                           env: LIMUX_REVIEWED_KEEP_LAST)
  --desktop-entry          Install a user desktop entry under ~/.local/share/applications
  --no-desktop-entry       Do not install a desktop entry (default)
  --ghostty-share <path>   Ghostty runtime share dir containing shell-integration
  --ghostty-terminfo <path>
                           Terminfo dir containing g/ghostty or x/xterm-ghostty
  --allow-missing-ghostty-resources
                           Proceed even when no valid Ghostty resource bundle
                           can be staged (stamps the manifest DEGRADED)
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

json_escape() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    value="${value//$'\n'/\\n}"
    printf '%s' "$value"
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
allow_missing_ghostty="false"
keep_reviewed="${LIMUX_REVIEWED_KEEP_LAST:-3}"

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
        --keep-reviewed)
            [[ $# -ge 2 ]] || die "--keep-reviewed requires a value"
            keep_reviewed="$2"
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
        --allow-missing-ghostty-resources)
            allow_missing_ghostty="true"
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
[[ "$keep_reviewed" =~ ^[1-9][0-9]*$ ]] \
    || die "--keep-reviewed must be a positive integer, got: ${keep_reviewed}"

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
promote_default_aliases="false"

case "$runtime_channel" in
    legacy)
        launcher_name="limux-legacy"
        cli_launcher_name="limux-legacy-cli"
        desktop_file_name="dev.limux.linux.legacy.desktop"
        desktop_display_name="Limux Legacy"
        ;;
    stable)
        install_subdir="stable/${install_id}"
        launcher_name="limux-stable"
        cli_launcher_name="limux-stable-cli"
        desktop_file_name="dev.limux.linux.stable.desktop"
        desktop_display_name="Limux Stable"
        wrapper_cli_args=("--channel" "stable")
        promote_default_aliases="true"
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
source_sha="$(git -C "$repo_root" rev-parse --verify HEAD 2>/dev/null || true)"
if [[ -z "$source_sha" ]]; then
    source_sha="unknown"
fi
cargo_version="$(grep '^version' "$repo_root/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')"
if [[ -z "$cargo_version" ]]; then
    cargo_version="unknown"
fi
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

# --- Ghostty runtime resource resolution ------------------------------------
#
# The staged bundle must satisfy the host's runtime contract — keep this in
# sync with `is_ghostty_resources_dir` + `has_ghostty_terminfo` in
# rust/limux-host-linux/src/main.rs (which carry a mirror comment pointing
# back here):
#
#   <install-root>/share/limux/ghostty/shell-integration/   (directory)
#   <install-root>/share/limux/terminfo/x/xterm-ghostty     (compiled entry
#   and/or .../terminfo/g/ghostty                             FILES, sibling of
#                                                             the ghostty/ dir,
#                                                             never nested in it)
#
# A "source-only" shape (ghostty/src-style: shell-integration without compiled
# sibling terminfo) caused the 2026-06-29 terminal-input regression
# (docs/terminal-input-regression-20260701.md) and is rejected below.

ghostty_source_tree_marker() {
    # ghostty/src carries its terminfo as Zig code, not compiled entries —
    # its presence identifies a Ghostty SOURCE tree, never a runtime share dir.
    [[ -f "$1/terminfo/ghostty.zig" ]]
}

ghostty_share_src=""
if [[ -n "$ghostty_share_override" ]]; then
    if ghostty_source_tree_marker "$ghostty_share_override"; then
        die "refusing --ghostty-share ${ghostty_share_override}: this is a Ghostty SOURCE tree (ghostty/src-style), not runtime resources. Source-only shapes caused the 2026-06-29 terminal-input regression (docs/terminal-input-regression-20260701.md). Use ghostty/zig-out/share/ghostty from a Ghostty build, or drop the flag to stage the vendored snapshot."
    fi
    [[ -d "${ghostty_share_override}/shell-integration" ]] \
        || die "--ghostty-share ${ghostty_share_override} has no shell-integration/ directory"
    ghostty_share_src="$ghostty_share_override"
else
    for candidate in \
        "${repo_root}/ghostty/zig-out/share/ghostty" \
        "/usr/local/share/ghostty" \
        "/usr/share/ghostty"
    do
        if [[ -d "$candidate/shell-integration" ]] && ! ghostty_source_tree_marker "$candidate"; then
            ghostty_share_src="$candidate"
            break
        fi
    done
fi

# Vendored fallback for shell integration: copy ONLY shell-integration/ out of
# the (read-only) ghostty source tree — never the tree itself.
vendored_shell_integration_src="${repo_root}/ghostty/src/shell-integration"

ghostty_terminfo_src=""
terminfo_plan="none"
terminfo_unavailable_reason="no compiled terminfo entries found in any candidate directory"
if [[ -n "$ghostty_terminfo_override" ]]; then
    # An explicit override pins the terminfo source: no fallback beyond it.
    if [[ -f "${ghostty_terminfo_override}/g/ghostty" || -f "${ghostty_terminfo_override}/x/xterm-ghostty" ]]; then
        ghostty_terminfo_src="$ghostty_terminfo_override"
        terminfo_plan="copy"
    else
        terminfo_unavailable_reason="--ghostty-terminfo ${ghostty_terminfo_override} contains no compiled entry files (g/ghostty or x/xterm-ghostty)"
    fi
else
    for candidate in \
        "${repo_root}/ghostty/zig-out/share/terminfo" \
        "/usr/local/share/terminfo" \
        "/usr/share/terminfo"
    do
        if [[ -f "$candidate/g/ghostty" || -f "$candidate/x/xterm-ghostty" ]]; then
            ghostty_terminfo_src="$candidate"
            terminfo_plan="copy"
            break
        fi
    done
fi

# Default path: compile the vendored terminfo snapshot with tic at install
# time (see the snapshot's provenance header for how it is produced).
terminfo_snapshot="${script_dir}/resources/ghostty.terminfo"
if [[ "$terminfo_plan" == "none" && -z "$ghostty_terminfo_override" ]]; then
    if [[ ! -f "$terminfo_snapshot" ]]; then
        terminfo_unavailable_reason="vendored snapshot missing: ${terminfo_snapshot}"
    elif ! command -v tic >/dev/null 2>&1; then
        terminfo_unavailable_reason="tic not found — install ncurses-bin (Debian/Ubuntu: sudo apt install ncurses-bin) to compile the vendored terminfo snapshot"
    else
        terminfo_plan="tic"
    fi
fi

shell_integration_origin=""
if [[ -n "$ghostty_share_src" ]]; then
    shell_integration_origin="prebuilt share dir: ${ghostty_share_src}"
elif [[ -d "$vendored_shell_integration_src" ]]; then
    shell_integration_origin="vendored ghostty source tree (shell-integration/ only)"
fi

ghostty_resource_shape="DEGRADED"
if [[ -n "$shell_integration_origin" && "$terminfo_plan" != "none" ]]; then
    ghostty_resource_shape="valid"
fi

case "$terminfo_plan" in
    copy) terminfo_origin="compiled entries copied from: ${ghostty_terminfo_src}" ;;
    tic)  terminfo_origin="vendored snapshot scripts/user-local-install/resources/ghostty.terminfo, tic-compiled at install time" ;;
    *)    terminfo_origin="none (${terminfo_unavailable_reason})" ;;
esac

ghostty_resource_origin="shell-integration: ${shell_integration_origin:-none}; terminfo: ${terminfo_origin}"

if [[ "$ghostty_resource_shape" != "valid" && "$allow_missing_ghostty" != "true" ]]; then
    missing_detail=""
    if [[ -z "$shell_integration_origin" ]]; then
        missing_detail="no shell-integration source found (run: git submodule update --init ghostty — checkout only, no build)"
    else
        missing_detail="shell-integration is available but no compiled terminfo can be staged: ${terminfo_unavailable_reason}. A source-only bundle (shell-integration without compiled sibling terminfo) is the exact shape behind the 2026-06-29 terminal-input regression and the runtime would reject it."
    fi
    die "cannot stage a valid Ghostty resource bundle — ${missing_detail}
expected install shape: share/limux/ghostty/shell-integration/ plus sibling share/limux/terminfo/x/xterm-ghostty (and/or g/ghostty) entry FILES (contract: is_ghostty_resources_dir, rust/limux-host-linux/src/main.rs)
pass --allow-missing-ghostty-resources to install anyway (manifest will be stamped DEGRADED)"
fi

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
elif [[ -d "$vendored_shell_integration_src" ]]; then
    run mkdir -p "${install_root}/share/limux/ghostty"
    run cp -R "$vendored_shell_integration_src" "${install_root}/share/limux/ghostty/shell-integration"
else
    plan "warning: DEGRADED install — no Ghostty shell-integration staged"
fi

case "$terminfo_plan" in
    copy)
        run mkdir -p "${install_root}/share/limux/terminfo"
        if [[ -f "${ghostty_terminfo_src}/g/ghostty" ]]; then
            run mkdir -p "${install_root}/share/limux/terminfo/g"
            run cp "${ghostty_terminfo_src}/g/ghostty" "${install_root}/share/limux/terminfo/g/ghostty"
        fi
        if [[ -f "${ghostty_terminfo_src}/x/xterm-ghostty" ]]; then
            run mkdir -p "${install_root}/share/limux/terminfo/x"
            run cp "${ghostty_terminfo_src}/x/xterm-ghostty" "${install_root}/share/limux/terminfo/x/xterm-ghostty"
        fi
        ;;
    tic)
        run mkdir -p "${install_root}/share/limux/terminfo"
        run tic -x -o "${install_root}/share/limux/terminfo" "$terminfo_snapshot"
        ;;
    *)
        plan "warning: DEGRADED install — no Ghostty terminfo staged"
        ;;
esac

if [[ "$mode" == "apply" && "$ghostty_resource_shape" == "valid" ]]; then
    # Post-stage verification of the runtime contract on the real files.
    # `tic -x -o` sets the database LOCATION only; whether it writes a
    # directory tree (x/xterm-ghostty) or a hashed db depends on the ncurses
    # build — so assert the entry FILES the runtime checks actually exist.
    [[ -d "${install_root}/share/limux/ghostty/shell-integration" ]] \
        || die "post-stage check failed: ${install_root}/share/limux/ghostty/shell-integration is not a directory"
    [[ -f "${install_root}/share/limux/terminfo/x/xterm-ghostty" || -f "${install_root}/share/limux/terminfo/g/ghostty" ]] \
        || die "post-stage check failed: no compiled terminfo entry file at ${install_root}/share/limux/terminfo/{x/xterm-ghostty,g/ghostty} (hashed-db ncurses tic output? the runtime requires the directory-tree form)"
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
if [[ "$promote_default_aliases" == "true" ]]; then
    install_symlink "${install_root}/bin/${launcher_name}" "${bin_link_dir}/limux"
    install_symlink "${install_root}/bin/${cli_launcher_name}" "${bin_link_dir}/limux-cli"
fi

if [[ "$desktop_entry" == "true" ]]; then
    run mkdir -p "$app_dir"
    archive_existing_path "${app_dir}/${desktop_file_name}" "desktop entry"
    run cp "${install_root}/share/applications/${desktop_file_name}" "${app_dir}/${desktop_file_name}"
fi

# Manifest value shapes are load-bearing: `Ghostty resources:` must stay a
# path or `not found` — origin/provenance NEVER goes on that line, so the
# historical audit grep from docs/terminal-input-regression-20260701.md
# ("a valid install must not say `Ghostty resources: .../ghostty/src`")
# keeps working. For the vendored-assembly path the value is the STAGED
# bundle, which is what the runtime actually consumes.
if [[ -n "$ghostty_share_src" ]]; then
    manifest_ghostty_resources="$ghostty_share_src"
elif [[ -d "$vendored_shell_integration_src" ]]; then
    manifest_ghostty_resources="${install_root}/share/limux/ghostty"
else
    manifest_ghostty_resources="not found"
fi
case "$terminfo_plan" in
    copy) manifest_ghostty_terminfo="$ghostty_terminfo_src" ;;
    tic)  manifest_ghostty_terminfo="${install_root}/share/limux/terminfo" ;;
    *)    manifest_ghostty_terminfo="not found" ;;
esac
# Field value is exactly `valid` or `DEGRADED`; the escape-hatch marker
# line below is separate so both contracts stay grep-able.
manifest_ghostty_degraded_marker=""
if [[ "$ghostty_resource_shape" != "valid" ]]; then
    manifest_ghostty_degraded_marker=$'\n- DEGRADED: no ghostty resources'
fi

manifest="$(
    cat <<EOF_MANIFEST
# Limux User-Local Install Manifest

Mode: ${mode}
Timestamp UTC: ${timestamp}
Repo: ${repo_root}
Version: ${cargo_version}
Install ID: ${install_id}
Runtime channel: ${runtime_channel}
Runtime kind: ${channel_kind}
Preview channel id: ${channel_id:-n/a}
Install root: ${install_root}
Profile: ${profile}
Desktop entry: ${desktop_entry}
Launcher: ${bin_link_dir}/${launcher_name}
CLI launcher: ${bin_link_dir}/${cli_launcher_name}
Default aliases promoted: ${promote_default_aliases}
Reviewed installs retained per lane: ${keep_reviewed}

## Source Artifacts

- CLI: ${cli_src}
- Host: ${host_src}
- Ghostty library: ${ghostty_lib_src}
- Ghostty resources: ${manifest_ghostty_resources}
- Ghostty terminfo: ${manifest_ghostty_terminfo}
- Ghostty resource shape: ${ghostty_resource_shape}
- Ghostty resource origin: ${ghostty_resource_origin}${manifest_ghostty_degraded_marker}

## User Links

- ${bin_link_dir}/${launcher_name} -> ${install_root}/bin/${launcher_name}
- ${bin_link_dir}/${cli_launcher_name} -> ${install_root}/bin/${cli_launcher_name}
$(if [[ "$promote_default_aliases" == "true" ]]; then
    printf '%s\n' \
        "- ${bin_link_dir}/limux -> ${install_root}/bin/${launcher_name}" \
        "- ${bin_link_dir}/limux-cli -> ${install_root}/bin/${cli_launcher_name}"
fi)

## Archive Directory For Replaced Links

${archive_dir}

## Safety Boundary

- No sudo.
- No package manager.
- No cargo/zig build step (Ghostty terminfo may be tic-compiled from the vendored snapshot).
- No /etc writes.
- Existing link/file targets are moved into the archive directory, not deleted.
- Browser/WebKit use remains gated separately.
EOF_MANIFEST
)"

write_file "${install_root}/MANIFEST.md" "${manifest}"$'\n'

install_info="$(
    cat <<EOF_INSTALL_INFO
{
  "version": "$(json_escape "$cargo_version")",
  "install_id": "$(json_escape "$install_id")",
  "channel": "$(json_escape "$runtime_channel")",
  "profile": "$(json_escape "$profile")",
  "source_sha": "$(json_escape "$source_sha")",
  "created_utc": "$(json_escape "$timestamp")"
}
EOF_INSTALL_INFO
)"
write_file "${install_root}/install-info.json" "${install_info}"$'\n'

if [[ "$mode" == "apply" ]]; then
    (
        cd "$install_root"
        {
            printf '%s\n' \
                "bin/${launcher_name}" \
                "bin/${cli_launcher_name}" \
                libexec/limux-cli \
                libexec/limux-host \
                lib/libghostty.so \
                MANIFEST.md
            # PRD-A stamps install-info.json at the install root; keep it
            # covered whenever present (exists-check so either PR merge
            # order works).
            if [[ -f install-info.json ]]; then
                printf 'install-info.json\n'
            fi
            # Shipped Ghostty resources (shell-integration + terminfo) are a
            # variable file set — enumerate them so SHA256SUMS covers all.
            if [[ -d share/limux ]]; then
                find share/limux -type f | LC_ALL=C sort
            fi
        } | xargs -d '\n' sha256sum > SHA256SUMS
    )
fi

retention_args=(
    "--${mode}"
    "--reviewed-root" "${prefix}/limux-reviewed"
    "--keep" "$keep_reviewed"
    "--current-install-root" "$install_root"
    "--current-created-utc" "$timestamp"
    "--timestamp" "$timestamp"
)
retention_output="$(
    bash "${script_dir}/prune-reviewed-runtimes.sh" "${retention_args[@]}"
)"

if [[ -n "$manifest_out" ]]; then
    printf '%s\n' "$manifest" > "$manifest_out"
fi

log "Limux user-local install lane (${mode})"
log ""
log "$manifest"
log ""
log "Reviewed runtime retention:"
log "$retention_output"
log ""
log "Planned actions:"
for action in "${planned_actions[@]}"; do
    log "- ${action}"
done

if [[ "$mode" == "dry-run" ]]; then
    log ""
    log "Dry-run only. Re-run with --apply to install."
fi
