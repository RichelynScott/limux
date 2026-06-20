#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

fail() {
    printf 'validate-split-icons: ERROR: %s\n' "$*" >&2
    exit 1
}

command -v file >/dev/null 2>&1 || fail "file(1) is required"

split_icons=(
    "limux-split-horizontal-symbolic.svg"
    "limux-split-vertical-symbolic.svg"
    "hicolor/scalable/actions/limux-split-horizontal-symbolic.svg"
    "hicolor/scalable/actions/limux-split-vertical-symbolic.svg"
)

for rel in "${split_icons[@]}"; do
    path="rust/limux-host-linux/icons/${rel}"
    [[ -s "$path" ]] || fail "missing or empty split icon source: ${path}"

    mime="$(file -b --mime-type "$path" 2>/dev/null || true)"
    [[ "$mime" == "image/svg+xml" ]] || fail "split icon source is not SVG: ${path} (${mime:-unknown})"

    prefix="$(LC_ALL=C head -c 5 "$path")"
    case "$prefix" in
        "<svg "|"<?xml") ;;
        *) fail "split icon source does not start with SVG/XML bytes: ${path}" ;;
    esac

    LC_ALL=C grep -Eq '<svg([[:space:]>])' "$path" || fail "missing <svg> element: ${path}"
    LC_ALL=C grep -q '</svg>' "$path" || fail "missing closing </svg>: ${path}"
done

LC_ALL=C grep -q 'validate_split_icon_sources "$ICONS_DIR"' scripts/package.sh \
    || fail "scripts/package.sh does not validate split SVG icon sources before packaging"

LC_ALL=C grep -q 'validate_split_icon_sources "$icons_src"' scripts/user-local-install/install-user-local.sh \
    || fail "scripts/user-local-install/install-user-local.sh does not validate split SVG icon sources before copying icons"

printf 'split icon source and packaging validation checks passed\n'
