#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

fail() {
    printf 'runtime-isolation-smoke: ERROR: %s\n' "$*" >&2
    exit 1
}

profile="${LIMUX_SMOKE_PROFILE:-debug}"
case "$profile" in
    debug)
        cargo_flags=()
        ;;
    release)
        cargo_flags=(--release)
        ;;
    *)
        fail "LIMUX_SMOKE_PROFILE must be debug or release, got: ${profile}"
        ;;
esac

run_id="$(date -u +%Y%m%dT%H%M%S)-$$"
prefix="${LIMUX_RUNTIME_ISOLATION_SMOKE_PREFIX:-/tmp/limux-runtime-isolation-smoke-${USER:-user}-${run_id}}"
inherited_socket="/tmp/limux-runtime-isolation-inherited-stable.sock"

printf 'runtime-isolation-smoke: building limux-cli (%s)\n' "$profile"
cargo build "${cargo_flags[@]}" -p limux-cli --bin limux-cli

printf 'runtime-isolation-smoke: building limux host (%s)\n' "$profile"
cargo build "${cargo_flags[@]}" -p limux-host-linux --bin limux

[[ -f ghostty/zig-out/lib/libghostty.so ]] \
    || fail "missing Ghostty library: ghostty/zig-out/lib/libghostty.so"

installer="scripts/user-local-install/install-user-local.sh"

printf 'runtime-isolation-smoke: installing legacy lane into %s\n' "$prefix"
"$installer" --apply --profile "$profile" --prefix "$prefix" --install-id legacy-smoke --channel legacy --no-desktop-entry >/dev/null

printf 'runtime-isolation-smoke: installing stable lane into %s\n' "$prefix"
"$installer" --apply --profile "$profile" --prefix "$prefix" --install-id stable-smoke --channel stable --no-desktop-entry >/dev/null

printf 'runtime-isolation-smoke: installing preview lane into %s\n' "$prefix"
"$installer" --apply --profile "$profile" --prefix "$prefix" --install-id preview-smoke --channel preview --no-desktop-entry >/dev/null

for launcher in limux limux-cli limux-stable limux-stable-cli limux-preview limux-preview-cli; do
    [[ -L "${prefix}/bin/${launcher}" ]] || fail "missing launcher symlink: ${prefix}/bin/${launcher}"
done

legacy_target="$(readlink "${prefix}/bin/limux")"
stable_target="$(readlink "${prefix}/bin/limux-stable")"
preview_target="$(readlink "${prefix}/bin/limux-preview")"

[[ "$legacy_target" == *"/limux-reviewed/legacy-smoke/bin/limux" ]] \
    || fail "legacy launcher target is not legacy install: ${legacy_target}"
[[ "$stable_target" == *"/limux-reviewed/stable/stable-smoke/bin/limux-stable" ]] \
    || fail "stable launcher target is not stable install: ${stable_target}"
[[ "$preview_target" == *"/limux-reviewed/preview/default/preview-smoke/bin/limux-preview" ]] \
    || fail "preview launcher target is not preview install: ${preview_target}"

preview_info="$(LIMUX_SOCKET="$inherited_socket" "${prefix}/bin/limux-preview" target-info)"
stable_info="$("${prefix}/bin/limux-stable" target-info)"
legacy_info="$("${prefix}/bin/limux" target-info)"

[[ "$preview_info" == *"explicit_channel=preview:default"* ]] \
    || fail "preview target-info did not report preview channel: ${preview_info}"
[[ "$preview_info" == *"preview/default/limux.sock"* || "$preview_info" == *"limux-preview-default.sock"* ]] \
    || fail "preview target-info did not resolve a preview socket: ${preview_info}"
[[ "$preview_info" != *"$inherited_socket"* ]] \
    || fail "preview target-info used inherited stable socket: ${preview_info}"
[[ "$preview_info" == *"connects=false"* ]] \
    || fail "preview target-info unexpectedly connects: ${preview_info}"

[[ "$stable_info" == *"explicit_channel=stable"* ]] \
    || fail "stable target-info did not report stable channel: ${stable_info}"
[[ "$legacy_info" == *"explicit_channel=none"* ]] \
    || fail "legacy target-info should not report explicit channel: ${legacy_info}"

printf 'runtime-isolation-smoke: PASS\n'
printf 'runtime-isolation-smoke: retained prefix %s\n' "$prefix"
