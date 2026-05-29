use frust::tui::{
    FocusState, FocusUpdate, InputPolicy, Layer, ViewId, ViewNode, ViewTree, route_event,
    widgets::{Modal, Panel},
};
use ratatui::layout::Rect;

#[derive(Default)]
struct AppState {
    confirm_open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    CloseConfirm,
}

fn compose(state: &AppState, area: Rect) -> ViewTree<AppState, Msg> {
    let modal_area = Modal::<Msg>::centered(area, 30, 7);
    ViewTree::new(
        frust::tui::root(area)
            .child(ViewNode::new(
                Panel::new("content")
                    .title("Content")
                    .input_policy(InputPolicy::HitTest),
                area,
            ))
            .modal_if(
                state.confirm_open,
                ViewNode::new(
                    Modal::new("confirm", "Commit the selected changes?")
                        .title("Confirm")
                        .close_message(Msg::CloseConfirm),
                    modal_area,
                ),
            )
            .overlay(ViewNode::new(
                Panel::new("context-menu")
                    .title("Menu")
                    .layer(Layer::Overlay)
                    .input_policy(InputPolicy::HitTest),
                Rect::new(4, 3, 18, 5),
            )),
    )
}

fn main() {
    let state = AppState { confirm_open: true };
    let tree = compose(&state, Rect::new(0, 0, 80, 24));
    let mut focus = FocusState::default();
    let mut update = FocusUpdate::default();
    update.set_active_modal(ViewId::new("confirm"));
    focus = focus.apply(&update);

    let outcome = route_event(&frust::tui::UiEvent::Tick, &tree, &state, &focus);
    assert!(outcome.messages.is_empty());
}
