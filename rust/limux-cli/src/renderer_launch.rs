use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(crate) const RENDERER_ENV_KEYS: &[&str] = &[
    "GSK_RENDERER",
    "GDK_DEBUG",
    "GDK_DISABLE",
    "GALLIUM_DRIVER",
    "LIBGL_ALWAYS_SOFTWARE",
    "LP_NUM_THREADS",
    "MESA_GL_VERSION_OVERRIDE",
    "MESA_LOADER_DRIVER_OVERRIDE",
    "MESA_D3D12_DEFAULT_ADAPTER_NAME",
];

const WSL_D3D12_ENVIRONMENT: &[(&str, &str)] =
    &[("GSK_RENDERER", "gl"), ("GALLIUM_DRIVER", "d3d12")];
pub(crate) const AUTO_INJECTED_RENDERER_ENV: &str = "LIMUX_AUTO_RENDERER_ENV";
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_PROBE_STDOUT_BYTES: usize = 262_144;
type ProbeCapture = JoinHandle<std::io::Result<(Vec<u8>, bool)>>;

#[derive(Debug)]
struct ProbeExecution {
    payload: Option<Value>,
    #[cfg_attr(not(test), allow(dead_code))]
    child_pid: u32,
}

struct ProbeChild {
    child: Child,
    capture: Option<ProbeCapture>,
    reaped: bool,
}

impl ProbeChild {
    fn id(&self) -> u32 {
        self.child.id()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }

    fn terminate_and_reap(&mut self) -> std::io::Result<()> {
        // A failed kill commonly means the child exited between try_wait and
        // kill. Waiting is still the authoritative reap operation.
        let _ = self.child.kill();
        self.child.wait()?;
        self.reaped = true;
        Ok(())
    }

    fn join_capture(&mut self) -> std::io::Result<(Vec<u8>, bool)> {
        self.capture
            .take()
            .expect("renderer probe capture handle must be available")
            .join()
            .map_err(|_| std::io::Error::other("renderer probe capture thread panicked"))?
    }
}

impl Drop for ProbeChild {
    fn drop(&mut self) {
        // Child does not kill or reap on Drop. This guard owns both duties on
        // every post-spawn error path, then joins the pipe reader after EOF.
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(capture) = self.capture.take() {
            let _ = capture.join();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RendererLaunchContext {
    pub(crate) is_wsl: bool,
    pub(crate) dxg_available: bool,
    pub(crate) child_env_removal_supported: bool,
    pub(crate) explicit_renderer_environment: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererLaunchPlan {
    PreserveInherited,
    ProbeWslD3d12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererSelection {
    PreserveInherited,
    WslD3d12,
}

impl RendererSelection {
    pub(crate) fn environment(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::PreserveInherited => &[],
            Self::WslD3d12 => WSL_D3D12_ENVIRONMENT,
        }
    }
}

pub(crate) fn launch_plan(context: RendererLaunchContext) -> RendererLaunchPlan {
    if context.explicit_renderer_environment
        || !context.is_wsl
        || !context.dxg_available
        || !context.child_env_removal_supported
    {
        RendererLaunchPlan::PreserveInherited
    } else {
        RendererLaunchPlan::ProbeWslD3d12
    }
}

pub(crate) fn accepts_wsl_d3d12_probe(payload: &Value) -> bool {
    payload.get("status").and_then(Value::as_str) == Some("captured")
        && matches!(
            payload.get("selected_renderer").and_then(Value::as_str),
            Some("GskGLRenderer" | "GskNglRenderer")
        )
        && payload.get("is_software_fallback").and_then(Value::as_bool) == Some(false)
        && payload
            .pointer("/requested_policy/gallium_driver")
            .and_then(Value::as_str)
            == Some("d3d12")
        && payload
            .pointer("/gpu_device_usage/dxg_open")
            .and_then(Value::as_bool)
            == Some(true)
        && payload
            .pointer("/probe_surface/realized")
            .and_then(Value::as_bool)
            == Some(true)
        && payload
            .pointer("/probe_surface/width_px")
            .and_then(Value::as_i64)
            .is_some_and(|width| width > 0)
        && payload
            .pointer("/probe_surface/height_px")
            .and_then(Value::as_i64)
            .is_some_and(|height| height > 0)
}

pub(crate) fn selection_from_probe(payload: Option<&Value>) -> RendererSelection {
    if payload.is_some_and(accepts_wsl_d3d12_probe) {
        RendererSelection::WslD3d12
    } else {
        RendererSelection::PreserveInherited
    }
}

pub(crate) fn wsl_d3d12_probe_command(host: &Path) -> Command {
    let mut command = Command::new(host);
    command.arg("--renderer-probe");
    for key in super::HOST_LAUNCH_TARGET_ENV_REMOVALS
        .iter()
        .chain(super::HOST_LAUNCH_SOCKET_ENV_REMOVALS.iter())
        .chain(super::HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
    {
        command.env_remove(key);
    }
    for key in RENDERER_ENV_KEYS {
        command.env_remove(key);
    }
    for (key, value) in WSL_D3D12_ENVIRONMENT {
        command.env(key, value);
    }
    command.env("LIMUX_HOST_LOG", "off");
    command
}

fn run_probe_command(mut command: Command, timeout: Duration) -> std::io::Result<ProbeExecution> {
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut process = ProbeChild {
        child,
        capture: None,
        reaped: false,
    };
    let mut stdout = process
        .child
        .stdout
        .take()
        .expect("piped renderer probe stdout must be available");
    process.capture = Some(
        std::thread::Builder::new()
            .name("limux-renderer-probe-reader".to_string())
            .spawn(move || {
                let mut retained = Vec::new();
                let mut overflow = false;
                let mut chunk = [0_u8; 8192];
                loop {
                    let count = match stdout.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(count) => count,
                        Err(error) => return Err(error),
                    };
                    let remaining = MAX_PROBE_STDOUT_BYTES.saturating_sub(retained.len());
                    retained.extend_from_slice(&chunk[..count.min(remaining)]);
                    overflow |= count > remaining;
                }
                Ok((retained, overflow))
            })?,
    );
    let child_pid = process.id();

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = process.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            process.terminate_and_reap()?;
            process.join_capture()?;
            return Ok(ProbeExecution {
                payload: None,
                child_pid,
            });
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let (stdout, overflow) = process.join_capture()?;
    if !status.success() || overflow {
        return Ok(ProbeExecution {
            payload: None,
            child_pid,
        });
    }
    Ok(ProbeExecution {
        payload: serde_json::from_slice(&stdout).ok(),
        child_pid,
    })
}

pub(crate) fn run_wsl_d3d12_probe(host: &Path) -> std::io::Result<Option<Value>> {
    run_probe_command(wsl_d3d12_probe_command(host), PROBE_TIMEOUT)
        .map(|execution| execution.payload)
}

pub(crate) fn apply_selection(command: &mut Command, selection: RendererSelection) {
    if selection == RendererSelection::PreserveInherited {
        return;
    }
    let environment = selection.environment();
    for (key, value) in environment {
        command.env(key, value);
    }
    command.env(
        AUTO_INJECTED_RENDERER_ENV,
        environment
            .iter()
            .map(|(key, _)| *key)
            .collect::<Vec<_>>()
            .join(","),
    );
}

pub(crate) fn explicit_renderer_environment() -> bool {
    RENDERER_ENV_KEYS
        .iter()
        .any(|key| std::env::var_os(key).is_some())
}

pub(crate) fn is_wsl() -> bool {
    std::env::var_os("WSL_INTEROP").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .is_some_and(|release| release.to_ascii_lowercase().contains("microsoft"))
}

/// The vendored embedded Ghostty C API currently supports only environment
/// overlays; it cannot remove an inherited key from a terminal child. Keep the
/// automatic renderer policy fail-closed until the upstream capability lands.
pub(crate) fn child_env_removal_supported() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn explicit_renderer_environment_is_never_overridden() {
        let plan = launch_plan(RendererLaunchContext {
            is_wsl: true,
            dxg_available: true,
            child_env_removal_supported: true,
            explicit_renderer_environment: true,
        });

        assert_eq!(plan, RendererLaunchPlan::PreserveInherited);
    }

    #[test]
    fn d3d12_probe_requires_wsl_dxg_and_child_env_removal() {
        let eligible = RendererLaunchContext {
            is_wsl: true,
            dxg_available: true,
            child_env_removal_supported: true,
            explicit_renderer_environment: false,
        };
        assert_eq!(launch_plan(eligible), RendererLaunchPlan::ProbeWslD3d12);

        for ineligible in [
            RendererLaunchContext {
                is_wsl: false,
                ..eligible
            },
            RendererLaunchContext {
                dxg_available: false,
                ..eligible
            },
            RendererLaunchContext {
                child_env_removal_supported: false,
                ..eligible
            },
        ] {
            assert_eq!(
                launch_plan(ineligible),
                RendererLaunchPlan::PreserveInherited
            );
        }
    }

    #[test]
    fn d3d12_probe_accepts_only_real_dxg_acceleration() {
        let accepted = json!({
            "status": "captured",
            "selected_renderer": "GskGLRenderer",
            "is_software_fallback": false,
            "fallback_indicators": [],
            "requested_policy": { "gallium_driver": "d3d12" },
            "gpu_device_usage": { "dxg_open": true },
            "probe_surface": { "realized": true, "width_px": 64, "height_px": 64 }
        });
        assert!(accepts_wsl_d3d12_probe(&accepted));

        for rejected in [
            json!({
                "status": "captured",
                "selected_renderer": "GskGLRenderer",
                "is_software_fallback": true,
                "requested_policy": { "gallium_driver": "d3d12" },
                "gpu_device_usage": { "dxg_open": false },
                "probe_surface": { "realized": true, "width_px": 64, "height_px": 64 }
            }),
            json!({
                "status": "captured",
                "selected_renderer": "GskCairoRenderer",
                "is_software_fallback": false,
                "requested_policy": { "gallium_driver": "d3d12" },
                "gpu_device_usage": { "dxg_open": true },
                "probe_surface": { "realized": true, "width_px": 64, "height_px": 64 }
            }),
            json!({
                "status": "captured",
                "selected_renderer": "GskGLRenderer",
                "is_software_fallback": false,
                "requested_policy": { "gallium_driver": "d3d12" },
                "gpu_device_usage": { "dxg_open": true },
                "probe_surface": { "realized": true, "width_px": 0, "height_px": 0 }
            }),
            json!({ "status": "error" }),
        ] {
            assert!(!accepts_wsl_d3d12_probe(&rejected));
        }
    }

    #[test]
    fn rejected_probe_falls_back_to_inherited_renderer() {
        let selection = selection_from_probe(Some(&json!({
            "status": "captured",
            "selected_renderer": "GskGLRenderer",
            "is_software_fallback": true,
            "requested_policy": { "gallium_driver": "d3d12" },
            "gpu_device_usage": { "dxg_open": false },
            "probe_surface": { "realized": true, "width_px": 64, "height_px": 64 }
        })));

        assert_eq!(selection, RendererSelection::PreserveInherited);
    }

    #[test]
    fn accepted_probe_selects_only_the_d3d12_environment() {
        let selection = selection_from_probe(Some(&json!({
            "status": "captured",
            "selected_renderer": "GskGLRenderer",
            "is_software_fallback": false,
            "requested_policy": { "gallium_driver": "d3d12" },
            "gpu_device_usage": { "dxg_open": true },
            "probe_surface": { "realized": true, "width_px": 64, "height_px": 64 }
        })));

        assert_eq!(selection, RendererSelection::WslD3d12);
        assert_eq!(
            selection.environment(),
            &[("GSK_RENDERER", "gl"), ("GALLIUM_DRIVER", "d3d12")]
        );
    }

    #[test]
    fn probe_command_has_no_user_session_and_resets_renderer_controls() {
        let command = wsl_d3d12_probe_command(Path::new("/owned/limux-host"));
        let args = command
            .get_args()
            .map(|argument| argument.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(args, ["--renderer-probe"]);

        let env_of = |key: &str| {
            command.get_envs().find_map(|(candidate, value)| {
                (candidate == key).then(|| value.map(|value| value.to_string_lossy().to_string()))
            })
        };
        assert_eq!(env_of("GSK_RENDERER"), Some(Some("gl".to_string())));
        assert_eq!(env_of("GALLIUM_DRIVER"), Some(Some("d3d12".to_string())));
        assert_eq!(env_of("LIMUX_HOST_LOG"), Some(Some("off".to_string())));
        for key in super::super::HOST_LAUNCH_TARGET_ENV_REMOVALS
            .iter()
            .chain(super::super::HOST_LAUNCH_SOCKET_ENV_REMOVALS.iter())
            .chain(super::super::HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
        {
            assert_eq!(env_of(key), Some(None), "{key} must not reach the probe");
        }
    }

    #[test]
    fn selected_renderer_is_tagged_for_future_child_env_removal() {
        let mut command = std::process::Command::new("/owned/limux-host");
        apply_selection(&mut command, RendererSelection::WslD3d12);

        let marker = command
            .get_envs()
            .find_map(|(key, value)| {
                (key == AUTO_INJECTED_RENDERER_ENV)
                    .then(|| value.map(|value| value.to_string_lossy().to_string()))
            })
            .flatten();
        assert_eq!(marker.as_deref(), Some("GSK_RENDERER,GALLIUM_DRIVER"));
    }

    #[test]
    fn subprocess_runner_accepts_json_and_reaps_the_child() {
        let payload = json!({"status": "captured"});
        let mut command = Command::new("/usr/bin/printf");
        command.args(["%s", &payload.to_string()]);

        let execution = run_probe_command(command, Duration::from_secs(1))
            .expect("successful probe subprocess");

        assert_eq!(execution.payload, Some(payload));
        assert!(!Path::new(&format!("/proc/{}", execution.child_pid)).exists());
    }

    #[test]
    fn subprocess_runner_rejects_and_reaps_malformed_nonzero_and_oversized_children() {
        let mut malformed = Command::new("/usr/bin/printf");
        malformed.args(["%s", "{"]);

        let nonzero = Command::new("/usr/bin/false");

        let mut oversized = Command::new("/usr/bin/head");
        oversized.args(["-c", &(MAX_PROBE_STDOUT_BYTES + 1).to_string(), "/dev/zero"]);

        for command in [malformed, nonzero, oversized] {
            let execution = run_probe_command(command, Duration::from_secs(1))
                .expect("rejected probe subprocess");
            assert_eq!(execution.payload, None);
            assert!(!Path::new(&format!("/proc/{}", execution.child_pid)).exists());
        }
    }

    #[test]
    fn subprocess_runner_times_out_kills_and_reaps_the_child() {
        let mut command = Command::new("/usr/bin/sleep");
        command.arg("30");

        let execution = run_probe_command(command, Duration::from_millis(25))
            .expect("timed-out probe subprocess");

        assert_eq!(execution.payload, None);
        assert!(!Path::new(&format!("/proc/{}", execution.child_pid)).exists());
    }
}
