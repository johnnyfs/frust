use frust::tui::{FocusState, FocusUpdate, ViewId};

#[test]
fn focus_update_applies_only_explicit_fields() {
    let focus = FocusState {
        keyboard_focus: Some(ViewId::new("old-key")),
        mouse_capture: Some(ViewId::new("old-mouse")),
        hovered: None,
        active_modal: Some(ViewId::new("modal")),
    };
    let mut update = FocusUpdate::default();
    update.focus_keyboard("new-key");
    update.release_mouse();

    let next = focus.apply(&update);
    assert_eq!(next.keyboard_focus, Some(ViewId::new("new-key")));
    assert_eq!(next.mouse_capture, None);
    assert_eq!(next.active_modal, Some(ViewId::new("modal")));
}
