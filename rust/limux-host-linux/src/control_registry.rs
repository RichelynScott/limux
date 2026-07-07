#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouteClass {
    BridgeNative,
    ReadOnlyFallthrough,
}

const READ_ONLY_FALLTHROUGH_METHODS: &[&str] = &["window.list", "window.current"];

const BRIDGE_NATIVE_METHODS: &[&str] = &[
    "system.ping",
    "system.identify",
    "system.capabilities",
    "window.present",
    "workspace.current",
    "workspace.list",
    "workspace.create",
    "workspace.select",
    "workspace.rename",
    "workspace.close",
    "pane.list",
    "pane.surfaces",
    "pane.create",
    "pane.action",
    "surface.list",
    "surface.health",
    "surface.read_text",
    "surface.send_text",
    "surface.send_key",
    "notification.create",
    "cursor.pane_create_empty",
    "cursor.workspace_open_folder",
];

pub(crate) fn route_class(method: &str) -> Option<RouteClass> {
    if READ_ONLY_FALLTHROUGH_METHODS.contains(&method) {
        Some(RouteClass::ReadOnlyFallthrough)
    } else if BRIDGE_NATIVE_METHODS.contains(&method) {
        Some(RouteClass::BridgeNative)
    } else {
        None
    }
}

pub(crate) fn is_read_only_fallthrough(method: &str) -> bool {
    route_class(method) == Some(RouteClass::ReadOnlyFallthrough)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_reads_are_fallthrough_methods() {
        assert_eq!(
            route_class("window.list"),
            Some(RouteClass::ReadOnlyFallthrough)
        );
        assert_eq!(
            route_class("window.current"),
            Some(RouteClass::ReadOnlyFallthrough)
        );
    }

    #[test]
    fn surface_read_text_is_live_bridge_native_not_fallthrough() {
        assert_eq!(
            route_class("surface.read_text"),
            Some(RouteClass::BridgeNative)
        );
        assert!(!is_read_only_fallthrough("surface.read_text"));
    }
}
