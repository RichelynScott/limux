use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;
use serde::Serialize;

const RENDERER_ENV_KEYS: [&str; 8] = [
    "GSK_RENDERER",
    "GDK_DISABLE",
    "GDK_DEBUG",
    "LIBGL_ALWAYS_SOFTWARE",
    "GALLIUM_DRIVER",
    "MESA_LOADER_DRIVER_OVERRIDE",
    "MESA_D3D12_DEFAULT_ADAPTER_NAME",
    "LP_NUM_THREADS",
];

static DIAGNOSTICS: OnceLock<RendererDiagnostics> = OnceLock::new();

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
struct RequestedRendererPolicy {
    gsk_renderer: Option<String>,
    gdk_disable: Option<String>,
    gdk_debug: Option<String>,
    libgl_always_software: Option<String>,
    gallium_driver: Option<String>,
    mesa_loader_driver_override: Option<String>,
    mesa_d3d12_default_adapter_name: Option<String>,
    lp_num_threads: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SoftwareFallbackEvidence {
    is_software_fallback: bool,
    indicators: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct GpuDeviceAvailability {
    dxg: bool,
    dri_render_node: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct GpuDeviceUsage {
    dxg_open: bool,
    dri_render_node_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct PreviewBackend {
    name: &'static str,
    environment: &'static [(&'static str, &'static str)],
    expected: &'static str,
    fallback: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct RendererDiagnostics {
    requested_policy: RequestedRendererPolicy,
    selected_renderer: Option<String>,
    is_software_fallback: bool,
    fallback_indicators: Vec<String>,
    gpu_devices: GpuDeviceAvailability,
    gpu_device_usage: GpuDeviceUsage,
    preview_backend_matrix: Vec<PreviewBackend>,
    preview_fallback_chain: Vec<&'static str>,
}

impl RendererDiagnostics {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|err| {
            serde_json::json!({
                "status": "error",
                "message": format!("renderer diagnostics serialization failed: {err}"),
            })
        })
    }
}

fn requested_policy_from_pairs<I, K, V>(pairs: I) -> RequestedRendererPolicy
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut policy = RequestedRendererPolicy::default();
    for (key, value) in pairs {
        let value = Some(value.as_ref().to_string());
        match key.as_ref() {
            "GSK_RENDERER" => policy.gsk_renderer = value,
            "GDK_DISABLE" => policy.gdk_disable = value,
            "GDK_DEBUG" => policy.gdk_debug = value,
            "LIBGL_ALWAYS_SOFTWARE" => policy.libgl_always_software = value,
            "GALLIUM_DRIVER" => policy.gallium_driver = value,
            "MESA_LOADER_DRIVER_OVERRIDE" => policy.mesa_loader_driver_override = value,
            "MESA_D3D12_DEFAULT_ADAPTER_NAME" => {
                policy.mesa_d3d12_default_adapter_name = value;
            }
            "LP_NUM_THREADS" => policy.lp_num_threads = value,
            _ => {}
        }
    }
    policy
}

fn requested_policy_from_env() -> RequestedRendererPolicy {
    requested_policy_from_pairs(
        RENDERER_ENV_KEYS
            .into_iter()
            .filter_map(|key| std::env::var(key).ok().map(|value| (key, value))),
    )
}

fn software_driver_name(value: &str) -> Option<&'static str> {
    let value = value.to_ascii_lowercase();
    ["llvmpipe", "swrast", "softpipe"]
        .into_iter()
        .find(|driver| value.contains(driver))
}

fn env_flag_enabled(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no"
    )
}

fn detect_software_fallback<'a>(
    selected_renderer: Option<&str>,
    policy: &RequestedRendererPolicy,
    thread_names: impl IntoIterator<Item = &'a str>,
) -> SoftwareFallbackEvidence {
    let mut indicators = Vec::new();

    if let Some(renderer) = selected_renderer {
        if renderer.to_ascii_lowercase().contains("cairorenderer") {
            indicators.push(format!("renderer:{renderer}"));
        } else if let Some(driver) = software_driver_name(renderer) {
            indicators.push(format!("renderer:{driver}"));
        }
    }
    if policy
        .libgl_always_software
        .as_deref()
        .is_some_and(env_flag_enabled)
    {
        indicators.push("env:LIBGL_ALWAYS_SOFTWARE".to_string());
    }
    if let Some(driver) = policy
        .gallium_driver
        .as_deref()
        .and_then(software_driver_name)
    {
        indicators.push(format!("env:GALLIUM_DRIVER={driver}"));
    }
    if let Some(driver) = policy
        .mesa_loader_driver_override
        .as_deref()
        .and_then(software_driver_name)
    {
        indicators.push(format!("env:MESA_LOADER_DRIVER_OVERRIDE={driver}"));
    }

    let mut thread_drivers = HashSet::new();
    for name in thread_names {
        if let Some(driver) = software_driver_name(name) {
            thread_drivers.insert(driver);
        }
    }
    for driver in ["llvmpipe", "swrast", "softpipe"] {
        if thread_drivers.contains(driver) {
            indicators.push(format!("thread:{driver}"));
        }
    }

    SoftwareFallbackEvidence {
        is_software_fallback: !indicators.is_empty(),
        indicators,
    }
}

fn preview_backend_matrix() -> Vec<PreviewBackend> {
    vec![
        PreviewBackend {
            name: "wsl-d3d12-gl",
            environment: &[("GSK_RENDERER", "gl"), ("GALLIUM_DRIVER", "d3d12")],
            expected: "GTK GL renderer backed by WSL D3D12",
            fallback: Some("desktop-gl"),
        },
        PreviewBackend {
            name: "desktop-gl",
            environment: &[("GSK_RENDERER", "gl")],
            expected: "GTK desktop GL renderer using automatic Mesa selection",
            fallback: Some("software-gl"),
        },
        PreviewBackend {
            name: "software-gl",
            environment: &[
                ("GSK_RENDERER", "gl"),
                ("LIBGL_ALWAYS_SOFTWARE", "1"),
                ("GALLIUM_DRIVER", "llvmpipe"),
                ("LP_NUM_THREADS", "2"),
            ],
            expected: "bounded final software GL fallback",
            fallback: None,
        },
    ]
}

fn fallback_chain(
    matrix: &[PreviewBackend],
    starting_backend: &str,
) -> Result<Vec<&'static str>, String> {
    let mut current = Some(starting_backend);
    let mut seen = HashSet::new();
    let mut chain = Vec::new();

    while let Some(name) = current {
        let entry = matrix
            .iter()
            .find(|entry| entry.name == name)
            .ok_or_else(|| format!("unknown preview renderer backend: {name}"))?;
        if !seen.insert(entry.name) {
            return Err(format!("preview renderer fallback cycle at {}", entry.name));
        }
        chain.push(entry.name);
        current = entry.fallback;
    }

    Ok(chain)
}

fn build_diagnostics<'a>(
    requested_policy: RequestedRendererPolicy,
    selected_renderer: Option<String>,
    thread_names: impl IntoIterator<Item = &'a str>,
    gpu_devices: GpuDeviceAvailability,
    gpu_device_usage: GpuDeviceUsage,
) -> RendererDiagnostics {
    let evidence = detect_software_fallback(
        selected_renderer.as_deref(),
        &requested_policy,
        thread_names,
    );
    let preview_backend_matrix = preview_backend_matrix();
    let preview_fallback_chain = fallback_chain(&preview_backend_matrix, "wsl-d3d12-gl")
        .unwrap_or_else(|_| vec!["software-gl"]);
    RendererDiagnostics {
        requested_policy,
        selected_renderer,
        is_software_fallback: evidence.is_software_fallback,
        fallback_indicators: evidence.indicators,
        gpu_devices,
        gpu_device_usage,
        preview_backend_matrix,
        preview_fallback_chain,
    }
}

#[cfg(target_os = "linux")]
fn current_thread_names() -> Vec<String> {
    let Ok(tasks) = std::fs::read_dir("/proc/self/task") else {
        return Vec::new();
    };

    tasks
        .take(512)
        .filter_map(Result::ok)
        .filter_map(|task| std::fs::read_to_string(task.path().join("comm")).ok())
        .map(|name| name.trim().to_string())
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn current_thread_names() -> Vec<String> {
    Vec::new()
}

fn gpu_device_availability() -> GpuDeviceAvailability {
    let dri_render_node = std::fs::read_dir("/dev/dri").ok().is_some_and(|entries| {
        entries
            .take(64)
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with("renderD"))
    });
    GpuDeviceAvailability {
        dxg: Path::new("/dev/dxg").exists(),
        dri_render_node,
    }
}

fn gpu_device_usage_from_targets<I, P>(targets: I) -> GpuDeviceUsage
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut usage = GpuDeviceUsage {
        dxg_open: false,
        dri_render_node_open: false,
    };
    for target in targets {
        let target = target.as_ref();
        usage.dxg_open |= target == Path::new("/dev/dxg");
        usage.dri_render_node_open |= target
            .parent()
            .is_some_and(|parent| parent == Path::new("/dev/dri"))
            && target
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("renderD"));
    }
    usage
}

#[cfg(target_os = "linux")]
fn gpu_device_usage() -> GpuDeviceUsage {
    let targets = std::fs::read_dir("/proc/self/fd")
        .ok()
        .into_iter()
        .flatten()
        .take(1024)
        .filter_map(Result::ok)
        .filter_map(|entry| std::fs::read_link(entry.path()).ok())
        .collect::<Vec<_>>();
    gpu_device_usage_from_targets(targets)
}

#[cfg(not(target_os = "linux"))]
fn gpu_device_usage() -> GpuDeviceUsage {
    GpuDeviceUsage {
        dxg_open: false,
        dri_render_node_open: false,
    }
}

pub(super) fn capture(window: &adw::ApplicationWindow) {
    if DIAGNOSTICS.get().is_some() {
        return;
    }

    let selected_renderer = window
        .renderer()
        .map(|renderer| renderer.type_().name().to_string());
    let thread_names = current_thread_names();
    let diagnostics = build_diagnostics(
        requested_policy_from_env(),
        selected_renderer,
        thread_names.iter().map(String::as_str),
        gpu_device_availability(),
        gpu_device_usage(),
    );

    if DIAGNOSTICS.set(diagnostics.clone()).is_ok() {
        eprintln!("limux: renderer diagnostics {}", diagnostics.to_json());
    }
}

pub(super) fn current_json() -> serde_json::Value {
    let Some(diagnostics) = DIAGNOSTICS.get() else {
        return serde_json::json!({ "status": "pending" });
    };
    let mut payload = diagnostics.to_json();
    if payload.get("status").and_then(serde_json::Value::as_str) == Some("error") {
        return payload;
    }
    if let Some(payload) = payload.as_object_mut() {
        payload.insert(
            "status".to_string(),
            serde_json::Value::String("captured".to_string()),
        );
    }
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_policy_records_only_renderer_controls() {
        let policy = requested_policy_from_pairs([
            ("GSK_RENDERER", "gl"),
            ("GDK_DISABLE", "gles-api,vulkan"),
            ("GALLIUM_DRIVER", "d3d12"),
            ("UNRELATED_SECRET", "must-not-appear"),
        ]);

        assert_eq!(policy.gsk_renderer.as_deref(), Some("gl"));
        assert_eq!(policy.gdk_disable.as_deref(), Some("gles-api,vulkan"));
        assert_eq!(policy.gallium_driver.as_deref(), Some("d3d12"));
        assert!(!serde_json::to_string(&policy)
            .expect("serialize renderer policy")
            .contains("must-not-appear"));
    }

    #[test]
    fn llvmpipe_threads_mark_software_fallback_without_unbounded_details() {
        let evidence = detect_software_fallback(
            Some("GskGLRenderer"),
            &RequestedRendererPolicy::default(),
            ["gmain", "llvmpipe-0", "llvmpipe-1", "renderer"],
        );

        assert!(evidence.is_software_fallback);
        assert_eq!(evidence.indicators, ["thread:llvmpipe"]);
    }

    #[test]
    fn explicit_software_environment_is_detected_without_threads() {
        let policy = requested_policy_from_pairs([
            ("LIBGL_ALWAYS_SOFTWARE", "1"),
            ("GALLIUM_DRIVER", "swrast"),
            ("LP_NUM_THREADS", "2"),
        ]);
        let evidence = detect_software_fallback(Some("GskGLRenderer"), &policy, []);

        assert!(evidence.is_software_fallback);
        assert_eq!(
            evidence.indicators,
            ["env:LIBGL_ALWAYS_SOFTWARE", "env:GALLIUM_DRIVER=swrast"]
        );
        assert_eq!(policy.lp_num_threads.as_deref(), Some("2"));
    }

    #[test]
    fn cairo_renderer_is_reported_as_software_fallback() {
        let evidence = detect_software_fallback(
            Some("GskCairoRenderer"),
            &RequestedRendererPolicy::default(),
            [],
        );

        assert!(evidence.is_software_fallback);
        assert_eq!(evidence.indicators, ["renderer:GskCairoRenderer"]);
    }

    #[test]
    fn preview_backend_matrix_has_a_finite_fail_closed_chain() {
        let matrix = preview_backend_matrix();
        let chain = fallback_chain(&matrix, "wsl-d3d12-gl").expect("valid backend chain");

        assert_eq!(chain, ["wsl-d3d12-gl", "desktop-gl", "software-gl"]);
        assert!(matrix
            .iter()
            .all(|entry| entry.name != "software-gl" || entry.fallback.is_none()));
    }

    #[test]
    fn diagnostics_payload_exposes_selection_and_fallback_evidence() {
        let diagnostics = build_diagnostics(
            requested_policy_from_pairs([("GSK_RENDERER", "gl")]),
            Some("GskGLRenderer".to_string()),
            ["llvmpipe-0"],
            GpuDeviceAvailability {
                dxg: true,
                dri_render_node: false,
            },
            GpuDeviceUsage {
                dxg_open: false,
                dri_render_node_open: false,
            },
        );
        let payload = diagnostics.to_json();

        assert_eq!(payload["selected_renderer"], "GskGLRenderer");
        assert_eq!(payload["is_software_fallback"], true);
        assert_eq!(payload["gpu_devices"]["dxg"], true);
        assert_eq!(payload["fallback_indicators"][0], "thread:llvmpipe");
    }

    #[test]
    fn gpu_device_usage_distinguishes_open_dxg_and_dri_render_nodes() {
        let usage = gpu_device_usage_from_targets([
            "/dev/dxg",
            "/dev/dri/renderD128",
            "/tmp/unrelated-device",
        ]);

        assert!(usage.dxg_open);
        assert!(usage.dri_render_node_open);
    }
}
