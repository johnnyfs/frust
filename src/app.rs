//! Application-owned state and update logic.

use std::time::Duration;

use bevy_ecs::{schedule::Schedule, world::World as EcsWorld};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::{
    data::{
        grid::{ORIGIN, Vector},
        world::World,
    },
    ecs::{
        ActiveWalkDestination, CombatLog, ControlFocus, GameMode, PendingWalkDestination, Position,
        ViewFocus, end_current_turn, movement_schedule, spawn_initial_entities,
        sync_view_focus_system,
    },
    view::{
        coordinates::{local_to_world, world_to_local},
        entityview::{EntityView, from_ecs},
        inspectorview::{InspectorView, from_ecs_at},
        worldview::{WorldView, from_world},
    },
};

/// Initial area name shown by the client UI.
pub const BRIDGEPORT_OUTSKIRTS: &str = "Bridgeport Outskirts";
/// Fixed gameplay cadence for one walking step.
pub const PLAYER_STEP_INTERVAL: Duration = Duration::from_millis(100);
/// Local file used for combat/debug messages so the terminal UI never scrolls.
pub const COMBAT_LOG_PATH: &str = "frust.log";

/// Durable client state.
pub struct AppState {
    /// ECS simulation state.
    pub ecs_world: EcsWorld,
    movement_schedule: Schedule,
    viewport_cursor: Option<Vector>,
    /// Whether the terminal client should exit.
    pub quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let mut ecs_world = EcsWorld::new();
        ecs_world.insert_resource(World::new().with_region(BRIDGEPORT_OUTSKIRTS, ORIGIN));
        spawn_initial_entities(&mut ecs_world);
        sync_view_focus_system(&mut ecs_world);

        Self {
            ecs_world,
            movement_schedule: movement_schedule(),
            viewport_cursor: None,
            quit: false,
        }
    }
}

impl AppState {
    pub fn world_view(&self, size: Vector) -> WorldView {
        let world = self.ecs_world.resource::<World>();
        let center = self.ecs_world.resource::<ViewFocus>().center;
        from_world(world, center, size)
    }

    pub fn entity_view(&self, size: Vector) -> EntityView {
        from_ecs(&self.ecs_world, size)
    }

    pub fn inspector_at(&self, coord: Vector) -> InspectorView {
        from_ecs_at(&self.ecs_world, coord)
    }

    pub fn viewport_cell_to_world(&self, size: Vector, local: Vector) -> Vector {
        let center = self.ecs_world.resource::<ViewFocus>().center;
        local_to_world(center, size, local)
    }

    pub fn viewport_cursor(&self) -> Option<Vector> {
        self.viewport_cursor
    }

    pub fn focused_walk_destination(&self) -> Option<Vector> {
        self.ecs_world.resource::<ActiveWalkDestination>().0
    }

    pub fn viewport_destination_cell(&self, size: Vector) -> Option<Vector> {
        let center = self.ecs_world.resource::<ViewFocus>().center;
        world_to_local(center, size, self.focused_walk_destination()?)
    }

    pub fn viewport_focus_cell(&self, size: Vector) -> Option<Vector> {
        if !self.is_turn_based() {
            return None;
        }

        let center = self.ecs_world.resource::<ViewFocus>().center;
        let focused = self.ecs_world.resource::<ControlFocus>().entity;
        let position = self.ecs_world.get::<Position>(focused)?.0;
        world_to_local(center, size, position)
    }

    pub fn is_turn_based(&self) -> bool {
        *self.ecs_world.resource::<GameMode>() == GameMode::TurnBased
    }

    pub fn combat_log_lines(&self, limit: usize) -> Vec<&str> {
        let log = self.ecs_world.resource::<CombatLog>();
        log.lines().rev().take(limit).collect::<Vec<_>>()
    }

    pub fn enable_file_logging(&mut self, path: impl Into<std::path::PathBuf>) {
        let _ = self.ecs_world.resource_mut::<CombatLog>().set_file(path);
    }

    fn tick(&mut self) {
        self.movement_schedule.run(&mut self.ecs_world);
    }
}

/// Application messages emitted by UI routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMessage {
    ViewportClicked(Vector),
    WalkFocusedEntityTo(Vector),
    SetViewportCursor(Option<Vector>),
    EndTurn,
}

/// Applies a UI/application message to durable state.
pub fn update(state: &mut AppState, message: AppMessage) {
    match message {
        AppMessage::ViewportClicked(destination) | AppMessage::WalkFocusedEntityTo(destination) => {
            state.ecs_world.resource_mut::<PendingWalkDestination>().0 = Some(destination);
            state.ecs_world.resource_mut::<ActiveWalkDestination>().0 = Some(destination);
        }
        AppMessage::SetViewportCursor(cursor) => {
            state.viewport_cursor = cursor;
        }
        AppMessage::EndTurn => {
            end_current_turn(&mut state.ecs_world);
        }
    }
}

/// Advances one gameplay tick.
pub fn tick(state: &mut AppState) {
    state.tick();
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

/// Converts global raw input into application messages before UI routing.
pub fn global_message(event: &Event) -> Option<AppMessage> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') => {
            Some(AppMessage::EndTurn)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{AppMessage, AppState, tick, update};
    use crate::{
        data::grid::Vector,
        ecs::{ControlFocus, Position, Renderable, RenderableEntities, ViewFocus, WalkTarget},
    };

    #[test]
    fn ecs_startup_creates_map_player_sign_and_focus() {
        let mut state = AppState::default();

        assert!(
            state
                .ecs_world
                .get_resource::<crate::data::world::World>()
                .is_some()
        );
        assert!(state.ecs_world.get_resource::<ControlFocus>().is_some());
        assert_eq!(
            state.ecs_world.resource::<ViewFocus>().center,
            Vector { x: 0, y: 0 }
        );

        let mut query = state.ecs_world.query::<(&Position, &Renderable)>();
        let entities = query.iter(&state.ecs_world).collect::<Vec<_>>();
        assert!(entities.iter().any(|(position, renderable)| position.0
            == (Vector { x: 0, y: 0 })
            && renderable.glyph == '@'));
        assert!(entities.iter().any(|(position, renderable)| position.0
            == (Vector { x: 4, y: 1 })
            && renderable.glyph == '|'));
    }

    #[test]
    fn click_destination_adds_and_replaces_walk_target() {
        let mut state = AppState::default();
        let player = state.ecs_world.resource::<ControlFocus>().entity;

        update(
            &mut state,
            AppMessage::WalkFocusedEntityTo(Vector { x: 3, y: 0 }),
        );
        tick(&mut state);
        assert_eq!(
            state
                .ecs_world
                .get::<WalkTarget>(player)
                .unwrap()
                .destination,
            Vector { x: 3, y: 0 }
        );

        update(
            &mut state,
            AppMessage::WalkFocusedEntityTo(Vector { x: -2, y: 0 }),
        );
        tick(&mut state);
        assert_eq!(
            state
                .ecs_world
                .get::<WalkTarget>(player)
                .unwrap()
                .destination,
            Vector { x: -2, y: 0 }
        );
    }

    #[test]
    fn gameplay_tick_walks_one_step_and_stops_at_destination() {
        let mut state = AppState::default();
        let player = state.ecs_world.resource::<ControlFocus>().entity;

        update(
            &mut state,
            AppMessage::WalkFocusedEntityTo(Vector { x: 2, y: 0 }),
        );
        tick(&mut state);
        assert_eq!(
            state.ecs_world.get::<Position>(player).unwrap().0,
            Vector { x: 1, y: 0 }
        );
        assert!(state.ecs_world.get::<WalkTarget>(player).is_some());

        tick(&mut state);
        assert_eq!(
            state.ecs_world.get::<Position>(player).unwrap().0,
            Vector { x: 2, y: 0 }
        );
        assert!(state.ecs_world.get::<WalkTarget>(player).is_none());
    }

    #[test]
    fn world_view_is_centered_on_view_focus() {
        let mut state = AppState::default();

        state.ecs_world.resource_mut::<ViewFocus>().center = Vector { x: 160, y: 0 };

        assert_eq!(
            state.world_view(Vector { x: 1, y: 1 }).current_region_name,
            ""
        );
    }

    #[test]
    fn entity_view_includes_visible_entities_and_respects_z_priority() {
        let mut state = AppState::default();
        let hidden_under_player = state
            .ecs_world
            .spawn((
                Position(Vector { x: 0, y: 0 }),
                Renderable {
                    glyph: 'x',
                    color: ratatui::style::Color::Red,
                    bold: false,
                    z: 1,
                },
            ))
            .id();
        state
            .ecs_world
            .resource_mut::<RenderableEntities>()
            .entities
            .push(hidden_under_player);

        let entity_view = state.entity_view(Vector { x: 20, y: 8 });

        assert_eq!(entity_view.get(10, 4).unwrap().glyph, '@');
        assert_eq!(entity_view.get(14, 5).unwrap().glyph, '|');
        assert!(entity_view.get(0, 0).is_none());
    }

    #[test]
    fn destination_is_available_immediately_and_clears_on_arrival() {
        let mut state = AppState::default();

        update(
            &mut state,
            AppMessage::WalkFocusedEntityTo(Vector { x: 1, y: 0 }),
        );

        assert_eq!(
            state.focused_walk_destination(),
            Some(Vector { x: 1, y: 0 })
        );
        assert_eq!(
            state.viewport_destination_cell(Vector { x: 20, y: 8 }),
            Some(Vector { x: 11, y: 4 })
        );

        tick(&mut state);

        assert_eq!(state.focused_walk_destination(), None);
        assert_eq!(
            state.viewport_destination_cell(Vector { x: 20, y: 8 }),
            None
        );
    }
}
