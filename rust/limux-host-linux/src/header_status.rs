use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderSection {
    Application,
    Workspace,
    Resources,
    DirectoryManagers,
}

impl HeaderSection {
    pub(crate) fn defaults() -> Vec<Self> {
        vec![
            Self::Application,
            Self::Workspace,
            Self::Resources,
            Self::DirectoryManagers,
        ]
    }

    pub(crate) fn from_name(value: &str) -> Option<Self> {
        match value {
            "application" => Some(Self::Application),
            "workspace" => Some(Self::Workspace),
            "resources" => Some(Self::Resources),
            "directory_managers" => Some(Self::DirectoryManagers),
            _ => None,
        }
    }

    pub(crate) fn as_name(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Workspace => "workspace",
            Self::Resources => "resources",
            Self::DirectoryManagers => "directory_managers",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeaderSnapshot {
    pub(crate) workspace_name: String,
    pub(crate) pane_count: usize,
    pub(crate) ram_mb: Option<u64>,
    pub(crate) cpu_percent: Option<f64>,
    pub(crate) managers: ManagerStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagerStatus {
    Loading,
    Available(Vec<String>),
    Unavailable,
}

pub(crate) fn render_header_markup(
    version: &str,
    sections: &[HeaderSection],
    snapshot: &HeaderSnapshot,
) -> String {
    let segments = sections
        .iter()
        .map(|section| match section {
            HeaderSection::Application => format!("Limux v{}", escape_markup(version)),
            HeaderSection::Workspace => format!(
                "<b>WORKSPACE: {}</b>",
                escape_markup(&snapshot.workspace_name)
            ),
            HeaderSection::Resources => {
                let ram = snapshot
                    .ram_mb
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "--".to_string());
                let cpu = snapshot
                    .cpu_percent
                    .map(|value| format!("{:.1}", (value * 10.0).round() / 10.0))
                    .unwrap_or_else(|| "--".to_string());
                format!(
                    "<b>[LIVE]</b> <b>[ACTIVE]</b> PANES:{} RAM:{ram}MB CPU:{cpu}%",
                    snapshot.pane_count
                )
            }
            HeaderSection::DirectoryManagers => {
                let value = match &snapshot.managers {
                    ManagerStatus::Available(managers) if managers.is_empty() => {
                        "none (0)".to_string()
                    }
                    ManagerStatus::Available(managers) => {
                        format!("{} ({})", managers.join(", "), managers.len())
                    }
                    ManagerStatus::Loading => "loading".to_string(),
                    ManagerStatus::Unavailable => "unavailable".to_string(),
                };
                format!("DIR MGR(s): <b>{}</b>", escape_markup(&value))
            }
        })
        .collect::<Vec<_>>();

    segments.join(" <b>|</b> ")
}

fn escape_markup(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn manager_names_for_directory(
    payload: &str,
    directory: &str,
) -> Result<Vec<String>, String> {
    let rows = serde_json::from_str::<Vec<serde_json::Value>>(payload)
        .map_err(|error| format!("invalid hcom manager JSON: {error}"))?;
    let mut names = BTreeSet::new();

    for row in rows {
        if row.get("liveness").and_then(serde_json::Value::as_str) != Some("live") {
            continue;
        }
        let covered = row
            .get("covered_by_claim_path")
            .and_then(serde_json::Value::as_str)
            .or_else(|| row.get("project_key").and_then(serde_json::Value::as_str));
        if !covered.is_some_and(|scope| scope_covers_directory(scope, directory)) {
            continue;
        }
        if let Some(name) = row.get("mgr").and_then(serde_json::Value::as_str) {
            let name = name.trim();
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }

    Ok(names.into_iter().collect())
}

fn scope_covers_directory(scope: &str, directory: &str) -> bool {
    let scope = scope
        .trim()
        .strip_suffix("/**")
        .or_else(|| scope.trim().strip_suffix("/*"))
        .unwrap_or_else(|| scope.trim());
    !scope.is_empty() && Path::new(directory).starts_with(Path::new(scope))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcStat {
    pub(crate) pid: u32,
    pub(crate) parent_pid: u32,
    pub(crate) cpu_ticks: u64,
}

pub(crate) fn parse_proc_stat(raw: &str) -> Option<ProcStat> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open {
        return None;
    }
    let pid = raw[..open].trim().parse().ok()?;
    let tail = raw[close + 1..].split_whitespace().collect::<Vec<_>>();
    let parent_pid = tail.get(1)?.parse().ok()?;
    let user_ticks = tail.get(11)?.parse::<u64>().ok()?;
    let system_ticks = tail.get(12)?.parse::<u64>().ok()?;
    Some(ProcStat {
        pid,
        parent_pid,
        cpu_ticks: user_ticks.saturating_add(system_ticks),
    })
}

pub(crate) fn cpu_percent_from_ticks(
    previous_ticks: u64,
    current_ticks: u64,
    elapsed_seconds: f64,
    clock_ticks_per_second: u64,
) -> Option<f64> {
    if elapsed_seconds <= 0.0 || clock_ticks_per_second == 0 {
        return None;
    }
    let delta = current_ticks.saturating_sub(previous_ticks) as f64;
    Some((delta / clock_ticks_per_second as f64 / elapsed_seconds * 100.0).max(0.0))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessUsage {
    pub(crate) pid: u32,
    pub(crate) parent_pid: u32,
    pub(crate) cpu_ticks: u64,
    pub(crate) rss_kb: u64,
}

pub(crate) fn process_tree_totals(root_pid: u32, processes: &[ProcessUsage]) -> (u64, u64) {
    let mut included = HashSet::from([root_pid]);
    loop {
        let before = included.len();
        for process in processes {
            if included.contains(&process.parent_pid) {
                included.insert(process.pid);
            }
        }
        if included.len() == before {
            break;
        }
    }

    processes
        .iter()
        .filter(|process| included.contains(&process.pid))
        .fold((0u64, 0u64), |(cpu, rss), process| {
            (
                cpu.saturating_add(process.cpu_ticks),
                rss.saturating_add(process.rss_kb),
            )
        })
}

pub(crate) fn parse_vm_rss_kb(raw: &str) -> Option<u64> {
    raw.lines().find_map(|line| {
        let value = line.strip_prefix("VmRSS:")?.trim();
        value.split_whitespace().next()?.parse().ok()
    })
}

pub(crate) struct ProcessTreeSampler {
    previous: Option<(Instant, u64)>,
    clock_ticks_per_second: u64,
}

impl ProcessTreeSampler {
    pub(crate) fn new() -> Self {
        let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        Self {
            previous: None,
            clock_ticks_per_second: u64::try_from(ticks).unwrap_or(100),
        }
    }

    pub(crate) fn sample(&mut self) -> (Option<u64>, Option<f64>) {
        let processes = read_process_usage();
        if processes.is_empty() {
            return (None, None);
        }

        let (cpu_ticks, rss_kb) = process_tree_totals(std::process::id(), &processes);
        let now = Instant::now();
        let cpu_percent = self.previous.and_then(|(previous_at, previous_ticks)| {
            cpu_percent_from_ticks(
                previous_ticks,
                cpu_ticks,
                now.duration_since(previous_at).as_secs_f64(),
                self.clock_ticks_per_second,
            )
        });
        self.previous = Some((now, cpu_ticks));

        (Some(rss_kb.saturating_add(1_023) / 1_024), cpu_percent)
    }
}

fn read_process_usage() -> Vec<ProcessUsage> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<u32>().ok())
        .filter_map(|pid| {
            let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
            let parsed = parse_proc_stat(&stat)?;
            let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
            Some(ProcessUsage {
                pid: parsed.pid,
                parent_pid: parsed.parent_pid,
                cpu_ticks: parsed.cpu_ticks,
                rss_kb: parse_vm_rss_kb(&status).unwrap_or(0),
            })
        })
        .collect()
}

pub(crate) fn query_directory_managers(directory: &str) -> Result<Vec<String>, String> {
    let run = |program: &std::ffi::OsStr| {
        Command::new(program)
            .args(["list", "mgrs", "--json"])
            .env_remove("HCOM_NAME")
            .env_remove("HCOM_PROCESS_ID")
            .output()
    };
    let output = match run(std::ffi::OsStr::new("hcom")) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let fallback = dirs::home_dir()
                .map(|home| home.join(".local/bin/hcom"))
                .ok_or_else(|| "failed to resolve user-local hcom path".to_string())?;
            run(fallback.as_os_str()).map_err(|fallback_error| {
                format!("failed to run hcom manager query: {fallback_error}")
            })?
        }
        Err(error) => return Err(format!("failed to run hcom manager query: {error}")),
    };

    if !output.status.success() {
        return Err(format!(
            "hcom manager query exited with status {}",
            output.status
        ));
    }
    let payload = String::from_utf8(output.stdout)
        .map_err(|error| format!("hcom manager query returned non-UTF-8 output: {error}"))?;
    manager_names_for_directory(&payload, directory)
}

#[cfg(test)]
mod tests {
    use super::{
        cpu_percent_from_ticks, manager_names_for_directory, parse_proc_stat, parse_vm_rss_kb,
        process_tree_totals, render_header_markup, HeaderSection, HeaderSnapshot, ManagerStatus,
        ProcessUsage,
    };

    #[test]
    fn default_header_markup_orders_sections_and_escapes_dynamic_text() {
        let snapshot = HeaderSnapshot {
            workspace_name: "A&B <ops>".to_string(),
            pane_count: 3,
            ram_mb: Some(512),
            cpu_percent: Some(7.25),
            managers: ManagerStatus::Available(vec!["lifo".to_string(), "ops<&".to_string()]),
        };

        let markup = render_header_markup("0.2.2", &HeaderSection::defaults(), &snapshot);

        assert_eq!(
            markup,
            "Limux v0.2.2 <b>|</b> <b>WORKSPACE: A&amp;B &lt;ops&gt;</b> <b>|</b> <b>[LIVE]</b> <b>[ACTIVE]</b> PANES:3 RAM:512MB CPU:7.3% <b>|</b> DIR MGR(s): <b>lifo, ops&lt;&amp; (2)</b>"
        );
    }

    #[test]
    fn configured_header_sections_can_be_reordered_or_omitted() {
        let snapshot = HeaderSnapshot {
            workspace_name: "dev".to_string(),
            pane_count: 1,
            ram_mb: None,
            cpu_percent: None,
            managers: ManagerStatus::Loading,
        };

        let markup = render_header_markup(
            "0.2.2",
            &[HeaderSection::Workspace, HeaderSection::Application],
            &snapshot,
        );

        assert_eq!(markup, "<b>WORKSPACE: dev</b> <b>|</b> Limux v0.2.2");
    }

    #[test]
    fn manager_names_are_live_unique_and_scoped_to_the_directory() {
        let payload = r#"[
          {
            "mgr": "lifo",
            "liveness": "live",
            "covered_by_claim_path": "/home/riche/MCPs/limux"
          },
          {
            "mgr": "lifo",
            "liveness": "live",
            "project_key": "/home/riche/MCPs/limux"
          },
          {
            "mgr": "dino",
            "liveness": "live",
            "covered_by_claim_path": "/home/riche/MCPs/hcom"
          },
          {
            "mgr": "old",
            "liveness": "stale",
            "covered_by_claim_path": "/home/riche/MCPs/limux"
          }
        ]"#;

        assert_eq!(
            manager_names_for_directory(payload, "/home/riche/MCPs/limux/rust"),
            Ok(vec!["lifo".to_string()])
        );
    }

    #[test]
    fn proc_stat_parser_handles_process_names_with_spaces_and_parentheses() {
        let parsed = parse_proc_stat(
            "123 (limux pane (dev)) S 42 0 0 0 0 0 0 0 0 0 100 25 0 0 0 0 0 0 0 0 0 0",
        )
        .expect("valid stat line");

        assert_eq!(parsed.pid, 123);
        assert_eq!(parsed.parent_pid, 42);
        assert_eq!(parsed.cpu_ticks, 125);
    }

    #[test]
    fn cpu_percent_uses_elapsed_time_and_clock_ticks() {
        assert_eq!(cpu_percent_from_ticks(100, 125, 0.5, 100), Some(50.0));
        assert_eq!(cpu_percent_from_ticks(125, 100, 0.5, 100), Some(0.0));
        assert_eq!(cpu_percent_from_ticks(100, 125, 0.0, 100), None);
    }

    #[test]
    fn process_tree_totals_include_only_the_host_and_its_descendants() {
        let processes = vec![
            ProcessUsage {
                pid: 10,
                parent_pid: 1,
                cpu_ticks: 100,
                rss_kb: 1_024,
            },
            ProcessUsage {
                pid: 11,
                parent_pid: 10,
                cpu_ticks: 20,
                rss_kb: 2_048,
            },
            ProcessUsage {
                pid: 12,
                parent_pid: 11,
                cpu_ticks: 5,
                rss_kb: 512,
            },
            ProcessUsage {
                pid: 99,
                parent_pid: 1,
                cpu_ticks: 9_999,
                rss_kb: 9_999,
            },
        ];

        assert_eq!(process_tree_totals(10, &processes), (125, 3_584));
    }

    #[test]
    fn vm_rss_parser_reads_kilobytes_and_rejects_missing_values() {
        assert_eq!(
            parse_vm_rss_kb("Name:\tlimux\nVmRSS:\t  4242 kB\nThreads:\t4\n"),
            Some(4_242)
        );
        assert_eq!(parse_vm_rss_kb("Name:\tlimux\n"), None);
    }

    #[test]
    fn unavailable_manager_status_is_explicit() {
        let snapshot = HeaderSnapshot {
            workspace_name: "dev".to_string(),
            pane_count: 1,
            ram_mb: Some(64),
            cpu_percent: Some(1.0),
            managers: ManagerStatus::Unavailable,
        };

        assert!(
            render_header_markup("0.2.2", &[HeaderSection::DirectoryManagers], &snapshot,)
                .contains("DIR MGR(s): <b>unavailable</b>")
        );
    }
}
