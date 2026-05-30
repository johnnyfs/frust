//! Application-owned state and update logic.

use crate::{
    data::{
        grid::{ORIGIN, Vector},
        world::World,
    },
    view::worldview::{WorldView, from_world},
};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

/// Initial area name shown by the client UI.
pub const BRIDGEPORT_OUTSKIRTS: &str = "Bridgeport Outskirts";

/// Durable client state.
#[derive(Debug)]
pub struct AppState {
    /// Durable world state.
    pub world: World,
    /// Whether the terminal client should exit.
    pub quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            world: World::new().with_region(BRIDGEPORT_OUTSKIRTS, ORIGIN),
            quit: false,
        }
    }
}

impl AppState {
    pub fn world_view(&self, size: Vector) -> WorldView {
        from_world(&self.world, ORIGIN, size)
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
