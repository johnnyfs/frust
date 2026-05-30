//! Application-owned state and update logic.

use std::{path::PathBuf, time::Duration};

use bevy_ecs::{schedule::Schedule, world::World as EcsWorld};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::{
    data::{
        grid::{ORIGIN, Vector},
        region::{self, RegionDocument, RegionError},
        world::{TerrainType, World},
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
        party_status::PartyStatusView,
        worldview::{WorldView, from_world},
    },
};

/// Initial area name shown by the client UI.
pub const BRIDGEPORT_OUTSKIRTS: &str = "Bridgeport Outskirts";
/// Fixed gameplay cadence for one walking step.
pub const PLAYER_STEP_INTERVAL: Duration = Duration::from_millis(100);
/// Local file used for combat/debug messages so the terminal UI never scrolls.
pub const COMBAT_LOG_PATH: &str = "frust.log";
/// Default region resource loaded at startup.
pub const DEFAULT_REGION_PATH: &str = "resources/regions/bridgeport_outskirts.json";

/// Durable client state.
pub struct AppState {
    /// ECS simulation state.
    pub ecs_world: EcsWorld,
    movement_schedule: Schedule,
    viewport_cursor: Option<Vector>,
    /// Whether the terminal client should exit.
    pub quit: bool,
    /// Whether terrain edit mode is active.
    edit_mode: bool,
    /// Frozen view center while edit mode is active.
    edit_focus: Option<Vector>,
    /// Terrain painted on left click in edit mode.
    selected_terrain: TerrainType,
    /// Whether the terrain palette is collapsed.
    palette_collapsed: bool,
    /// Whether the loaded region has unsaved edits.
    dirty: bool,
    /// Last region-save error, if any.
    save_error: Option<String>,
    /// Path of the loaded region resource, used for saving.
    region_path: Option<PathBuf>,
    /// Center coordinate of the loaded region.
    region_center: Vector,
    /// Last pointer position captured during a middle-drag pan.
    pan_anchor: Option<Vector>,
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
            edit_mode: false,
            edit_focus: None,
            selected_terrain: TerrainType::Grass,
            palette_collapsed: false,
            dirty: false,
            save_error: None,
            region_path: None,
            region_center: ORIGIN,
            pan_anchor: None,
        }
    }
}

impl AppState {
    /// View center used for rendering and coordinate conversion. In edit mode
    /// this is the frozen edit focus; otherwise it follows the ECS view focus.
    fn view_center(&self) -> Vector {
        self.edit_focus
            .unwrap_or_else(|| self.ecs_world.resource::<ViewFocus>().center)
    }

    pub fn world_view(&self, size: Vector) -> WorldView {
        let world = self.ecs_world.resource::<World>();
        from_world(world, self.view_center(), size)
    }

    pub fn entity_view(&self, size: Vector) -> EntityView {
        from_ecs(&self.ecs_world, self.view_center(), size)
    }

    pub fn inspector_at(&self, coord: Vector) -> InspectorView {
        from_ecs_at(&self.ecs_world, coord)
    }

    pub fn party_status_view(&self) -> PartyStatusView {
        crate::view::party_status::from_ecs(&self.ecs_world)
    }

    pub fn viewport_cell_to_world(&self, size: Vector, local: Vector) -> Vector {
        local_to_world(self.view_center(), size, local)
    }

    pub fn viewport_cursor(&self) -> Option<Vector> {
        self.viewport_cursor
    }

    pub fn focused_walk_destination(&self) -> Option<Vector> {
        self.ecs_world.resource::<ActiveWalkDestination>().0
    }

    pub fn viewport_destination_cell(&self, size: Vector) -> Option<Vector> {
        world_to_local(self.view_center(), size, self.focused_walk_destination()?)
    }

    pub fn viewport_focus_cell(&self, size: Vector) -> Option<Vector> {
        if !self.is_turn_based() {
            return None;
        }

        let focused = self.ecs_world.resource::<ControlFocus>().entity;
        let position = self.ecs_world.get::<Position>(focused)?.0;
        world_to_local(self.view_center(), size, position)
    }

    /// Whether terrain edit mode is active.
    pub fn edit_mode(&self) -> bool {
        self.edit_mode
    }

    /// Terrain selected for painting in edit mode.
    pub fn selected_terrain(&self) -> TerrainType {
        self.selected_terrain
    }

    /// Whether the terrain palette is collapsed.
    pub fn palette_collapsed(&self) -> bool {
        self.palette_collapsed
    }

    /// Last region-save error message, if any.
    pub fn save_error(&self) -> Option<&str> {
        self.save_error.as_deref()
    }

    /// Loads (or creates) a region resource and installs it as the world.
    pub fn load_region_from(&mut self, path: impl Into<PathBuf>) -> Result<(), RegionError> {
        let path = path.into();
        let document = region::load_or_create(&path)?;
        let (center, region) = document.to_region()?;
        let mut world = World::new();
        world.insert_region(center, region);
        self.ecs_world.insert_resource(world);
        self.region_path = Some(path);
        self.region_center = center;
        self.dirty = false;
        self.save_error = None;
        Ok(())
    }

    fn toggle_edit_mode(&mut self) {
        if self.edit_mode {
            self.edit_mode = false;
            self.edit_focus = None;
            self.pan_anchor = None;
        } else {
            self.edit_mode = true;
            self.edit_focus = Some(self.ecs_world.resource::<ViewFocus>().center);
        }
    }

    fn paint_terrain(&mut self, coord: Vector) {
        let kind = self.selected_terrain;
        if self.ecs_world.resource_mut::<World>().set_terrain(coord, kind) {
            self.dirty = true;
        }
    }

    fn begin_pan(&mut self, point: Vector) {
        if self.edit_mode {
            self.pan_anchor = Some(point);
        }
    }

    fn drag_pan(&mut self, point: Vector) {
        if !self.edit_mode {
            return;
        }
        let Some(anchor) = self.pan_anchor else {
            return;
        };
        let delta = Vector {
            x: point.x - anchor.x,
            y: point.y - anchor.y,
        };
        let focus = self.view_center();
        self.edit_focus = Some(Vector {
            x: focus.x - delta.x,
            y: focus.y - delta.y,
        });
        self.pan_anchor = Some(point);
    }

    fn end_pan(&mut self) {
        self.pan_anchor = None;
    }

    fn save_region(&mut self) {
        let Some(path) = self.region_path.clone() else {
            self.save_error = Some("no region file configured".to_string());
            return;
        };
        let world = self.ecs_world.resource::<World>();
        let Some(region) = world.region_at(self.region_center) else {
            self.save_error = Some("no loaded region to save".to_string());
            return;
        };
        let document = RegionDocument::from_region(self.region_center, region);
        match region::save_document(&path, &document) {
            Ok(()) => {
                self.dirty = false;
                self.save_error = None;
            }
            Err(error) => {
                self.save_error = Some(error.to_string());
            }
        }
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
    /// Toggle terrain edit mode.
    ToggleEditMode,
    /// Save the loaded region (edit mode only).
    SaveRegion,
    /// Paint the selected terrain at a world coordinate.
    PaintTerrain(Vector),
    /// Select the active terrain in the palette.
    SelectTerrain(TerrainType),
    /// Collapse/expand the terrain palette.
    TogglePaletteCollapse,
    /// Begin a middle-drag pan at a screen point.
    BeginEditPan(Vector),
    /// Continue a middle-drag pan to a screen point.
    DragEditPan(Vector),
    /// End the current paint/pan capture.
    EndEditPan,
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
        AppMessage::ToggleEditMode => {
            state.toggle_edit_mode();
        }
        AppMessage::SaveRegion => {
            if state.edit_mode {
                state.save_region();
            }
        }
        AppMessage::PaintTerrain(coord) => {
            state.paint_terrain(coord);
        }
        AppMessage::SelectTerrain(terrain) => {
            state.selected_terrain = terrain;
        }
        AppMessage::TogglePaletteCollapse => {
            state.palette_collapsed = !state.palette_collapsed;
        }
        AppMessage::BeginEditPan(point) => {
            state.begin_pan(point);
        }
        AppMessage::DragEditPan(point) => {
            state.drag_pan(point);
        }
        AppMessage::EndEditPan => {
            state.end_pan();
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
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char(' ') => Some(AppMessage::EndTurn),
            KeyCode::Char('e') => Some(AppMessage::ToggleEditMode),
            KeyCode::Char('s') => Some(AppMessage::SaveRegion),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{AppMessage, AppState, global_message, tick, update};
    use crate::{
        data::{
            grid::{ORIGIN, Vector},
            world::{TerrainType, World},
        },
        ecs::{ControlFocus, Position, Renderable, RenderableEntities, ViewFocus, WalkTarget},
    };
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn temp_region_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "frust-app-region-{}-{label}.json",
            std::process::id()
        ))
    }

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

    #[test]
    fn e_key_toggles_edit_mode_and_freezes_edit_focus() {
        let mut state = AppState::default();
        state.ecs_world.resource_mut::<ViewFocus>().center = Vector { x: 7, y: -3 };
        assert!(!state.edit_mode());
        assert_eq!(state.edit_focus, None);

        let message = global_message(&key_event(KeyCode::Char('e'))).unwrap();
        assert_eq!(message, AppMessage::ToggleEditMode);
        update(&mut state, message);

        assert!(state.edit_mode());
        assert_eq!(state.edit_focus, Some(Vector { x: 7, y: -3 }));

        update(&mut state, AppMessage::ToggleEditMode);
        assert!(!state.edit_mode());
        assert_eq!(state.edit_focus, None);
    }

    #[test]
    fn s_key_saves_only_in_edit_mode() {
        let path = temp_region_path("save-only-in-edit");
        let _ = std::fs::remove_file(&path);

        let mut state = AppState::default();
        state.region_path = Some(path.clone());
        state.region_center = ORIGIN;

        // Outside edit mode the save is ignored.
        let message = global_message(&key_event(KeyCode::Char('s'))).unwrap();
        assert_eq!(message, AppMessage::SaveRegion);
        update(&mut state, message);
        assert!(!path.exists());

        // In edit mode the save writes the document.
        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SaveRegion);
        assert!(path.exists());
        assert_eq!(state.save_error(), None);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn painting_changes_terrain_and_marks_world_dirty() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Forest));

        assert!(!state.dirty);
        update(&mut state, AppMessage::PaintTerrain(ORIGIN));

        assert!(state.dirty);
        assert_eq!(
            state
                .ecs_world
                .resource::<World>()
                .terrain_at(ORIGIN)
                .map(|terrain| terrain.kind()),
            Some(TerrainType::Forest)
        );
    }

    #[test]
    fn painting_outside_a_region_is_a_noop() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);
        update(
            &mut state,
            AppMessage::PaintTerrain(Vector { x: 100_000, y: 0 }),
        );
        assert!(!state.dirty);
    }

    #[test]
    fn middle_drag_pans_edit_focus_with_inverted_delta() {
        let mut state = AppState::default();
        state.ecs_world.resource_mut::<ViewFocus>().center = ORIGIN;
        update(&mut state, AppMessage::ToggleEditMode);
        assert_eq!(state.edit_focus, Some(ORIGIN));

        update(&mut state, AppMessage::BeginEditPan(Vector { x: 10, y: 10 }));
        // Dragging down (y increases) moves the focus upward (y decreases).
        update(&mut state, AppMessage::DragEditPan(Vector { x: 7, y: 15 }));

        assert_eq!(state.edit_focus, Some(Vector { x: 3, y: -5 }));

        update(&mut state, AppMessage::EndEditPan);
        assert_eq!(state.pan_anchor, None);
    }
}
