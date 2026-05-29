//! Deterministic input routing and hit testing.

use crate::tui::{
    EventResult, FocusState, FocusUpdate, InputPolicy, MouseKind, NodeRef, UiEvent, ViewId,
    ViewTree,
};

/// Messages and focus mutations produced by routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteOutcome<M> {
    /// Application messages emitted by views.
    pub messages: Vec<M>,
    /// Focus/capture mutations inferred by the router.
    pub focus_update: FocusUpdate,
}

impl<M> Default for RouteOutcome<M> {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            focus_update: FocusUpdate::default(),
        }
    }
}

/// Deterministic event router.
#[derive(Debug, Clone)]
pub struct Router {
    /// Allows bubbled events to walk parent nodes.
    pub bubble: bool,
    /// Allows ignored mouse events to fall through to lower hit-tested nodes.
    pub fallthrough_on_ignored: bool,
    /// Offers otherwise unhandled events to the root node.
    pub route_unhandled_to_root: bool,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            bubble: true,
            fallthrough_on_ignored: true,
            route_unhandled_to_root: false,
        }
    }
}

/// Routes an event with the default router.
pub fn route_event<S: 'static, M: 'static>(
    event: &UiEvent,
    tree: &ViewTree<S, M>,
    state: &S,
    focus: &FocusState,
) -> RouteOutcome<M> {
    Router::default().route_event(event, tree, state, focus)
}

impl Router {
    /// Routes an event through the tree.
    pub fn route_event<S: 'static, M: 'static>(
        &self,
        event: &UiEvent,
        tree: &ViewTree<S, M>,
        state: &S,
        focus: &FocusState,
    ) -> RouteOutcome<M> {
        let flat = tree.flatten();
        let mut outcome = RouteOutcome::default();

        if let Some(modal_id) = &focus.active_modal
            && let Some(modal) =
                find_node(&flat, modal_id).filter(|node| node.input_policy.captures_all())
        {
            self.offer_capture_target(event, state, focus, &flat, modal, &mut outcome);
            return outcome;
        }

        if let UiEvent::Mouse(mouse) = event {
            if mouse.kind == MouseKind::Move {
                let hovered = tree.hit_test(mouse.position).map(|node| node.id);
                outcome.focus_update.hover(hovered);
            }

            if mouse.kind.follows_capture()
                && let Some(capture_id) = &focus.mouse_capture
                && let Some(captured) = find_node(&flat, capture_id)
            {
                self.offer_capture_target(event, state, focus, &flat, captured, &mut outcome);
                if mouse.kind == MouseKind::Up {
                    outcome.focus_update.release_mouse();
                }
                return outcome;
            }
        }

        if matches!(event, UiEvent::Key(_))
            && let Some(focused_id) = &focus.keyboard_focus
            && let Some(focused) = find_node(&flat, focused_id)
        {
            self.offer_capture_target(event, state, focus, &flat, focused, &mut outcome);
            return outcome;
        }

        if let UiEvent::Mouse(mouse) = event {
            for node in tree.event_order().into_iter().filter(|node| {
                node.input_policy.can_hit_test() && mouse.position.is_inside(node.rect)
            }) {
                let result = self.offer_node(event, state, focus, &flat, node.clone());
                let handled = self.apply_result(result, &mut outcome);

                if mouse.kind == MouseKind::Down && handled {
                    infer_mouse_down_focus(node.input_policy, &node.id, &mut outcome.focus_update);
                }

                if handled || !self.fallthrough_on_ignored {
                    return outcome;
                }
            }
        }

        if self.route_unhandled_to_root
            && let Some(root) = flat.first().cloned()
        {
            self.offer_capture_target(event, state, focus, &flat, root, &mut outcome);
        }

        outcome
    }

    fn offer_capture_target<S: 'static, M: 'static>(
        &self,
        event: &UiEvent,
        state: &S,
        focus: &FocusState,
        flat: &[NodeRef<'_, S, M>],
        node: NodeRef<'_, S, M>,
        outcome: &mut RouteOutcome<M>,
    ) {
        let result = self.offer_node(event, state, focus, flat, node);
        self.apply_result(result, outcome);
    }

    fn offer_node<S: 'static, M: 'static>(
        &self,
        event: &UiEvent,
        state: &S,
        focus: &FocusState,
        flat: &[NodeRef<'_, S, M>],
        node: NodeRef<'_, S, M>,
    ) -> EventResult<M> {
        match node.view.handle_event(event, node.rect, state, focus) {
            EventResult::Bubble if self.bubble => self.bubble_from(event, state, focus, flat, node),
            other => other,
        }
    }

    fn bubble_from<S: 'static, M: 'static>(
        &self,
        event: &UiEvent,
        state: &S,
        focus: &FocusState,
        flat: &[NodeRef<'_, S, M>],
        node: NodeRef<'_, S, M>,
    ) -> EventResult<M> {
        let mut parent = node.parent;
        while let Some(index) = parent {
            let Some(parent_node) = flat.get(index).cloned() else {
                break;
            };
            match parent_node
                .view
                .handle_event(event, parent_node.rect, state, focus)
            {
                EventResult::Bubble => parent = parent_node.parent,
                other => return other,
            }
        }
        EventResult::Ignored
    }

    fn apply_result<M>(&self, result: EventResult<M>, outcome: &mut RouteOutcome<M>) -> bool {
        match result {
            EventResult::Handled(messages) => {
                outcome.messages.extend(messages);
                true
            }
            EventResult::Bubble => false,
            EventResult::Ignored => false,
        }
    }
}

fn find_node<'a, S, M>(flat: &[NodeRef<'a, S, M>], id: &ViewId) -> Option<NodeRef<'a, S, M>> {
    flat.iter().find(|node| &node.id == id).cloned()
}

fn infer_mouse_down_focus(policy: InputPolicy, id: &ViewId, update: &mut FocusUpdate) {
    if policy.can_focus() {
        update.focus_keyboard(id.clone());
    }
    if policy.captures_mouse() {
        update.capture_mouse(id.clone());
    }
}
