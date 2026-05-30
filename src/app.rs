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
        from_world(&self.world, self.world.player_position(), size)
    }
}

/// Application messages emitted by UI routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMessage {
    MovePlayer(Vector),
}

/// Applies a UI/application message to durable state.
pub fn update(state: &mut AppState, message: AppMessage) {
    match message {
        AppMessage::MovePlayer(delta) => state.world.move_player_by(delta),
    }
}

/// Converts global application keybindings into messages.
pub fn message_for_event(event: &Event) -> Option<AppMessage> {
    match event {
        Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
            match key.code {
                KeyCode::Up => Some(AppMessage::MovePlayer(Vector { x: 0, y: -1 })),
                KeyCode::Down => Some(AppMessage::MovePlayer(Vector { x: 0, y: 1 })),
                KeyCode::Left => Some(AppMessage::MovePlayer(Vector { x: -1, y: 0 })),
                KeyCode::Right => Some(AppMessage::MovePlayer(Vector { x: 1, y: 0 })),
                _ => None,
            }
        }
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    use super::{AppMessage, AppState, message_for_event, update};
    use crate::data::grid::Vector;

    #[test]
    fn arrow_key_messages_move_player() {
        let mut state = AppState::default();

        update(&mut state, AppMessage::MovePlayer(Vector { x: 1, y: 0 }));
        update(&mut state, AppMessage::MovePlayer(Vector { x: 0, y: -1 }));

        assert_eq!(state.world.player_position(), Vector { x: 1, y: -1 });
    }

    #[test]
    fn arrow_key_events_become_movement_messages() {
        let event = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));

        assert_eq!(
            message_for_event(&event),
            Some(AppMessage::MovePlayer(Vector { x: 1, y: 0 }))
        );
    }

    #[test]
    fn world_view_is_centered_on_player_position() {
        let mut state = AppState::default();

        update(&mut state, AppMessage::MovePlayer(Vector { x: 512, y: 0 }));

        assert_eq!(
            state.world_view(Vector { x: 1, y: 1 }).current_region_name,
            ""
        );
    }
}
