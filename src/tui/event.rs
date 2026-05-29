//! Normalized Crossterm event types.

use crossterm::event::{
    Event, KeyEvent, KeyModifiers, MouseButton as CrosstermMouseButton,
    MouseEventKind as CrosstermMouseKind,
};

use crate::tui::Point;

/// Normalized UI event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    /// Keyboard event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize { width: u16, height: u16 },
    /// Synthetic application tick.
    Tick,
}

impl TryFrom<Event> for UiEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Key(key) => Ok(Self::Key(key)),
            Event::Mouse(mouse) => Ok(Self::Mouse(MouseEvent::from(mouse))),
            Event::Resize(width, height) => Ok(Self::Resize { width, height }),
            _ => Err(()),
        }
    }
}

/// Normalized mouse event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// Screen position.
    pub position: Point,
    /// Mouse event kind.
    pub kind: MouseKind,
    /// Button when applicable.
    pub button: Option<MouseButton>,
    /// Keyboard modifiers.
    pub modifiers: KeyModifiers,
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(value: crossterm::event::MouseEvent) -> Self {
        let (kind, button) = match value.kind {
            CrosstermMouseKind::Down(button) => (MouseKind::Down, Some(button.into())),
            CrosstermMouseKind::Up(button) => (MouseKind::Up, Some(button.into())),
            CrosstermMouseKind::Drag(button) => (MouseKind::Drag, Some(button.into())),
            CrosstermMouseKind::Moved => (MouseKind::Move, None),
            CrosstermMouseKind::ScrollDown => (MouseKind::ScrollDown, None),
            CrosstermMouseKind::ScrollUp => (MouseKind::ScrollUp, None),
            CrosstermMouseKind::ScrollLeft => (MouseKind::ScrollLeft, None),
            CrosstermMouseKind::ScrollRight => (MouseKind::ScrollRight, None),
        };

        Self {
            position: Point::new(value.column, value.row),
            kind,
            button,
            modifiers: value.modifiers,
        }
    }
}

/// Normalized mouse event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseKind {
    /// Button down.
    Down,
    /// Button up.
    Up,
    /// Pointer drag.
    Drag,
    /// Pointer move.
    Move,
    /// Wheel up.
    ScrollUp,
    /// Wheel down.
    ScrollDown,
    /// Wheel left.
    ScrollLeft,
    /// Wheel right.
    ScrollRight,
}

impl MouseKind {
    /// Returns true for drag, move, and up events eligible for capture routing.
    pub fn follows_capture(self) -> bool {
        matches!(self, Self::Drag | Self::Move | Self::Up)
    }
}

/// Normalized mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle button.
    Middle,
}

impl From<CrosstermMouseButton> for MouseButton {
    fn from(value: CrosstermMouseButton) -> Self {
        match value {
            CrosstermMouseButton::Left => Self::Left,
            CrosstermMouseButton::Right => Self::Right,
            CrosstermMouseButton::Middle => Self::Middle,
        }
    }
}
