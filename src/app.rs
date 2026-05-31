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
        ActiveWalkDestination, CombatLog, ControlFocus, Faction, GameMode, PendingWalkDestination,
        Position, TurnOrder, ViewFocus, end_current_turn, movement_schedule, spawn_initial_entities,
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
    /// Last world cell painted during the current left-drag stroke; used to
    /// interpolate a line so fast mouse motion never skips tiles.
    last_paint: Option<Vector>,
    /// World cell where the current right-drag box started.
    box_anchor: Option<Vector>,
    /// Current far corner of the right-drag box (for preview and commit).
    box_end: Option<Vector>,
    /// Whether the right-drag has moved off its anchor (box) vs. stayed put (fill).
    box_dragged: bool,
    /// Terrain changes made by the current gesture, as `(coord, prior terrain)`.
    current_edit: Vec<(Vector, TerrainType)>,
    /// Completed edit gestures, newest last, for undo.
    undo_stack: Vec<Vec<(Vector, TerrainType)>>,
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
            last_paint: None,
            box_anchor: None,
            box_end: None,
            box_dragged: false,
            current_edit: Vec::new(),
            undo_stack: Vec::new(),
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

    /// Converts a world coordinate to a viewport cell, if it is on screen.
    pub fn viewport_world_to_cell(&self, size: Vector, coord: Vector) -> Option<Vector> {
        world_to_local(self.view_center(), size, coord)
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

    /// The active right-drag box as `(anchor, end)` world coordinates, while a
    /// box (not a flood-fill click) is being dragged. Used to render a preview.
    pub fn edit_box(&self) -> Option<(Vector, Vector)> {
        match (self.box_anchor, self.box_end) {
            (Some(anchor), Some(end)) if self.box_dragged => Some((anchor, end)),
            _ => None,
        }
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
            self.end_stroke();
        } else {
            self.edit_mode = true;
            self.edit_focus = Some(self.ecs_world.resource::<ViewFocus>().center);
        }
    }

    fn paint_terrain(&mut self, coord: Vector) {
        let kind = self.selected_terrain;
        if let Some(old) = self.ecs_world.resource_mut::<World>().replace_terrain(coord, kind) {
            self.current_edit.push((coord, old));
            self.dirty = true;
        }
        self.last_paint = Some(coord);
    }

    /// Paints a continuous line from the last painted cell to `coord` so a fast
    /// left-drag never skips tiles, then advances the stroke cursor.
    fn paint_terrain_line(&mut self, coord: Vector) {
        let from = self.last_paint.unwrap_or(coord);
        let kind = self.selected_terrain;
        for cell in line_points(from, coord) {
            if let Some(old) = self.ecs_world.resource_mut::<World>().replace_terrain(cell, kind) {
                self.current_edit.push((cell, old));
                self.dirty = true;
            }
        }
        self.last_paint = Some(coord);
    }

    fn begin_box(&mut self, coord: Vector) {
        self.box_anchor = Some(coord);
        self.box_end = Some(coord);
        self.box_dragged = false;
    }

    fn extend_box(&mut self, coord: Vector) {
        if let Some(anchor) = self.box_anchor {
            self.box_end = Some(coord);
            if coord != anchor {
                self.box_dragged = true;
            }
        }
    }

    /// Commits the right-button gesture: a moved box fills its rectangle, while
    /// a click that never moved flood-fills the tile under it.
    fn commit_box(&mut self) {
        let Some(anchor) = self.box_anchor else {
            self.end_stroke();
            return;
        };
        let end = self.box_end.unwrap_or(anchor);
        let kind = self.selected_terrain;
        let edits = if self.box_dragged {
            self.ecs_world
                .resource_mut::<World>()
                .fill_rect_recording(anchor, end, kind)
        } else {
            self.ecs_world
                .resource_mut::<World>()
                .flood_fill_recording(anchor, kind)
        };
        if !edits.is_empty() {
            self.current_edit.extend(edits);
            self.dirty = true;
        }
        self.end_stroke();
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

    /// Ends any in-progress paint/pan/box capture, clearing transient cursors.
    /// Any terrain changes made during the gesture become one undo entry.
    fn end_stroke(&mut self) {
        if !self.current_edit.is_empty() {
            self.undo_stack.push(std::mem::take(&mut self.current_edit));
        }
        self.pan_anchor = None;
        self.last_paint = None;
        self.box_anchor = None;
        self.box_end = None;
        self.box_dragged = false;
    }

    /// Reverts the most recent edit gesture, restoring each touched cell's prior
    /// terrain. No-op when there is nothing to undo.
    fn undo(&mut self) {
        let Some(edits) = self.undo_stack.pop() else {
            return;
        };
        // Restore in reverse so overlapping cells land on their oldest value.
        for (coord, kind) in edits.into_iter().rev() {
            self.ecs_world.resource_mut::<World>().replace_terrain(coord, kind);
        }
        self.dirty = true;
    }

    /// Whether there is a recorded edit gesture available to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
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

    /// Whether it is currently a player-controlled character's turn in battle.
    pub fn is_player_turn(&self) -> bool {
        if !self.is_turn_based() {
            return false;
        }
        let Some(active) = self.ecs_world.resource::<TurnOrder>().active() else {
            return false;
        };
        self.ecs_world.get::<Faction>(active) == Some(&Faction::Party)
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
    /// Undo the last terrain edit (edit mode only).
    Undo,
    /// Paint the selected terrain at a world coordinate (left-button down).
    PaintTerrain(Vector),
    /// Paint a line from the last painted cell to this world coordinate (drag).
    PaintTerrainLine(Vector),
    /// Begin a right-drag box / flood-fill gesture at a world coordinate.
    BeginEditBox(Vector),
    /// Extend the right-drag box to a world coordinate.
    ExtendEditBox(Vector),
    /// Commit the right-button gesture (box fill, or flood fill on a click).
    CommitEditBox,
    /// Select the active terrain in the palette.
    SelectTerrain(TerrainType),
    /// Collapse/expand the terrain palette.
    TogglePaletteCollapse,
    /// Begin a middle-drag pan at a screen point.
    BeginEditPan(Vector),
    /// Continue a middle-drag pan to a screen point.
    DragEditPan(Vector),
    /// End the current paint/pan/box capture.
    EndEditStroke,
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
        AppMessage::Undo => {
            if state.edit_mode {
                state.undo();
            }
        }
        AppMessage::PaintTerrain(coord) => {
            state.paint_terrain(coord);
        }
        AppMessage::PaintTerrainLine(coord) => {
            state.paint_terrain_line(coord);
        }
        AppMessage::BeginEditBox(coord) => {
            state.begin_box(coord);
        }
        AppMessage::ExtendEditBox(coord) => {
            state.extend_box(coord);
        }
        AppMessage::CommitEditBox => {
            state.commit_box();
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
        AppMessage::EndEditStroke => {
            state.end_stroke();
        }
    }
}

/// Returns the integer cells on the line from `from` to `to` (inclusive),
/// using Bresenham's algorithm so a fast drag paints every crossed tile.
fn line_points(from: Vector, to: Vector) -> Vec<Vector> {
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut error = dx + dy;
    let mut x = from.x;
    let mut y = from.y;
    let mut points = Vec::new();
    loop {
        points.push(Vector { x, y });
        if x == to.x && y == to.y {
            break;
        }
        let double_error = 2 * error;
        if double_error >= dy {
            error += dy;
            x += sx;
        }
        if double_error <= dx {
            error += dx;
            y += sy;
        }
    }
    points
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
            KeyCode::Char('u') => Some(AppMessage::Undo),
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

        update(&mut state, AppMessage::EndEditStroke);
        assert_eq!(state.pan_anchor, None);
    }

    #[test]
    fn left_drag_paints_a_continuous_line_without_skipping_tiles() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Road));

        // Press at the origin, then a fast drag jumps several tiles away.
        update(&mut state, AppMessage::PaintTerrain(ORIGIN));
        update(
            &mut state,
            AppMessage::PaintTerrainLine(Vector { x: 4, y: 2 }),
        );

        // Every cell on the line from (0,0) to (4,2) is painted, none skipped.
        for cell in super::line_points(ORIGIN, Vector { x: 4, y: 2 }) {
            assert_eq!(
                state
                    .ecs_world
                    .resource::<World>()
                    .terrain_at(cell)
                    .map(|terrain| terrain.kind()),
                Some(TerrainType::Road),
                "missing painted tile at {cell:?}"
            );
        }
        assert!(state.dirty);
    }

    #[test]
    fn right_click_without_drag_flood_fills() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);

        // Paint a small all-grass pocket surrounded by road so the fill is bounded.
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Road));
        update(
            &mut state,
            AppMessage::BeginEditBox(Vector { x: -2, y: -2 }),
        );
        update(
            &mut state,
            AppMessage::ExtendEditBox(Vector { x: 2, y: 2 }),
        );
        update(&mut state, AppMessage::CommitEditBox);
        // Carve a grass hole back into the middle.
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Grass));
        update(&mut state, AppMessage::BeginEditBox(ORIGIN));
        update(&mut state, AppMessage::ExtendEditBox(Vector { x: 1, y: 0 }));
        update(&mut state, AppMessage::CommitEditBox);

        // Flood fill the grass pocket with forest via a right click (no drag).
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Forest));
        update(&mut state, AppMessage::BeginEditBox(ORIGIN));
        update(&mut state, AppMessage::CommitEditBox);

        let world = state.ecs_world.resource::<World>();
        assert_eq!(
            world.terrain_at(ORIGIN).map(|t| t.kind()),
            Some(TerrainType::Forest)
        );
        // The surrounding road is untouched by the bounded fill.
        assert_eq!(
            world.terrain_at(Vector { x: 2, y: 2 }).map(|t| t.kind()),
            Some(TerrainType::Road)
        );
    }

    #[test]
    fn right_drag_fills_a_box() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Pond));

        update(&mut state, AppMessage::BeginEditBox(Vector { x: -1, y: -1 }));
        update(&mut state, AppMessage::ExtendEditBox(Vector { x: 1, y: 1 }));
        // A box is previewed mid-drag.
        assert_eq!(
            state.edit_box(),
            Some((Vector { x: -1, y: -1 }, Vector { x: 1, y: 1 }))
        );
        update(&mut state, AppMessage::CommitEditBox);

        let world = state.ecs_world.resource::<World>();
        for y in -1..=1 {
            for x in -1..=1 {
                assert_eq!(
                    world.terrain_at(Vector { x, y }).map(|t| t.kind()),
                    Some(TerrainType::Pond),
                    "box cell ({x}, {y}) not filled"
                );
            }
        }
        // Capture state is cleared after commit.
        assert_eq!(state.edit_box(), None);
        assert!(state.dirty);
    }

    #[test]
    fn u_key_undoes_only_in_edit_mode() {
        assert_eq!(
            global_message(&key_event(KeyCode::Char('u'))),
            Some(AppMessage::Undo)
        );

        let mut state = AppState::default();
        let original = state
            .ecs_world
            .resource::<World>()
            .terrain_at(ORIGIN)
            .map(|t| t.kind())
            .unwrap();

        // Outside edit mode, undo is ignored (nothing recorded anyway).
        update(&mut state, AppMessage::Undo);

        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Forest));
        // A full left-stroke: down, drag, up.
        update(&mut state, AppMessage::PaintTerrain(ORIGIN));
        update(&mut state, AppMessage::PaintTerrainLine(Vector { x: 3, y: 0 }));
        update(&mut state, AppMessage::EndEditStroke);
        assert!(state.can_undo());
        assert_eq!(
            state.ecs_world.resource::<World>().terrain_at(ORIGIN).map(|t| t.kind()),
            Some(TerrainType::Forest)
        );

        update(&mut state, AppMessage::Undo);
        assert!(!state.can_undo());
        for cell in super::line_points(ORIGIN, Vector { x: 3, y: 0 }) {
            assert_eq!(
                state.ecs_world.resource::<World>().terrain_at(cell).map(|t| t.kind()),
                Some(original),
                "stroke cell {cell:?} not restored by undo"
            );
        }
    }

    #[test]
    fn undo_reverts_box_and_flood_fills_as_single_steps() {
        let mut state = AppState::default();
        update(&mut state, AppMessage::ToggleEditMode);
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Pond));

        // One box gesture.
        update(&mut state, AppMessage::BeginEditBox(Vector { x: -1, y: -1 }));
        update(&mut state, AppMessage::ExtendEditBox(Vector { x: 1, y: 1 }));
        update(&mut state, AppMessage::CommitEditBox);
        // One flood gesture over the pond pocket.
        update(&mut state, AppMessage::SelectTerrain(TerrainType::Road));
        update(&mut state, AppMessage::BeginEditBox(ORIGIN));
        update(&mut state, AppMessage::CommitEditBox);
        assert_eq!(
            state.ecs_world.resource::<World>().terrain_at(ORIGIN).map(|t| t.kind()),
            Some(TerrainType::Road)
        );

        // Undo the flood: pocket returns to pond.
        update(&mut state, AppMessage::Undo);
        assert_eq!(
            state.ecs_world.resource::<World>().terrain_at(ORIGIN).map(|t| t.kind()),
            Some(TerrainType::Pond)
        );
        // Undo the box: corner returns to its original terrain.
        update(&mut state, AppMessage::Undo);
        assert_ne!(
            state.ecs_world.resource::<World>().terrain_at(Vector { x: 1, y: 1 }).map(|t| t.kind()),
            Some(TerrainType::Pond)
        );
        assert!(!state.can_undo());
    }
}
