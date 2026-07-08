use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;

use crate::layout_state::{self, LayoutNodeState, PaneState, SplitOrientation, SplitState};
use crate::pane;
use crate::window::{
    apply_split_ratio_after_layout, attach_split_position_persistence,
    minimum_split_extent_for_orientation, update_split_ratio_state, State,
};

// ---------------------------------------------------------------------------
// SplitNode — runtime data model for the split tree
// ---------------------------------------------------------------------------

/// Runtime split tree node. Source of truth for the split layout.
/// The widget tree is rebuilt from this on every structural change.
pub(crate) enum SplitNode {
    Leaf {
        pane_widget: gtk::Widget,
    },
    Split {
        orientation: gtk::Orientation,
        /// Shared with the Paned's position_notify handler so resize drags
        /// update the data model directly.
        ratio: Rc<RefCell<f64>>,
        left: Box<SplitNode>,
        right: Box<SplitNode>,
    },
}

impl SplitNode {
    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self, SplitNode::Leaf { .. })
    }

    /// Find the leaf containing `target` and replace it with `replacement`.
    pub(crate) fn replace(&mut self, target: &gtk::Widget, replacement: SplitNode) -> bool {
        match self {
            SplitNode::Leaf { pane_widget } => {
                if pane_widget == target {
                    *self = replacement;
                    true
                } else {
                    false
                }
            }
            SplitNode::Split { left, right, .. } => {
                // Check containment first to route ownership to the correct subtree
                if left.contains_pane(target) {
                    left.replace(target, replacement)
                } else {
                    right.replace(target, replacement)
                }
            }
        }
    }

    fn contains_pane(&self, target: &gtk::Widget) -> bool {
        match self {
            SplitNode::Leaf { pane_widget } => pane_widget == target,
            SplitNode::Split { left, right, .. } => {
                left.contains_pane(target) || right.contains_pane(target)
            }
        }
    }

    /// Find the leaf containing `target` and promote its sibling in place.
    pub(crate) fn remove(&mut self, target: &gtk::Widget) -> bool {
        match self {
            SplitNode::Leaf { .. } => false,
            SplitNode::Split { left, right, .. } => {
                if matches!(left.as_ref(), SplitNode::Leaf { pane_widget } if pane_widget == target)
                {
                    // Target is left child — promote right sibling.
                    *self = std::mem::replace(
                        right.as_mut(),
                        SplitNode::Leaf {
                            pane_widget: target.clone(),
                        },
                    );
                    return true;
                }
                if matches!(right.as_ref(), SplitNode::Leaf { pane_widget } if pane_widget == target)
                {
                    // Target is right child — promote left sibling.
                    *self = std::mem::replace(
                        left.as_mut(),
                        SplitNode::Leaf {
                            pane_widget: target.clone(),
                        },
                    );
                    return true;
                }
                left.remove(target) || right.remove(target)
            }
        }
    }

    /// Snapshot to the serializable layout format for session persistence.
    pub(crate) fn snapshot(&self, working_directory: Option<&str>) -> LayoutNodeState {
        match self {
            SplitNode::Leaf { pane_widget } => pane::snapshot_pane_state(pane_widget)
                .map(LayoutNodeState::Pane)
                .unwrap_or_else(|| LayoutNodeState::Pane(PaneState::fallback(working_directory))),
            SplitNode::Split {
                orientation,
                ratio,
                left,
                right,
            } => LayoutNodeState::Split(SplitState {
                orientation: if *orientation == gtk::Orientation::Horizontal {
                    SplitOrientation::Horizontal
                } else {
                    SplitOrientation::Vertical
                },
                ratio: *ratio.borrow(),
                start: Box::new(left.snapshot(working_directory)),
                end: Box::new(right.snapshot(working_directory)),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// SplitTreeContainer — manages async widget-tree rebuild lifecycle
// ---------------------------------------------------------------------------

/// Manages the workspace's split layout following Ghostty's atomic rebuild
/// pattern. Holds a SplitNode data model (source of truth) and a gtk::Box
/// container for the built widget tree. On structural changes, tears down the
/// old widget tree and rebuilds from the data model on the next idle tick.
pub(crate) struct SplitTreeContainer {
    tree: RefCell<SplitNode>,
    bin: gtk::Box,
    rebuild_source: RefCell<Option<glib::SourceId>>,
    last_focused: RefCell<Option<gtk::Widget>>,
    zoomed_pane: RefCell<Option<gtk::Widget>>,
    state: State,
}

impl SplitTreeContainer {
    /// Create a new container with a single pane (no splits).
    pub(crate) fn new(state: &State, initial_pane: gtk::Widget) -> Rc<Self> {
        let bin = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bin.set_hexpand(true);
        bin.set_vexpand(true);
        bin.append(&initial_pane);

        Rc::new(Self {
            tree: RefCell::new(SplitNode::Leaf {
                pane_widget: initial_pane,
            }),
            bin,
            rebuild_source: RefCell::new(None),
            last_focused: RefCell::new(None),
            zoomed_pane: RefCell::new(None),
            state: state.clone(),
        })
    }

    /// Create a container from a pre-built tree (for session restore).
    pub(crate) fn new_from_tree(state: &State, node: SplitNode) -> Rc<Self> {
        let bin = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bin.set_hexpand(true);
        bin.set_vexpand(true);

        // Build the initial widget tree synchronously (no async needed on first build)
        let widget = build_widget_tree(&node, state);
        bin.append(&widget);

        Rc::new(Self {
            tree: RefCell::new(node),
            bin,
            rebuild_source: RefCell::new(None),
            last_focused: RefCell::new(None),
            zoomed_pane: RefCell::new(None),
            state: state.clone(),
        })
    }

    /// The container widget to add to the gtk::Stack.
    pub(crate) fn widget(&self) -> &gtk::Box {
        &self.bin
    }

    /// Borrow the tree for reading (e.g. session snapshot).
    pub(crate) fn tree(&self) -> std::cell::Ref<'_, SplitNode> {
        self.tree.borrow()
    }

    /// Whether the tree is a single leaf (no splits).
    pub(crate) fn is_single_pane(&self) -> bool {
        self.tree.borrow().is_leaf()
    }

    pub(crate) fn is_zoomed_pane(&self, target: &gtk::Widget) -> bool {
        self.zoomed_pane
            .borrow()
            .as_ref()
            .map(|pane| pane == target)
            .unwrap_or(false)
    }

    pub(crate) fn toggle_zoom(self: &Rc<Self>, target: &gtk::Widget) -> bool {
        if self.zoomed_pane.borrow().is_some() {
            self.restore_zoom();
            false
        } else {
            self.zoom_pane(target);
            true
        }
    }

    fn zoom_pane(self: &Rc<Self>, target: &gtk::Widget) {
        self.save_focus();
        *self.zoomed_pane.borrow_mut() = Some(target.clone());
        *self.last_focused.borrow_mut() = Some(target.clone());
        self.trigger_rebuild();
    }

    fn restore_zoom(self: &Rc<Self>) {
        self.save_focus();
        self.zoomed_pane.borrow_mut().take();
        self.trigger_rebuild();
    }

    /// Split a pane. Mutates the data model, then triggers async rebuild.
    pub(crate) fn can_split(&self, target: &gtk::Widget, orientation: gtk::Orientation) -> bool {
        pane_has_room_to_split(target, orientation)
    }

    pub(crate) fn split(
        self: &Rc<Self>,
        target: &gtk::Widget,
        new_pane: gtk::Widget,
        orientation: gtk::Orientation,
        new_pane_first: bool,
        ratio: f64,
    ) -> bool {
        if !pane_has_room_to_split(target, orientation) {
            return false;
        }

        self.save_focus();
        self.zoomed_pane.borrow_mut().take();
        *self.last_focused.borrow_mut() = Some(new_pane.clone());

        let shared_ratio = Rc::new(RefCell::new(layout_state::clamp_split_ratio(ratio)));
        let new_node = if new_pane_first {
            SplitNode::Split {
                orientation,
                ratio: shared_ratio,
                left: Box::new(SplitNode::Leaf {
                    pane_widget: new_pane,
                }),
                right: Box::new(SplitNode::Leaf {
                    pane_widget: target.clone(),
                }),
            }
        } else {
            SplitNode::Split {
                orientation,
                ratio: shared_ratio,
                left: Box::new(SplitNode::Leaf {
                    pane_widget: target.clone(),
                }),
                right: Box::new(SplitNode::Leaf {
                    pane_widget: new_pane,
                }),
            }
        };

        let replaced = {
            let mut tree = self.tree.borrow_mut();
            tree.replace(target, new_node)
        };

        if replaced {
            self.trigger_rebuild();
        }
        replaced
    }

    /// Remove a pane. Mutates the data model, then triggers async rebuild.
    pub(crate) fn remove(self: &Rc<Self>, target: &gtk::Widget) -> bool {
        self.save_focus();
        self.zoomed_pane.borrow_mut().take();

        let removed = {
            let mut tree = self.tree.borrow_mut();
            tree.remove(target)
        };

        if removed {
            self.trigger_rebuild();
        }
        removed
    }

    pub(crate) fn rebuild_for_pane_metadata(self: &Rc<Self>, target: &gtk::Widget) {
        if !self.tree.borrow().contains_pane(target) {
            return;
        }
        self.save_focus();
        self.trigger_rebuild();
    }

    /// Tear down the old widget tree and schedule a rebuild on the next idle
    /// tick. The one-tick separation between unrealize (teardown) and realize
    /// (rebuild) is what prevents GLArea breakage.
    fn trigger_rebuild(self: &Rc<Self>) {
        // Cancel any pending rebuild
        if let Some(source) = self.rebuild_source.take() {
            source.remove();
        }

        // Clear the bin — tears down the old widget tree.
        // unrealize cascades to all GLAreas in the subtree.
        while let Some(child) = self.bin.first_child() {
            self.bin.remove(&child);
        }

        // Rebuild on the next idle tick. The tick separation between
        // unrealize (above) and realize (rebuild) is critical.
        self.schedule_rebuild();
    }

    /// Schedule the actual rebuild on the next idle tick.
    fn schedule_rebuild(self: &Rc<Self>) {
        if self.rebuild_source.borrow().is_some() {
            return;
        }
        let container = Rc::clone(self);
        let source = glib::idle_add_local_once(move || {
            container.rebuild_source.replace(None);
            container.do_rebuild();
        });
        self.rebuild_source.replace(Some(source));
    }

    /// Build new widget tree from data model, attach atomically.
    fn do_rebuild(self: &Rc<Self>) {
        // Pane widgets may still be parented to old (floating) Paneds from
        // the previous tree. GTK4 won't let us add them to new containers
        // until they're unparented. Detach them all first.
        let tree = self.tree.borrow();
        let zoomed = self.zoomed_pane.borrow().clone();
        if let Some(pane) = zoomed {
            if pane.parent().is_some() {
                detach_pane_from_old_parent(&pane);
                self.schedule_rebuild();
                return;
            }
            self.bin.append(&pane);
        } else {
            if tree_has_pane_parents(&tree) {
                detach_panes_from_old_tree(&tree);
                self.schedule_rebuild();
                return;
            }
            let widget = build_widget_tree(&tree, &self.state);
            self.bin.append(&widget);
        }
        refresh_terminal_displays_after_rebuild(self.bin.upcast_ref());

        // Newly created panes are tracked as pane containers rather than the
        // inner terminal/browser widget, so restore through the pane helper
        // when possible and fall back to plain widget focus otherwise.
        if let Some(focused) = self.last_focused.borrow().as_ref() {
            if !pane::focus_active_tab_in_pane(focused) {
                focused.grab_focus();
            }
        }
    }

    fn save_focus(&self) {
        let focus = self
            .bin
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok())
            .and_then(|w| gtk::prelude::GtkWindowExt::focus(&w));
        *self.last_focused.borrow_mut() = focus;
    }
}

impl Drop for SplitTreeContainer {
    fn drop(&mut self) {
        if let Some(source) = self.rebuild_source.take() {
            source.remove();
        }
    }
}

// ---------------------------------------------------------------------------
// Widget tree helpers
// ---------------------------------------------------------------------------

/// Detach pane widgets from their old parents (floating Paneds left over
/// from the previous widget tree). GTK4 requires a widget to have no parent
/// before it can be added to a new container.
fn detach_panes_from_old_tree(node: &SplitNode) {
    match node {
        SplitNode::Leaf { pane_widget } => {
            if let Some(parent) = pane_widget.parent() {
                if let Some(paned) = parent.downcast_ref::<gtk::Paned>() {
                    // Detach from the old Paned by clearing whichever slot holds us
                    if paned
                        .start_child()
                        .map(|c| c == *pane_widget)
                        .unwrap_or(false)
                    {
                        paned.set_start_child(gtk::Widget::NONE);
                    } else {
                        paned.set_end_child(gtk::Widget::NONE);
                    }
                }
            }
        }
        SplitNode::Split { left, right, .. } => {
            detach_panes_from_old_tree(left);
            detach_panes_from_old_tree(right);
        }
    }
}

fn tree_has_pane_parents(node: &SplitNode) -> bool {
    match node {
        SplitNode::Leaf { pane_widget } => pane_widget.parent().is_some(),
        SplitNode::Split { left, right, .. } => {
            tree_has_pane_parents(left) || tree_has_pane_parents(right)
        }
    }
}

fn detach_pane_from_old_parent(pane_widget: &gtk::Widget) {
    if let Some(parent) = pane_widget.parent() {
        if let Some(paned) = parent.downcast_ref::<gtk::Paned>() {
            if paned
                .start_child()
                .map(|child| child == *pane_widget)
                .unwrap_or(false)
            {
                paned.set_start_child(gtk::Widget::NONE);
            } else {
                paned.set_end_child(gtk::Widget::NONE);
            }
        } else if let Some(container) = parent.downcast_ref::<gtk::Box>() {
            container.remove(pane_widget);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WidthLockSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WidthLockConstraints {
    start_required_width: Option<i32>,
    end_required_width: Option<i32>,
    start_min_width: i32,
    end_min_width: i32,
}

impl WidthLockConstraints {
    fn has_required_width(self) -> bool {
        self.start_required_width.is_some() || self.end_required_width.is_some()
    }
}

fn locked_width_position_for_horizontal_split(
    side: WidthLockSide,
    locked_width: i32,
    total_width: i32,
    min_child_width: i32,
) -> Option<i32> {
    if locked_width <= 0 || total_width < min_child_width.saturating_mul(2) {
        return None;
    }

    let max_child_width = total_width.saturating_sub(min_child_width);
    let locked_width = locked_width.clamp(min_child_width, max_child_width);
    Some(match side {
        WidthLockSide::Start => locked_width,
        WidthLockSide::End => total_width - locked_width,
    })
}

fn paned_child_area_extent(
    start_extent: i32,
    end_extent: i32,
    total_extent: i32,
    min_child_extent: i32,
) -> i32 {
    let child_extent = start_extent.saturating_add(end_extent);
    if start_extent > 0
        && end_extent > 0
        && child_extent <= total_extent
        && child_extent >= min_child_extent.saturating_mul(2)
    {
        child_extent
    } else {
        total_extent
    }
}

fn horizontal_paned_child_area_width(paned: &gtk::Paned) -> i32 {
    let allocation_width = paned.allocation().width();
    let start_width = paned
        .start_child()
        .map(|child| child.allocation().width())
        .unwrap_or_default();
    let end_width = paned
        .end_child()
        .map(|child| child.allocation().width())
        .unwrap_or_default();
    paned_child_area_extent(
        start_width,
        end_width,
        allocation_width,
        pane::MIN_PANE_WIDTH,
    )
}

fn minimum_width_for_subtree(node: &SplitNode) -> i32 {
    match node {
        SplitNode::Leaf { .. } => pane::MIN_PANE_WIDTH,
        SplitNode::Split {
            orientation,
            left,
            right,
            ..
        } => match *orientation {
            gtk::Orientation::Horizontal => {
                minimum_width_for_subtree(left).saturating_add(minimum_width_for_subtree(right))
            }
            gtk::Orientation::Vertical => {
                minimum_width_for_subtree(left).max(minimum_width_for_subtree(right))
            }
            _ => minimum_width_for_subtree(left).max(minimum_width_for_subtree(right)),
        },
    }
}

fn horizontal_extent_width_lock_panes(node: &SplitNode) -> Vec<gtk::Widget> {
    match node {
        SplitNode::Leaf { pane_widget } => vec![pane_widget.clone()],
        SplitNode::Split {
            orientation,
            left,
            right,
            ..
        } => {
            if *orientation == gtk::Orientation::Horizontal {
                Vec::new()
            } else {
                let mut panes = horizontal_extent_width_lock_panes(left);
                panes.extend(horizontal_extent_width_lock_panes(right));
                panes
            }
        }
    }
}

fn locked_width_required_for_subtree(node: &SplitNode) -> Option<i32> {
    match node {
        SplitNode::Leaf { pane_widget } => {
            pane::pane_locked_width(pane_widget).map(|width| width.max(pane::MIN_PANE_WIDTH))
        }
        SplitNode::Split {
            orientation,
            left,
            right,
            ..
        } => {
            let left_required = locked_width_required_for_subtree(left);
            let right_required = locked_width_required_for_subtree(right);

            match *orientation {
                gtk::Orientation::Horizontal => {
                    if left_required.is_none() && right_required.is_none() {
                        None
                    } else {
                        Some(
                            left_required.unwrap_or_else(|| minimum_width_for_subtree(left))
                                + right_required
                                    .unwrap_or_else(|| minimum_width_for_subtree(right)),
                        )
                    }
                }
                gtk::Orientation::Vertical => {
                    if left_required.is_none() && right_required.is_none() {
                        None
                    } else {
                        Some(
                            left_required
                                .unwrap_or_else(|| minimum_width_for_subtree(left))
                                .max(
                                    right_required
                                        .unwrap_or_else(|| minimum_width_for_subtree(right)),
                                ),
                        )
                    }
                }
                _ => {
                    if left_required.is_none() && right_required.is_none() {
                        None
                    } else {
                        Some(
                            left_required
                                .unwrap_or_else(|| minimum_width_for_subtree(left))
                                .max(
                                    right_required
                                        .unwrap_or_else(|| minimum_width_for_subtree(right)),
                                ),
                        )
                    }
                }
            }
        }
    }
}

fn first_locked_width(panes: &[gtk::Widget]) -> Option<i32> {
    panes.iter().find_map(pane::pane_locked_width)
}

fn locked_width_position_from_panes(
    start_lock_panes: &[gtk::Widget],
    end_lock_panes: &[gtk::Widget],
    total_width: i32,
    min_child_width: i32,
) -> Option<i32> {
    first_locked_width(start_lock_panes)
        .and_then(|width| {
            locked_width_position_for_horizontal_split(
                WidthLockSide::Start,
                width,
                total_width,
                min_child_width,
            )
        })
        .or_else(|| {
            first_locked_width(end_lock_panes).and_then(|width| {
                locked_width_position_for_horizontal_split(
                    WidthLockSide::End,
                    width,
                    total_width,
                    min_child_width,
                )
            })
        })
}

fn constrained_locked_width_position(
    current_position: i32,
    exact_position: Option<i32>,
    constraints: WidthLockConstraints,
    total_width: i32,
    min_child_width: i32,
) -> Option<i32> {
    if total_width < min_child_width.saturating_mul(2) {
        return None;
    }

    let max_child_width = total_width.saturating_sub(min_child_width);
    let min_position = constraints
        .start_required_width
        .unwrap_or(constraints.start_min_width)
        .clamp(min_child_width, max_child_width);
    let max_position = total_width
        .saturating_sub(
            constraints
                .end_required_width
                .unwrap_or(constraints.end_min_width),
        )
        .clamp(min_child_width, max_child_width);
    let desired_position = exact_position.unwrap_or(current_position);

    if min_position <= max_position {
        Some(desired_position.clamp(min_position, max_position))
    } else {
        Some(desired_position.clamp(max_position, min_position))
    }
}

fn apply_locked_width_position_from_panes(
    paned: &gtk::Paned,
    start_lock_panes: &[gtk::Widget],
    end_lock_panes: &[gtk::Widget],
    constraints: WidthLockConstraints,
    applying: &Rc<Cell<bool>>,
) -> bool {
    let has_active_lock = first_locked_width(start_lock_panes).is_some()
        || first_locked_width(end_lock_panes).is_some()
        || constraints.has_required_width();
    if !has_active_lock {
        return false;
    }

    let child_area_width = horizontal_paned_child_area_width(paned);
    let exact_position = locked_width_position_from_panes(
        start_lock_panes,
        end_lock_panes,
        child_area_width,
        pane::MIN_PANE_WIDTH,
    );
    let Some(position) = constrained_locked_width_position(
        paned.position(),
        exact_position,
        constraints,
        child_area_width,
        pane::MIN_PANE_WIDTH,
    ) else {
        return true;
    };

    if paned.position() != position {
        applying.set(true);
        paned.set_position(position);
        applying.set(false);
    }
    true
}

fn attach_locked_width_enforcement(
    paned: &gtk::Paned,
    orientation: gtk::Orientation,
    start_lock_panes: Vec<gtk::Widget>,
    end_lock_panes: Vec<gtk::Widget>,
    constraints: WidthLockConstraints,
    applying: Rc<Cell<bool>>,
) {
    if orientation != gtk::Orientation::Horizontal {
        return;
    }
    if start_lock_panes.is_empty() && end_lock_panes.is_empty() && !constraints.has_required_width()
    {
        return;
    }

    {
        let paned = paned.clone();
        let start_lock_panes = start_lock_panes.clone();
        let end_lock_panes = end_lock_panes.clone();
        let applying = applying.clone();
        glib::idle_add_local_once(move || {
            apply_locked_width_position_from_panes(
                &paned,
                &start_lock_panes,
                &end_lock_panes,
                constraints,
                &applying,
            );
        });
    }

    {
        let start_lock_panes = start_lock_panes.clone();
        let end_lock_panes = end_lock_panes.clone();
        let applying = applying.clone();
        paned.connect_map(move |paned| {
            apply_locked_width_position_from_panes(
                paned,
                &start_lock_panes,
                &end_lock_panes,
                constraints,
                &applying,
            );
        });
    }

    glib::timeout_add_local_once(std::time::Duration::from_millis(16), {
        let paned = paned.clone();
        let start_lock_panes = start_lock_panes.clone();
        let end_lock_panes = end_lock_panes.clone();
        let applying = applying.clone();
        move || {
            apply_locked_width_position_from_panes(
                &paned,
                &start_lock_panes,
                &end_lock_panes,
                constraints,
                &applying,
            );
        }
    });

    glib::timeout_add_local_once(std::time::Duration::from_millis(80), {
        let paned = paned.clone();
        move || {
            apply_locked_width_position_from_panes(
                &paned,
                &start_lock_panes,
                &end_lock_panes,
                constraints,
                &applying,
            );
        }
    });
}

fn snapshot_current_split_ratio(paned: &gtk::Paned, shared_ratio: &Rc<RefCell<f64>>) {
    let allocation = paned.allocation();
    let orientation = paned.orientation();
    let size = if orientation == gtk::Orientation::Horizontal {
        allocation.width()
    } else {
        allocation.height()
    };
    let new_ratio = layout_state::snapshot_split_ratio_with_min(
        paned.position(),
        size,
        Some(*shared_ratio.borrow()),
        minimum_split_extent_for_orientation(orientation),
    );
    *shared_ratio.borrow_mut() = layout_state::clamp_split_ratio(new_ratio);
}

/// Build a GTK widget tree from the SplitNode data model.
fn build_widget_tree(node: &SplitNode, state: &State) -> gtk::Widget {
    match node {
        SplitNode::Leaf { pane_widget } => pane_widget.clone(),
        SplitNode::Split {
            orientation,
            ratio,
            left,
            right,
        } => {
            let paned = gtk::Paned::builder()
                .orientation(*orientation)
                .hexpand(true)
                .vexpand(true)
                .build();
            paned.set_shrink_start_child(false);
            paned.set_shrink_end_child(false);
            paned.set_resize_start_child(true);
            paned.set_resize_end_child(true);

            // Flag to suppress position_notify during programmatic set_position calls
            // (initial layout and workspace re-map). Without this, set_position triggers
            // position_notify which recalculates the ratio from the not-yet-stable pixel
            // position, corrupting the stored ratio.
            let applying = Rc::new(Cell::new(false));

            let ratio_val = *ratio.borrow();
            update_split_ratio_state(&paned, ratio_val);
            attach_split_position_persistence(state, &paned, applying.clone());

            let start_lock_panes = horizontal_extent_width_lock_panes(left);
            let end_lock_panes = horizontal_extent_width_lock_panes(right);
            let start_required_width = locked_width_required_for_subtree(left);
            let end_required_width = locked_width_required_for_subtree(right);
            let width_lock_constraints = WidthLockConstraints {
                start_required_width,
                end_required_width,
                start_min_width: minimum_width_for_subtree(left),
                end_min_width: minimum_width_for_subtree(right),
            };

            // Wire resize drags back to the shared ratio cell in the data model.
            let shared_ratio = ratio.clone();
            let applying_for_notify = applying.clone();
            let start_lock_panes_for_notify = start_lock_panes.clone();
            let end_lock_panes_for_notify = end_lock_panes.clone();
            paned.connect_position_notify(move |paned| {
                if applying_for_notify.get() {
                    return;
                }
                if paned.orientation() == gtk::Orientation::Horizontal
                    && apply_locked_width_position_from_panes(
                        paned,
                        &start_lock_panes_for_notify,
                        &end_lock_panes_for_notify,
                        width_lock_constraints,
                        &applying_for_notify,
                    )
                {
                    snapshot_current_split_ratio(paned, &shared_ratio);
                    return;
                }
                snapshot_current_split_ratio(paned, &shared_ratio);
            });

            let left_widget = build_widget_tree(left, state);
            let right_widget = build_widget_tree(right, state);
            paned.set_start_child(Some(&left_widget));
            paned.set_end_child(Some(&right_widget));

            apply_split_ratio_after_layout(&paned, *orientation, ratio.clone(), applying.clone());
            attach_locked_width_enforcement(
                &paned,
                *orientation,
                start_lock_panes,
                end_lock_panes,
                width_lock_constraints,
                applying,
            );

            paned.upcast()
        }
    }
}

fn pane_has_room_to_split(target: &gtk::Widget, orientation: gtk::Orientation) -> bool {
    let allocation = target.allocation();
    let size = if orientation == gtk::Orientation::Horizontal {
        allocation.width()
    } else {
        allocation.height()
    };
    size <= 0 || split_extent_has_room(size, orientation)
}

fn minimum_split_extent(orientation: gtk::Orientation) -> i32 {
    if orientation == gtk::Orientation::Horizontal {
        pane::MIN_PANE_WIDTH
    } else {
        pane::MIN_PANE_HEIGHT
    }
}

fn split_extent_has_room(size: i32, orientation: gtk::Orientation) -> bool {
    size >= minimum_split_extent(orientation) * 2
}

fn refresh_terminal_displays_after_rebuild(root: &gtk::Widget) {
    pane::refresh_terminal_displays_in_root(root);

    let idle_root = root.clone();
    glib::idle_add_local_once(move || {
        pane::refresh_terminal_displays_in_root(&idle_root);
    });

    let first_frame_root = root.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(16), move || {
        pane::refresh_terminal_displays_in_root(&first_frame_root);
    });

    let settled_root = root.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
        pane::refresh_terminal_displays_in_root(&settled_root);
    });
}

// ---------------------------------------------------------------------------
// Conversion from serialized LayoutNodeState to runtime SplitNode
// ---------------------------------------------------------------------------

/// Build a SplitNode tree from a persisted LayoutNodeState.
pub(crate) fn build_split_node_from_layout(
    state: &State,
    shortcuts: &Rc<crate::shortcut_config::ResolvedShortcutConfig>,
    ws_id: &str,
    working_directory: Option<&str>,
    layout: &LayoutNodeState,
) -> SplitNode {
    match layout {
        LayoutNodeState::Pane(pane_state) => {
            let pane = crate::window::create_pane_for_workspace(
                state,
                shortcuts,
                ws_id,
                working_directory,
                Some(pane_state),
                false,
            );
            SplitNode::Leaf {
                pane_widget: pane.upcast(),
            }
        }
        LayoutNodeState::Split(split_state) => {
            let orientation = match split_state.orientation {
                SplitOrientation::Horizontal => gtk::Orientation::Horizontal,
                SplitOrientation::Vertical => gtk::Orientation::Vertical,
            };
            SplitNode::Split {
                orientation,
                ratio: Rc::new(RefCell::new(layout_state::clamp_split_ratio(
                    split_state.ratio,
                ))),
                left: Box::new(build_split_node_from_layout(
                    state,
                    shortcuts,
                    ws_id,
                    working_directory,
                    &split_state.start,
                )),
                right: Box::new(build_split_node_from_layout(
                    state,
                    shortcuts,
                    ws_id,
                    working_directory,
                    &split_state.end,
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn width_lock_constraints(
        start_required_width: Option<i32>,
        end_required_width: Option<i32>,
        start_min_width: i32,
        end_min_width: i32,
    ) -> WidthLockConstraints {
        WidthLockConstraints {
            start_required_width,
            end_required_width,
            start_min_width,
            end_min_width,
        }
    }

    #[test]
    fn locked_width_position_preserves_start_child_width() {
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::Start, 420, 1_200, 260),
            Some(420)
        );
    }

    #[test]
    fn locked_width_position_preserves_end_child_width() {
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::End, 380, 1_200, 260),
            Some(820)
        );
    }

    #[test]
    fn locked_width_position_respects_minimum_child_widths() {
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::Start, 100, 1_200, 260),
            Some(260)
        );
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::End, 100, 1_200, 260),
            Some(940)
        );
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::Start, 600, 400, 260),
            None
        );
    }

    #[test]
    fn paned_child_area_extent_excludes_handle_when_children_are_allocated() {
        assert_eq!(paned_child_area_extent(420, 372, 800, 260), 792);
    }

    #[test]
    fn paned_child_area_extent_falls_back_before_layout() {
        assert_eq!(paned_child_area_extent(0, 0, 800, 260), 800);
        assert_eq!(paned_child_area_extent(240, 240, 800, 260), 800);
    }

    #[test]
    fn constrained_locked_width_position_preserves_direct_exact_lock() {
        assert_eq!(
            constrained_locked_width_position(
                800,
                Some(420),
                width_lock_constraints(Some(420), None, 260, 260),
                1_200,
                260
            ),
            Some(420)
        );
    }

    #[test]
    fn constrained_locked_width_position_preserves_nested_start_requirement() {
        assert_eq!(
            constrained_locked_width_position(
                500,
                None,
                width_lock_constraints(Some(680), None, 260, 260),
                1_200,
                260
            ),
            Some(680)
        );
        assert_eq!(
            constrained_locked_width_position(
                760,
                None,
                width_lock_constraints(Some(680), None, 260, 260),
                1_200,
                260
            ),
            Some(760)
        );
    }

    #[test]
    fn constrained_locked_width_position_preserves_nested_end_requirement() {
        assert_eq!(
            constrained_locked_width_position(
                900,
                None,
                width_lock_constraints(None, Some(520), 260, 260),
                1_200,
                260
            ),
            Some(680)
        );
        assert_eq!(
            constrained_locked_width_position(
                600,
                None,
                width_lock_constraints(None, Some(520), 260, 260),
                1_200,
                260
            ),
            Some(600)
        );
    }

    #[test]
    fn constrained_locked_width_position_uses_child_area_for_end_locks() {
        assert_eq!(
            locked_width_position_for_horizontal_split(WidthLockSide::End, 380, 1_192, 260),
            Some(812)
        );
        assert_eq!(
            constrained_locked_width_position(
                900,
                None,
                width_lock_constraints(None, Some(520), 260, 260),
                1_192,
                260
            ),
            Some(672)
        );
    }

    #[test]
    fn constrained_locked_width_position_reserves_structural_sibling_minimums() {
        assert_eq!(
            constrained_locked_width_position(
                760,
                None,
                width_lock_constraints(Some(680), None, 680, 520),
                1_200,
                260
            ),
            Some(680)
        );
        assert_eq!(
            constrained_locked_width_position(
                500,
                None,
                width_lock_constraints(None, Some(680), 520, 680),
                1_200,
                260
            ),
            Some(520)
        );
    }

    #[test]
    fn split_extent_requires_room_for_both_children() {
        assert!(!split_extent_has_room(
            pane::MIN_PANE_WIDTH * 2 - 1,
            gtk::Orientation::Horizontal
        ));
        assert!(split_extent_has_room(
            pane::MIN_PANE_WIDTH * 2,
            gtk::Orientation::Horizontal
        ));
        assert!(!split_extent_has_room(
            pane::MIN_PANE_HEIGHT * 2 - 1,
            gtk::Orientation::Vertical
        ));
        assert!(split_extent_has_room(
            pane::MIN_PANE_HEIGHT * 2,
            gtk::Orientation::Vertical
        ));
    }
}
