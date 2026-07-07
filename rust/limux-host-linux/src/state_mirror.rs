use limux_core::{ControlState, ControlStateSnapshot, Dispatcher};
use limux_protocol::{V2Request, V2Response};
use serde_json::Value;

const INTERNAL_ERROR_CODE: i64 = -32603;

pub(crate) fn dispatch_snapshot(
    snapshot: ControlStateSnapshot,
    method: String,
    params: Value,
) -> V2Response {
    let mut state = ControlState::default();
    if let Err(error) = state.import_snapshot(snapshot) {
        return V2Response::error(
            None,
            INTERNAL_ERROR_CODE,
            format!("control snapshot import failed: {error}"),
            None,
        );
    }

    Dispatcher::with_state(state).dispatch_sync(V2Request::new(method, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use limux_core::{PaneSnapshot, SurfaceSnapshot, WindowSnapshot, WorkspaceSnapshot};
    use serde_json::json;

    fn snapshot() -> ControlStateSnapshot {
        ControlStateSnapshot {
            current_workspace_id: Some(1),
            notifications: Vec::new(),
            workspaces: vec![WorkspaceSnapshot {
                id: 1,
                name: "dev".to_string(),
                cwd: Some("/repo".to_string()),
                host_window_id: 10,
                current_window_id: Some(10),
                windows: vec![WindowSnapshot {
                    id: 10,
                    title: "dev".to_string(),
                    current_pane_id: Some(20),
                    panes: vec![PaneSnapshot {
                        id: 20,
                        current_surface_id: Some(30),
                        flag_color: Some("orange".to_string()),
                        surfaces: vec![SurfaceSnapshot {
                            id: 30,
                            title: "terminal".to_string(),
                            text: String::new(),
                            panel_type: "terminal".to_string(),
                            developer_tools_visible: false,
                            pinned: false,
                            unread: false,
                            flash_count: 0,
                            refresh_count: 0,
                        }],
                    }],
                }],
            }],
        }
    }

    #[test]
    fn dispatches_window_list_from_snapshot() {
        let response = dispatch_snapshot(snapshot(), "window.list".to_string(), json!({}));

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["windows"][0]["title"], "dev");
        assert_eq!(result["windows"][0]["pane_count"], 1);
    }

    #[test]
    fn dispatches_window_current_from_snapshot() {
        let response = dispatch_snapshot(snapshot(), "window.current".to_string(), json!({}));

        assert_eq!(response.error, None);
        let result = response.result.expect("result");
        assert_eq!(result["window"]["title"], "dev");
        assert_eq!(result["window"]["current_pane_id"], 20);
    }
}
