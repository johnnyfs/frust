//! Application-owned state and update logic.

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::data::world;

/// Initial area name shown by the client UI.
pub const BRIDGEPORT_OUTSKIRTS: &str = "Bridgeport Outskirts";

/// Durable client state.
#[derive(Debug)]
pub struct AppState {
    /// Current area display name.
    pub world: World,
    /// Whether the terminal client should exit.
    pub quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_area_name: BRIDGEPORT_OUTSKIRTS,

            quit: false,
        }
    }
}

/// Application messages emitted by UI routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMessage {}

/// Applies a UI/application message to durable state.
pub fn update(_state: &mut AppState, message: AppMessage) {
    match message {}
}

/// Returns true for global quit keybindings.
pub fn should_quit(event: &Event) -> bool {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            key.code == KeyCode::Char('q')
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        }
        _ => false,
    }
}
