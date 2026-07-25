//! Route coverage for installed launchers.
//!
//! Every unit test in this crate drives `parse_global_args_from` or the raw
//! binary. None of them go through an installed launcher — and that gap shipped
//! a feature that was unreachable for 100% of installed users (#92, reverted).
//!
//! Installed launchers pin the build lane:
//!
//! ```sh
//! exec "$INSTALL_ROOT/libexec/limux-cli" "--channel" "stable" "$@"
//! ```
//!
//! so an operator's `--profile work` arrives as `--channel stable --profile
//! work`. The reverted implementation treated that as contradictory user input
//! and refused. A mutation proof could not catch it: mutation testing perturbs
//! code the tests already execute, and no test entered this route.
//!
//! These tests invoke the REAL binary through a REAL launcher of the same shape
//! the installer writes. Test the route, not the parser.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Write a launcher byte-shaped like the one
/// `scripts/user-local-install/install-user-local.sh` generates.
fn write_launcher(dir: &Path, channel: &str) -> PathBuf {
    let launcher = dir.join(format!("limux-{}-cli", channel.replace(':', "-")));
    fs::write(
        &launcher,
        format!(
            "#!/usr/bin/env bash\n\
             set -euo pipefail\n\
             export LIMUX_CHANNEL=\"{channel}\"\n\
             exec \"{bin}\" \"--channel\" \"{channel}\" \"$@\"\n",
            channel = channel,
            bin = env!("CARGO_BIN_EXE_limux-cli"),
        ),
    )
    .expect("write launcher");
    fs::set_permissions(&launcher, PermissionsExt::from_mode(0o755)).expect("chmod launcher");
    launcher
}

fn run(launcher: &Path, args: &[&str], runtime_dir: &Path) -> (String, String, bool) {
    let out = Command::new(launcher)
        .args(args)
        .env("XDG_RUNTIME_DIR", runtime_dir)
        // A launcher-pinned lane must be the only lane in play; make sure no
        // inherited profile is silently supplying the answer.
        .env_remove("LIMUX_PROFILE_ID")
        .env_remove("LIMUX_SOCKET")
        .env_remove("LIMUX_SOCKET_PATH")
        .output()
        .expect("run launcher");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

/// THE regression test for the #92 revert. A launcher-pinned lane plus an
/// operator `--profile` must resolve, not conflict.
#[test]
fn launcher_pinned_lane_plus_user_profile_resolves() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let runtime = tmp.path().join("run");
    fs::create_dir_all(&runtime).expect("runtime dir");
    let launcher = write_launcher(tmp.path(), "stable");

    let (stdout, stderr, ok) = run(&launcher, &["--profile", "work", "target-info"], &runtime);

    assert!(
        ok,
        "launcher + --profile must succeed.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        !stderr.contains("conflicting"),
        "launcher-pinned lane must not read as a user conflict.\nstderr: {stderr}"
    );
    // The lane and the profile must BOTH appear in the resolved path.
    let expected = runtime.join("limux/stable/profiles/work/limux.sock");
    assert!(
        stdout.contains(&format!("resolved_socket={}", expected.display())),
        "expected lane+profile socket {}\ngot: {stdout}",
        expected.display()
    );
}

/// Same for a named preview lane, since that is how preview installs are
/// launched.
#[test]
fn preview_launcher_plus_user_profile_resolves() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let runtime = tmp.path().join("run");
    fs::create_dir_all(&runtime).expect("runtime dir");
    let launcher = write_launcher(tmp.path(), "preview:lab");

    let (stdout, stderr, ok) = run(&launcher, &["--profile", "work", "target-info"], &runtime);

    assert!(ok, "preview launcher + --profile must succeed.\n{stderr}");
    let expected = runtime.join("limux/preview/lab/profiles/work/limux.sock");
    assert!(
        stdout.contains(&format!("resolved_socket={}", expected.display())),
        "expected {}\ngot: {stdout}",
        expected.display()
    );
}

/// Two BUILDS must not share one profile's socket — the reason the cheap
/// "let --profile override the lane" fix was rejected.
#[test]
fn same_profile_through_different_launchers_does_not_collide() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let runtime = tmp.path().join("run");
    fs::create_dir_all(&runtime).expect("runtime dir");

    let stable = write_launcher(tmp.path(), "stable");
    let preview = write_launcher(tmp.path(), "preview:lab");

    let (a, _, _) = run(&stable, &["--profile", "work", "target-info"], &runtime);
    let (b, _, _) = run(&preview, &["--profile", "work", "target-info"], &runtime);

    let line = |s: &str| {
        s.lines()
            .find(|l| l.starts_with("resolved_socket="))
            .unwrap_or_default()
            .to_string()
    };
    assert_ne!(
        line(&a),
        line(&b),
        "profile `work` must not be shared across build lanes"
    );
}

/// A launcher with no `--profile` must behave exactly as before — this feature
/// must not move the default install's session.
#[test]
fn launcher_without_profile_is_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let runtime = tmp.path().join("run");
    fs::create_dir_all(&runtime).expect("runtime dir");
    let launcher = write_launcher(tmp.path(), "stable");

    let (stdout, _, ok) = run(&launcher, &["target-info"], &runtime);

    assert!(ok, "bare launcher must still work");
    let expected = runtime.join("limux/stable/limux.sock");
    assert!(
        stdout.contains(&format!("resolved_socket={}", expected.display())),
        "expected unchanged lane socket {}\ngot: {stdout}",
        expected.display()
    );
}

/// `profile list` must work through a launcher too, and must be lane-scoped.
#[test]
fn profile_list_through_launcher_is_lane_scoped() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let runtime = tmp.path().join("run");
    let data = tmp.path().join("data");
    fs::create_dir_all(&runtime).expect("runtime dir");
    // A profile that belongs to the STABLE lane only.
    fs::create_dir_all(data.join("limux/stable/profiles/work/session")).expect("stable profile");
    fs::write(
        data.join("limux/stable/profiles/work/session/session.json"),
        b"{}",
    )
    .expect("session");

    let stable = write_launcher(tmp.path(), "stable");
    let preview = write_launcher(tmp.path(), "preview:lab");

    let listed = |launcher: &Path| {
        let out = Command::new(launcher)
            .args(["profile", "list"])
            .env("XDG_RUNTIME_DIR", &runtime)
            .env("XDG_DATA_HOME", &data)
            .env_remove("LIMUX_PROFILE_ID")
            .output()
            .expect("run launcher");
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    assert!(
        listed(&stable).contains("work"),
        "stable lane must list its own profile"
    );
    assert!(
        !listed(&preview).contains("work"),
        "preview lane must NOT see the stable lane's profile"
    );
}
