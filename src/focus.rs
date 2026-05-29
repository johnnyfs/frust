//! Focus and capture state.

use crate::ViewId;

/// Routing state owned by the application or runtime shell.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusState {
    /// Current keyboard focus.
    pub keyboard_focus: Option<ViewId>,
    /// Current mouse capture target.
    pub mouse_capture: Option<ViewId>,
    /// Current hovered view.
    pub hovered: Option<ViewId>,
    /// Active modal/capture-all view.
    pub active_modal: Option<ViewId>,
}

impl FocusState {
    /// Applies a focus update and returns the resulting state.
    pub fn apply(&self, update: &FocusUpdate) -> Self {
        let mut next = self.clone();
        if let Some(value) = &update.keyboard_focus {
            next.keyboard_focus = value.clone();
        }
        if let Some(value) = &update.mouse_capture {
            next.mouse_capture = value.clone();
        }
        if let Some(value) = &update.hovered {
            next.hovered = value.clone();
        }
        if let Some(value) = &update.active_modal {
            next.active_modal = value.clone();
        }
        next
    }
}

/// Explicit focus/capture mutations produced by routing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusUpdate {
    /// `None` means unchanged. `Some(None)` means clear.
    pub keyboard_focus: Option<Option<ViewId>>,
    /// `None` means unchanged. `Some(None)` means clear.
    pub mouse_capture: Option<Option<ViewId>>,
    /// `None` means unchanged. `Some(None)` means clear.
    pub hovered: Option<Option<ViewId>>,
    /// `None` means unchanged. `Some(None)` means clear.
    pub active_modal: Option<Option<ViewId>>,
}

impl FocusUpdate {
    /// Focuses a view for keyboard events.
    pub fn focus_keyboard(&mut self, id: impl Into<ViewId>) {
        self.keyboard_focus = Some(Some(id.into()));
    }

    /// Clears keyboard focus.
    pub fn clear_keyboard_focus(&mut self) {
        self.keyboard_focus = Some(None);
    }

    /// Captures mouse events for a view.
    pub fn capture_mouse(&mut self, id: impl Into<ViewId>) {
        self.mouse_capture = Some(Some(id.into()));
    }

    /// Releases mouse capture.
    pub fn release_mouse(&mut self) {
        self.mouse_capture = Some(None);
    }

    /// Sets hovered view.
    pub fn hover(&mut self, id: Option<ViewId>) {
        self.hovered = Some(id);
    }

    /// Sets the active modal.
    pub fn set_active_modal(&mut self, id: impl Into<ViewId>) {
        self.active_modal = Some(Some(id.into()));
    }

    /// Clears the active modal.
    pub fn clear_active_modal(&mut self) {
        self.active_modal = Some(None);
    }

    /// Merges another update, preferring explicitly set fields from `other`.
    pub fn merge(&mut self, other: FocusUpdate) {
        if other.keyboard_focus.is_some() {
            self.keyboard_focus = other.keyboard_focus;
        }
        if other.mouse_capture.is_some() {
            self.mouse_capture = other.mouse_capture;
        }
        if other.hovered.is_some() {
            self.hovered = other.hovered;
        }
        if other.active_modal.is_some() {
            self.active_modal = other.active_modal;
        }
    }
}
