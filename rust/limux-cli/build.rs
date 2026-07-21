use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn command_stdout(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// True when `git` here resolves to the SAME repository as the crate source.
///
/// If the source tree has been copied/extracted underneath an unrelated git
/// repo, `git` answers about that outer repo instead. Before `-uno` that
/// misattribution at least surfaced as `dirty=true` (the copied tree showed up
/// as untracked); with `-uno` it would report a confident, wrong "verified
/// clean" against a foreign HEAD. Fail to "unknown" instead of attesting a
/// provenance we cannot stand behind.
fn git_matches_source_tree(root: &Path) -> bool {
    let Some(toplevel) = command_stdout(&["rev-parse", "--show-toplevel"]).map(PathBuf::from)
    else {
        return false;
    };
    match (toplevel.canonicalize(), root.canonicalize()) {
        (Ok(toplevel), Ok(root)) => toplevel == root,
        _ => false,
    }
}

/// Reports whether *tracked* files are modified, for build provenance.
///
/// `-uno` keeps untracked files (peer-owned docs, scratch files, editor
/// droppings) from marking an otherwise-clean release build as dirty. Staged
/// changes to tracked files still count as dirty.
///
/// This deliberately does not reuse `command_stdout`, which folds empty output
/// into `None`: here an empty-but-successful result is the *verified clean*
/// signal and must stay distinguishable from "git failed, we cannot tell".
fn git_tracked_dirty(root: &Path) -> &'static str {
    if !git_matches_source_tree(root) {
        return "unknown";
    }
    let Ok(output) = Command::new("git")
        .args(["status", "--porcelain", "-uno"])
        .output()
    else {
        return "unknown";
    };
    if !output.status.success() {
        return "unknown";
    }
    let Ok(status) = String::from_utf8(output.stdout) else {
        return "unknown";
    };
    if status.trim().is_empty() {
        "false"
    } else {
        "true"
    }
}

fn emit_if_exists(path: impl AsRef<Path>) {
    let path = path.as_ref();
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn emit_rs_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    emit_if_exists(dir);
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            emit_rs_files(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            emit_if_exists(path);
        }
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

fn emit_git_inputs(root: &Path) {
    emit_if_exists(root.join(".git"));
    if let Some(git_dir) = command_stdout(&["rev-parse", "--git-dir"]).map(PathBuf::from) {
        emit_if_exists(&git_dir);
        emit_if_exists(git_dir.join("HEAD"));
    }
    if let Some(common_dir) = command_stdout(&["rev-parse", "--git-common-dir"]).map(PathBuf::from)
    {
        emit_if_exists(common_dir.join("HEAD"));
        emit_if_exists(common_dir.join("refs"));
        emit_if_exists(common_dir.join("packed-refs"));
    }
    if let Some(index) = command_stdout(&["rev-parse", "--git-path", "index"]).map(PathBuf::from) {
        emit_if_exists(index);
    }
}

fn main() {
    let root = workspace_root();
    println!("cargo:rerun-if-changed={}/Cargo.toml", root.display());
    println!("cargo:rerun-if-changed={}/Cargo.lock", root.display());
    emit_git_inputs(&root);

    if let Ok(crates) = fs::read_dir(root.join("rust")) {
        for entry in crates.flatten() {
            let path = entry.path();
            emit_if_exists(path.join("Cargo.toml"));
            emit_if_exists(path.join("build.rs"));
            emit_rs_files(&path.join("src"));
        }
    }

    let sha = command_stdout(&["rev-parse", "--short=12", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = git_tracked_dirty(&root);
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=LIMUX_BUILD_SHA={sha}");
    println!("cargo:rustc-env=LIMUX_BUILD_DIRTY={dirty}");
    println!("cargo:rustc-env=LIMUX_BUILD_PROFILE={profile}");
}
