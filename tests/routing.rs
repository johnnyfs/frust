use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use frust::tui::{
    EventResult, FocusState, InputPolicy, Layer, MouseButton, MouseEvent, MouseKind, Point,
    UiEvent, View, ViewId, ViewNode, ViewTree, route_event,
};
use ratatui::{Frame, layout::Rect};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    View(&'static str),
}

struct TestView {
    id: ViewId,
    policy: InputPolicy,
    layer: Layer,
    z: i32,
    result: EventResult<Msg>,
}

impl TestView {
    fn new(id: &'static str, policy: InputPolicy, result: EventResult<Msg>) -> Self {
        Self {
            id: ViewId::new(id),
            policy,
            layer: Layer::Base,
            z: 0,
            result,
        }
    }

    fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }
}

impl View<(), Msg> for TestView {
    fn id(&self) -> ViewId {
        self.id.clone()
    }

    fn input_policy(&self) -> InputPolicy {
        self.policy
    }

    fn layer(&self) -> Layer {
        self.layer
    }

    fn z_offset(&self) -> i32 {
        self.z
    }

    fn render(&self, _frame: &mut Frame<'_>, _area: Rect, _state: &()) {}

    fn handle_event(
        &self,
        _event: &UiEvent,
        _area: Rect,
        _state: &(),
        _focus: &FocusState,
    ) -> EventResult<Msg> {
        self.result.clone()
    }
}

fn mouse(kind: MouseKind, x: u16, y: u16) -> UiEvent {
    UiEvent::Mouse(MouseEvent {
        position: Point::new(x, y),
        kind,
        button: Some(MouseButton::Left),
        modifiers: KeyModifiers::NONE,
    })
}

fn key(ch: char) -> UiEvent {
    UiEvent::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))
}

#[test]
fn hit_testing_overlapping_views_topmost_wins_and_outside_ignored() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 20, 10))
            .child(ViewNode::new(
                TestView::new(
                    "base",
                    InputPolicy::HitTest,
                    EventResult::message(Msg::View("base")),
                ),
                Rect::new(0, 0, 10, 5),
            ))
            .child(ViewNode::new(
                TestView::new(
                    "overlay",
                    InputPolicy::HitTest,
                    EventResult::message(Msg::View("overlay")),
                )
                .layer(Layer::Overlay),
                Rect::new(2, 1, 10, 5),
            )),
    );

    let outcome = route_event(
        &mouse(MouseKind::Down, 3, 2),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert_eq!(outcome.messages, vec![Msg::View("overlay")]);

    let outcome = route_event(
        &mouse(MouseKind::Down, 19, 9),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn ignored_mouse_event_falls_through_to_lower_hit() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 20, 10))
            .child(ViewNode::new(
                TestView::new(
                    "lower",
                    InputPolicy::HitTest,
                    EventResult::message(Msg::View("lower")),
                ),
                Rect::new(0, 0, 10, 5),
            ))
            .child(ViewNode::new(
                TestView::new("upper", InputPolicy::HitTest, EventResult::Ignored)
                    .layer(Layer::Overlay),
                Rect::new(0, 0, 10, 5),
            )),
    );

    let outcome = route_event(
        &mouse(MouseKind::Down, 1, 1),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert_eq!(outcome.messages, vec![Msg::View("lower")]);
}

#[test]
fn keyboard_routes_only_to_focused_view_and_mouse_down_can_focus() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 20, 10))
            .child(ViewNode::new(
                TestView::new(
                    "a",
                    InputPolicy::Focusable,
                    EventResult::message(Msg::View("a")),
                ),
                Rect::new(0, 0, 5, 5),
            ))
            .child(ViewNode::new(
                TestView::new(
                    "b",
                    InputPolicy::Focusable,
                    EventResult::message(Msg::View("b")),
                ),
                Rect::new(5, 0, 5, 5),
            )),
    );

    let focus = FocusState {
        keyboard_focus: Some(ViewId::new("b")),
        ..FocusState::default()
    };
    let outcome = route_event(&key('x'), &tree, &(), &focus);
    assert_eq!(outcome.messages, vec![Msg::View("b")]);

    let outcome = route_event(
        &mouse(MouseKind::Down, 1, 1),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert_eq!(
        outcome.focus_update.keyboard_focus,
        Some(Some(ViewId::new("a")))
    );
}

#[test]
fn modal_capture_all_receives_events_outside_and_blocks_underlying() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 30, 10))
            .child(ViewNode::new(
                TestView::new(
                    "under",
                    InputPolicy::HitTest,
                    EventResult::message(Msg::View("under")),
                ),
                Rect::new(0, 0, 30, 10),
            ))
            .child(ViewNode::new(
                TestView::new(
                    "modal",
                    InputPolicy::CaptureAll,
                    EventResult::message(Msg::View("modal")),
                )
                .layer(Layer::Modal),
                Rect::new(10, 2, 8, 4),
            )),
    );
    let focus = FocusState {
        active_modal: Some(ViewId::new("modal")),
        ..FocusState::default()
    };

    let outside = route_event(&mouse(MouseKind::Down, 1, 1), &tree, &(), &focus);
    assert_eq!(outside.messages, vec![Msg::View("modal")]);

    let keyed = route_event(&key('q'), &tree, &(), &focus);
    assert_eq!(keyed.messages, vec![Msg::View("modal")]);
}

#[test]
fn mouse_capture_routes_drag_outside_and_releases_on_up() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 30, 10)).child(ViewNode::new(
            TestView::new(
                "drag",
                InputPolicy::CaptureMouse,
                EventResult::message(Msg::View("drag")),
            ),
            Rect::new(0, 0, 5, 5),
        )),
    );

    let down = route_event(
        &mouse(MouseKind::Down, 1, 1),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert_eq!(
        down.focus_update.mouse_capture,
        Some(Some(ViewId::new("drag")))
    );

    let focus = FocusState::default().apply(&down.focus_update);
    let drag = route_event(&mouse(MouseKind::Drag, 20, 8), &tree, &(), &focus);
    assert_eq!(drag.messages, vec![Msg::View("drag")]);

    let up = route_event(&mouse(MouseKind::Up, 20, 8), &tree, &(), &focus);
    assert_eq!(up.messages, vec![Msg::View("drag")]);
    assert_eq!(up.focus_update.mouse_capture, Some(None));
}

#[test]
fn child_can_bubble_to_parent() {
    let tree = ViewTree::new(
        frust::tui::root(Rect::new(0, 0, 20, 10)).child(
            ViewNode::new(
                TestView::new(
                    "parent",
                    InputPolicy::HitTest,
                    EventResult::message(Msg::View("parent")),
                ),
                Rect::new(0, 0, 10, 5),
            )
            .child(ViewNode::new(
                TestView::new("child", InputPolicy::HitTest, EventResult::Bubble),
                Rect::new(1, 1, 5, 3),
            )),
        ),
    );

    let outcome = route_event(
        &mouse(MouseKind::Down, 2, 2),
        &tree,
        &(),
        &FocusState::default(),
    );
    assert_eq!(outcome.messages, vec![Msg::View("parent")]);
}
