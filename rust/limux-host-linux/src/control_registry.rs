#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouteClass {
    BridgeNative,
    CoreRead,
    Wave1Mutation,
    Deferred,
}

// PRD-E names a future `restricted` class for Cursor co-design. The shipped
// Cursor surface is still enforced by limux_protocol::restricted_method_allowlist,
// so cursor routes stay BridgeNative here until the Cursor lane consumes this
// registry as its source of truth.
pub(crate) const WAVE1_MUTATION_KILL_SWITCH_ENV: &str = "LIMUX_DISABLE_WAVE1_MUTATIONS";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RouteEntry {
    pub(crate) method: &'static str,
    pub(crate) class: RouteClass,
}

const ROUTES: &[RouteEntry] = &[
    RouteEntry {
        method: "system.ping",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "system.identify",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "system.capabilities",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "window.present",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.current",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.list",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.create",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.select",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.rename",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "workspace.close",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "pane.list",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "pane.surfaces",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "pane.create",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "pane.action",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "surface.list",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "surface.health",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "surface.read_text",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "surface.send_text",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "surface.send_key",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "notification.create",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "cursor.pane_create_empty",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "cursor.workspace_open_folder",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "window.list",
        class: RouteClass::CoreRead,
    },
    RouteEntry {
        method: "window.current",
        class: RouteClass::CoreRead,
    },
    RouteEntry {
        method: "surface.current",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "notification.list",
        class: RouteClass::BridgeNative,
    },
    RouteEntry {
        method: "pane.focus",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "pane.resize",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "resize-pane",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "surface.split",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "surface.focus",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "surface.close",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "workspace.reorder",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "workspace.next",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "workspace.previous",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "workspace.last",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "notification.clear",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "tab.action",
        class: RouteClass::Wave1Mutation,
    },
    RouteEntry {
        method: "pane.swap",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "pane.break",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "pane.join",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "surface.move",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "surface.reorder",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "surface.drag_to_split",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "workspace.move_to_window",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "window.create",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "window.close",
        class: RouteClass::Deferred,
    },
    RouteEntry {
        method: "window.focus",
        class: RouteClass::Deferred,
    },
];

const WIRED_WAVE1_MUTATIONS: &[&str] = &["pane.focus"];

impl RouteClass {
    fn is_capability_advertised(self) -> bool {
        matches!(self, Self::BridgeNative | Self::CoreRead)
    }
}

pub(crate) fn routes() -> &'static [RouteEntry] {
    ROUTES
}

pub(crate) fn route_class(method: &str) -> Option<RouteClass> {
    routes()
        .iter()
        .find(|entry| entry.method == method)
        .map(|entry| entry.class)
}

pub(crate) fn capability_methods() -> Vec<&'static str> {
    capability_methods_for_wave1_disabled(wave1_mutations_disabled())
}

fn capability_methods_for_wave1_disabled(wave1_disabled: bool) -> Vec<&'static str> {
    routes()
        .iter()
        .filter(|entry| {
            entry.class.is_capability_advertised()
                || (!wave1_disabled && is_wired_wave1_mutation(entry.method))
        })
        .map(|entry| entry.method)
        .collect()
}

pub(crate) fn is_read_only_fallthrough(method: &str) -> bool {
    route_class(method) == Some(RouteClass::CoreRead)
}

pub(crate) fn is_wired_wave1_mutation(method: &str) -> bool {
    WIRED_WAVE1_MUTATIONS.contains(&method)
}

pub(crate) fn wave1_mutations_disabled() -> bool {
    wave1_mutations_disabled_from_env_value(
        std::env::var(WAVE1_MUTATION_KILL_SWITCH_ENV)
            .ok()
            .as_deref(),
    )
}

pub(crate) fn wave1_mutations_disabled_from_env_value(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_reads_are_fallthrough_methods() {
        assert_eq!(route_class("window.list"), Some(RouteClass::CoreRead));
        assert_eq!(route_class("window.current"), Some(RouteClass::CoreRead));
    }

    #[test]
    fn live_reads_are_bridge_native_not_fallthrough() {
        assert_eq!(
            route_class("surface.current"),
            Some(RouteClass::BridgeNative)
        );
        assert_eq!(
            route_class("notification.list"),
            Some(RouteClass::BridgeNative)
        );
        assert!(!is_read_only_fallthrough("surface.current"));
        assert!(!is_read_only_fallthrough("notification.list"));
        assert!(capability_methods().contains(&"surface.current"));
        assert!(capability_methods().contains(&"notification.list"));
    }

    #[test]
    fn surface_read_text_is_live_bridge_native_not_fallthrough() {
        assert_eq!(
            route_class("surface.read_text"),
            Some(RouteClass::BridgeNative)
        );
        assert!(!is_read_only_fallthrough("surface.read_text"));
    }

    #[test]
    fn wave1_mutation_methods_are_classified_and_only_wired_routes_are_advertised() {
        let advertised = capability_methods_for_wave1_disabled(false);
        for method in [
            "pane.focus",
            "pane.resize",
            "resize-pane",
            "surface.split",
            "surface.focus",
            "surface.close",
            "workspace.reorder",
            "workspace.next",
            "workspace.previous",
            "workspace.last",
            "notification.clear",
            "tab.action",
        ] {
            assert_eq!(
                route_class(method),
                Some(RouteClass::Wave1Mutation),
                "{method} should be in the Wave-1 mutation set"
            );
        }
        assert!(advertised.contains(&"pane.focus"));
        for method in [
            "pane.resize",
            "resize-pane",
            "surface.split",
            "surface.focus",
            "surface.close",
            "workspace.reorder",
            "workspace.next",
            "workspace.previous",
            "workspace.last",
            "notification.clear",
            "tab.action",
        ] {
            assert!(
                !advertised.contains(&method),
                "{method} should not be advertised until a live GTK route is wired"
            );
        }
    }

    #[test]
    fn wired_wave1_capabilities_are_hidden_when_kill_switch_is_enabled() {
        let methods = capability_methods_for_wave1_disabled(true);

        assert!(!methods.contains(&"pane.focus"));
        assert!(methods.contains(&"workspace.list"));
        assert!(methods.contains(&"surface.current"));
    }

    #[test]
    fn explicitly_deferred_methods_are_classified_and_not_advertised() {
        for method in [
            "pane.swap",
            "pane.break",
            "pane.join",
            "surface.move",
            "surface.reorder",
            "surface.drag_to_split",
            "workspace.move_to_window",
            "window.create",
            "window.close",
            "window.focus",
        ] {
            assert_eq!(
                route_class(method),
                Some(RouteClass::Deferred),
                "{method} should have an explicit PRD-E deferred classification"
            );
            assert!(
                !capability_methods().contains(&method),
                "{method} should not be advertised while deferred"
            );
        }
    }

    #[test]
    fn wave1_mutation_kill_switch_accepts_standard_truthy_values() {
        for value in ["1", "true", "TRUE", "yes", "on", " On "] {
            assert!(
                wave1_mutations_disabled_from_env_value(Some(value)),
                "{value:?} should disable Wave-1 mutations"
            );
        }

        for value in [None, Some(""), Some("0"), Some("false"), Some("off")] {
            assert!(
                !wave1_mutations_disabled_from_env_value(value),
                "{value:?} should leave Wave-1 mutations enabled"
            );
        }
    }
}
