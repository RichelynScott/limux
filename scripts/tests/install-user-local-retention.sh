#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
test_root="${TMPDIR:-/tmp}/limux-reviewed-retention-test-${$}-${RANDOM}"
fixture_repo="${test_root}/repo"
fixture_prefix="${test_root}/prefix"
fixture_script_dir="${fixture_repo}/scripts/user-local-install"

fail() {
    printf 'install-user-local-retention: FAIL: %s\n' "$*" >&2
    exit 1
}

assert_exists() {
    local path="$1"
    [[ -e "$path" || -L "$path" ]] || fail "expected path to exist: ${path}"
}

assert_absent() {
    local path="$1"
    [[ ! -e "$path" && ! -L "$path" ]] || fail "expected path to be absent: ${path}"
}

write_install_info() {
    local root="$1"
    local install_id="$2"
    local channel="$3"
    local created_utc="$4"

    mkdir -p "${root}/bin"
    printf '#!/usr/bin/env bash\nexit 0\n' > "${root}/bin/limux"
    chmod 755 "${root}/bin/limux"
    cat > "${root}/install-info.json" <<EOF_INSTALL_INFO
{
  "version": "0.0.0-test",
  "install_id": "${install_id}",
  "channel": "${channel}",
  "profile": "debug",
  "source_sha": "test",
  "created_utc": "${created_utc}"
}
EOF_INSTALL_INFO
}

mkdir -p \
    "$fixture_script_dir/resources" \
    "${fixture_repo}/target/debug" \
    "${fixture_repo}/ghostty/zig-out/lib" \
    "${fixture_repo}/ghostty/share/ghostty/shell-integration" \
    "${fixture_repo}/terminfo/g" \
    "${fixture_repo}/rust/limux-host-linux/icons/hicolor/scalable/actions"

cp "${repo_root}/scripts/user-local-install/install-user-local.sh" \
    "${fixture_script_dir}/install-user-local.sh"
cp "${repo_root}/scripts/user-local-install/prune-reviewed-runtimes.sh" \
    "${fixture_script_dir}/prune-reviewed-runtimes.sh"
cp "${repo_root}/scripts/user-local-install/resources/ghostty.terminfo" \
    "${fixture_script_dir}/resources/ghostty.terminfo"

cat > "${fixture_repo}/Cargo.toml" <<'EOF_CARGO'
[package]
name = "limux-retention-fixture"
version = "0.0.0"
EOF_CARGO

for binary in limux-cli limux; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "${fixture_repo}/target/debug/${binary}"
    chmod 755 "${fixture_repo}/target/debug/${binary}"
done
printf 'fixture ghostty library\n' > "${fixture_repo}/ghostty/zig-out/lib/libghostty.so"
printf 'fixture shell integration\n' \
    > "${fixture_repo}/ghostty/share/ghostty/shell-integration/bash-integration"
printf 'fixture compiled terminfo\n' > "${fixture_repo}/terminfo/g/ghostty"

cat > "${fixture_repo}/rust/limux-host-linux/dev.limux.linux.desktop" <<'EOF_DESKTOP'
[Desktop Entry]
Name=Limux
Exec=limux
TryExec=limux
Type=Application
EOF_DESKTOP

for icon in \
    limux-split-horizontal-symbolic.svg \
    limux-split-vertical-symbolic.svg
do
    printf '<svg xmlns="http://www.w3.org/2000/svg"></svg>\n' \
        > "${fixture_repo}/rust/limux-host-linux/icons/${icon}"
    printf '<svg xmlns="http://www.w3.org/2000/svg"></svg>\n' \
        > "${fixture_repo}/rust/limux-host-linux/icons/hicolor/scalable/actions/${icon}"
done

reviewed_root="${fixture_prefix}/limux-reviewed"
write_install_info "${reviewed_root}/stable/old-1" old-1 stable 20260701T000000Z
write_install_info "${reviewed_root}/stable/old-2" old-2 stable 20260702T000000Z
write_install_info "${reviewed_root}/stable/old-3" old-3 stable 20260703T000000Z

bash "${fixture_script_dir}/install-user-local.sh" \
    --apply \
    --profile debug \
    --prefix "$fixture_prefix" \
    --install-id current \
    --channel stable \
    --ghostty-share "${fixture_repo}/ghostty/share/ghostty" \
    --ghostty-terminfo "${fixture_repo}/terminfo"

assert_absent "${reviewed_root}/stable/old-1"
assert_exists "${reviewed_root}/stable/old-2"
assert_exists "${reviewed_root}/stable/old-3"
assert_exists "${reviewed_root}/stable/current"
assert_exists "${fixture_prefix}/bin/limux"
assert_exists "${fixture_prefix}/bin/limux-cli"

old_1_archive="$(
    find "${reviewed_root}/archive" \
        -path '*/reviewed-runtimes/stable/old-1' \
        -type d \
        -print \
        -quit
)"
[[ -n "$old_1_archive" ]] || fail "old-1 was not archived"

retention_manifest="$(
    find "${reviewed_root}/archive" \
        -path '*/reviewed-runtimes/MANIFEST.tsv' \
        -type f \
        -print \
        -quit
)"
[[ -n "$retention_manifest" ]] || fail "retention manifest was not written"
grep -F $'stable/old-1\t' "$retention_manifest" >/dev/null \
    || fail "retention manifest omitted stable/old-1"
global_manifest="${reviewed_root}/archive/MANIFEST.tsv"
assert_exists "$global_manifest"
grep -F $'stable/old-1\t' "$global_manifest" >/dev/null \
    || fail "global retention manifest omitted the first prune run"

for number in 1 2 3 4 5 6; do
    write_install_info \
        "${reviewed_root}/legacy-${number}" \
        "legacy-${number}" \
        legacy \
        "2026060${number}T000000Z"
done
mkdir -p "${fixture_prefix}/libexec"
ln -s \
    "${reviewed_root}/legacy-1/bin/limux" \
    "${fixture_prefix}/libexec/limux-retention-test-link"

mkdir -p "${reviewed_root}/legacy-2/libexec"
cp "$(command -v sleep)" "${reviewed_root}/legacy-2/libexec/limux-host"
"${reviewed_root}/legacy-2/libexec/limux-host" 30 &
active_pid="$!"

for number in 1 2 3 4; do
    write_install_info \
        "${reviewed_root}/preview/blue/blue-${number}" \
        "blue-${number}" \
        preview:blue \
        "2026050${number}T000000Z"
done
for number in 1 2; do
    write_install_info \
        "${reviewed_root}/preview/red/red-${number}" \
        "red-${number}" \
        preview:red \
        "2026040${number}T000000Z"
done
write_install_info \
    "${reviewed_root}/stable/dry-run-old" \
    dry-run-old \
    stable \
    20260101T000000Z

dry_run_output="$(
    bash "${fixture_script_dir}/prune-reviewed-runtimes.sh" \
        --dry-run \
        --reviewed-root "$reviewed_root" \
        --keep 2 \
        --current-install-root "${reviewed_root}/stable/current" \
        --timestamp 20260729T120000Z
)"
assert_exists "${reviewed_root}/stable/dry-run-old"
grep -F 'WOULD_ARCHIVE lane=stable path=stable/dry-run-old' <<<"$dry_run_output" >/dev/null \
    || fail "dry-run did not list the excess stable install"
grep -F 'KEEP lane=legacy path=legacy-1' <<<"$dry_run_output" >/dev/null \
    || fail "dry-run did not protect the launcher-linked legacy install"
grep -F 'KEEP lane=legacy path=legacy-2' <<<"$dry_run_output" >/dev/null \
    || fail "dry-run did not protect the active-process legacy install"

apply_output="$(
    bash "${fixture_script_dir}/prune-reviewed-runtimes.sh" \
        --apply \
        --reviewed-root "$reviewed_root" \
        --keep 2 \
        --current-install-root "${reviewed_root}/stable/current" \
        --timestamp 20260729T120001Z
)"
kill "$active_pid"
wait "$active_pid" 2>/dev/null || true

for path in \
    "${reviewed_root}/legacy-1" \
    "${reviewed_root}/legacy-2" \
    "${reviewed_root}/legacy-5" \
    "${reviewed_root}/legacy-6" \
    "${reviewed_root}/preview/blue/blue-3" \
    "${reviewed_root}/preview/blue/blue-4" \
    "${reviewed_root}/preview/red/red-1" \
    "${reviewed_root}/preview/red/red-2"
do
    assert_exists "$path"
done
for path in \
    "${reviewed_root}/legacy-3" \
    "${reviewed_root}/legacy-4" \
    "${reviewed_root}/preview/blue/blue-1" \
    "${reviewed_root}/preview/blue/blue-2" \
    "${reviewed_root}/stable/dry-run-old"
do
    assert_absent "$path"
done
grep -F 'ARCHIVED lane=preview/blue path=preview/blue/blue-1' <<<"$apply_output" >/dev/null \
    || fail "apply output omitted the preview/blue archival"
grep -F 'KEEP lane=preview/red path=preview/red/red-1' <<<"$apply_output" >/dev/null \
    || fail "preview/red was not retained independently"
grep -F $'legacy-3\t' "$global_manifest" >/dev/null \
    || fail "global retention manifest did not append the second prune run"
grep -F $'preview/blue/blue-1\t' "$global_manifest" >/dev/null \
    || fail "global retention manifest omitted the preview archival"

printf 'install-user-local-retention: PASS\n'
