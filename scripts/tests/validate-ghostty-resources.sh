#!/usr/bin/env bash
# Validate Ghostty resource + terminfo packaging (PRD-B, roadmap W0.2).
#
# Asserts the invariants behind the 2026-06-29 terminal-input regression
# (docs/terminal-input-regression-20260701.md):
#   - the vendored terminfo snapshot stays in sync with the ghostty submodule
#   - tic compiles the snapshot into real entry FILES (x/xterm-ghostty)
#   - a user-local install stages the exact runtime resource shape
#     (is_ghostty_resources_dir contract, rust/limux-host-linux/src/main.rs)
#   - manifest carries the shape/origin fields and never the ghostty/src shape
#   - source-only resource shapes are rejected; the escape hatch stamps DEGRADED
#
# Requires: python3, tic, infocmp, built workspace artifacts
# (target/<profile>/limux{,-cli}) and ghostty/zig-out/lib/libghostty.so.
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root_dir"

profile="${LIMUX_VALIDATE_PROFILE:-debug}"
installer="scripts/user-local-install/install-user-local.sh"
generator="scripts/user-local-install/generate-ghostty-terminfo.py"
snapshot="scripts/user-local-install/resources/ghostty.terminfo"

fail() {
    printf 'validate-ghostty-resources: FAIL: %s\n' "$*" >&2
    exit 1
}

pass() {
    printf 'validate-ghostty-resources: ok: %s\n' "$*"
}

for tool in python3 tic infocmp; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool missing: ${tool}"
done
[[ -f "$snapshot" ]] || fail "vendored snapshot missing: ${snapshot}"
[[ -x "target/${profile}/limux" && -x "target/${profile}/limux-cli" ]] \
    || fail "built artifacts missing under target/${profile}; run cargo build (or ./scripts/check.sh) first"
[[ -f "ghostty/zig-out/lib/libghostty.so" ]] \
    || fail "ghostty/zig-out/lib/libghostty.so missing; build libghostty first (see .github/workflows/rust-quality.yml)"

# Scratch space is intentionally left in place after the run (repo policy:
# no destructive cleanup); mktemp under $TMPDIR is reclaimed by the OS/runner.
scratch="$(mktemp -d -t limux-validate-ghostty.XXXXXX)"

# --- 1. Vendored snapshot stays in sync with the submodule -------------------
if [[ -f "ghostty/src/terminfo/ghostty.zig" ]]; then
    python3 "$generator" --check "$snapshot" \
        || fail "vendored snapshot out of sync with ghostty/src/terminfo/ghostty.zig"
    pass "snapshot in sync with submodule definition"
else
    pass "ghostty submodule not initialized; skipping snapshot sync check"
fi

# --- 2. tic compiles the snapshot into entry FILES ---------------------------
tic_out="${scratch}/terminfo"
mkdir -p "$tic_out"
tic -x -o "$tic_out" "$snapshot" || fail "tic rejected the vendored snapshot"
[[ -f "${tic_out}/x/xterm-ghostty" ]] \
    || fail "tic did not produce ${tic_out}/x/xterm-ghostty as a regular file (hashed-db ncurses?)"
infocmp -A "$tic_out" xterm-ghostty >/dev/null || fail "infocmp cannot read compiled xterm-ghostty"
pass "tic round-trip produced x/xterm-ghostty and infocmp reads it"

# --- 3. Full install stages the runtime resource shape -----------------------
prefix="${scratch}/prefix"
install_root="${prefix}/limux-reviewed/preview/default/prd-b-validate"
"$installer" --apply --channel preview --profile "$profile" \
    --install-id prd-b-validate --prefix "$prefix" \
    > "${scratch}/apply.log" 2>&1 \
    || { cat "${scratch}/apply.log" >&2; fail "installer --apply failed"; }

[[ -d "${install_root}/share/limux/ghostty/shell-integration" ]] \
    || fail "installed root missing share/limux/ghostty/shell-integration/"
[[ -f "${install_root}/share/limux/terminfo/x/xterm-ghostty" || -f "${install_root}/share/limux/terminfo/g/ghostty" ]] \
    || fail "installed root missing compiled terminfo entry files under share/limux/terminfo/"
infocmp -A "${install_root}/share/limux/terminfo" xterm-ghostty >/dev/null \
    || fail "infocmp cannot resolve xterm-ghostty from the installed terminfo dir"

manifest="${install_root}/MANIFEST.md"
grep -q '^- Ghostty resource shape: valid$' "$manifest" \
    || fail "manifest missing 'Ghostty resource shape: valid'"
grep -q '^- Ghostty resource origin: ' "$manifest" \
    || fail "manifest missing 'Ghostty resource origin:' field"
# Historical audit invariant (docs/terminal-input-regression-20260701.md):
# a valid install must never record ghostty/src as its resources dir.
if grep -E '^- Ghostty resources: .*ghostty/src(/.*)?$' "$manifest"; then
    fail "manifest records a ghostty/src resource path — regression shape"
fi
grep -q 'share/limux/terminfo/' "${install_root}/SHA256SUMS" \
    || fail "SHA256SUMS does not cover shipped terminfo files"
grep -q 'share/limux/ghostty/shell-integration/' "${install_root}/SHA256SUMS" \
    || fail "SHA256SUMS does not cover shipped shell-integration files"
if [[ -f "${install_root}/install-info.json" ]]; then
    grep -q 'install-info.json' "${install_root}/SHA256SUMS" \
        || fail "SHA256SUMS does not cover install-info.json (PRD-A coverage regression)"
fi
(cd "$install_root" && sha256sum --check --quiet SHA256SUMS) \
    || fail "sha256sum -c failed on the installed tree"
pass "apply stages valid shape; manifest + SHA256SUMS invariants hold"

# --- 4. Regression: source-only shapes are rejected --------------------------
fixtures="${scratch}/fixtures"
mkdir -p "${fixtures}/src-tree/shell-integration" "${fixtures}/src-tree/terminfo" \
    "${fixtures}/src-only/shell-integration" "${fixtures}/empty-terminfo"
printf '// zig source marker\n' > "${fixtures}/src-tree/terminfo/ghostty.zig"
printf '# stub\n' > "${fixtures}/src-only/shell-integration/bash"

if "$installer" --dry-run --profile "$profile" \
    --ghostty-share "${fixtures}/src-tree" > "${scratch}/src-tree.log" 2>&1; then
    fail "installer accepted a ghostty/src-style SOURCE tree via --ghostty-share"
fi
grep -qi 'source tree' "${scratch}/src-tree.log" \
    || fail "source-tree rejection did not explain itself: $(cat "${scratch}/src-tree.log")"
pass "ghostty/src-style source tree rejected"

if "$installer" --dry-run --profile "$profile" \
    --ghostty-share "${fixtures}/src-only" \
    --ghostty-terminfo "${fixtures}/empty-terminfo" > "${scratch}/src-only.log" 2>&1; then
    fail "installer accepted a source-only shape (shell-integration without compiled terminfo)"
fi
grep -qi 'source-only' "${scratch}/src-only.log" \
    || fail "source-only rejection did not explain itself: $(cat "${scratch}/src-only.log")"
pass "source-only shape (no compiled terminfo) rejected"

# --- 5. Escape hatch stamps DEGRADED ------------------------------------------
"$installer" --dry-run --profile "$profile" \
    --ghostty-share "${fixtures}/src-only" \
    --ghostty-terminfo "${fixtures}/empty-terminfo" \
    --allow-missing-ghostty-resources \
    --manifest-out "${scratch}/degraded-manifest.md" >/dev/null 2>&1 \
    || fail "--allow-missing-ghostty-resources did not allow the degraded install"
grep -q '^- Ghostty resource shape: DEGRADED$' "${scratch}/degraded-manifest.md" \
    || fail "degraded manifest missing 'Ghostty resource shape: DEGRADED'"
grep -q '^- DEGRADED: no ghostty resources$' "${scratch}/degraded-manifest.md" \
    || fail "degraded manifest missing the 'DEGRADED: no ghostty resources' marker line"
pass "escape hatch stamps DEGRADED shape + marker"

printf 'validate-ghostty-resources: all checks passed\n'
