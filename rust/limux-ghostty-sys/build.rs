use std::path::PathBuf;

fn main() {
    // Find libghostty relative to the workspace root. CARGO_MANIFEST_DIR is
    // read at build-script RUNTIME, not via `env!` at compile time: the
    // compile-time form bakes the absolute checkout path into the build-script
    // binary, and with a shared cargo target dir a binary compiled in an
    // ephemeral worktree can later be selected from another checkout — it then
    // resolves a path that no longer exists and fails "libghostty not found"
    // while the library is present (observed 2026-07-29; see
    // docs/LIMUX_FASTFOLLOWS_2026-07-29.md item 4).
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("cargo sets CARGO_MANIFEST_DIR when running build scripts"),
    );
    let ghostty_root = manifest_dir.join("../../ghostty");
    let ghostty_lib = ghostty_root
        .join("zig-out/lib")
        .canonicalize()
        .expect("libghostty not found — run: cd ghostty && zig build -Dapp-runtime=none -Doptimize=ReleaseFast");

    println!("cargo:rustc-link-search=native={}", ghostty_lib.display());
    println!("cargo:rustc-link-lib=dylib=ghostty");
    println!("cargo:rustc-link-lib=dylib=epoxy");

    // Compile glad (GL loader) which libghostty depends on but doesn't
    // include when built as a shared library.
    let glad_src = ghostty_root.join("vendor/glad/src/gl.c");
    let glad_include = ghostty_root.join("vendor/glad/include");
    assert!(
        glad_src.is_file() && glad_include.is_dir(),
        "Ghostty GLAD sources not found — initialize the ghostty submodule before building"
    );
    cc::Build::new()
        .file(&glad_src)
        .include(&glad_include)
        .compile("glad");

    // Re-run if libghostty changes
    println!(
        "cargo:rerun-if-changed={}",
        ghostty_lib.join("libghostty.so").display()
    );
    println!("cargo:rerun-if-changed={}", glad_src.display());
    println!("cargo:rerun-if-changed={}", glad_include.display());
}
