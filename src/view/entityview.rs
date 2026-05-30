use bevy_ecs::world::World as EcsWorld;
use ratatui::style::Color;

use crate::{
    data::grid::{SparseGrid, Vector},
    ecs::{Position, Renderable, RenderableEntities, ViewFocus},
    view::coordinates::world_to_local,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityViewCell {
    pub glyph: char,
    pub color: Color,
    pub bold: bool,
    pub z: i32,
}

#[derive(Debug)]
pub struct EntityView {
    cells: SparseGrid<EntityViewCell>,
}

impl EntityView {
    pub fn get(&self, x: usize, y: usize) -> Option<&EntityViewCell> {
        self.cells.get(&Vector {
            x: x as i32,
            y: y as i32,
        })
    }
}

pub fn from_ecs(world: &EcsWorld, size: Vector) -> EntityView {
    let center = world.resource::<ViewFocus>().center;
    let renderables = world.resource::<RenderableEntities>();
    let mut cells: SparseGrid<EntityViewCell> = SparseGrid::new();

    for entity in renderables.entities.iter().copied() {
        let Ok(entity_ref) = world.get_entity(entity) else {
            continue;
        };
        let Some(position) = entity_ref.get::<Position>() else {
            continue;
        };
        let Some(renderable) = entity_ref.get::<Renderable>() else {
            continue;
        };
        let Some(local) = world_to_local(center, size, position.0) else {
            continue;
        };

        let cell = EntityViewCell {
            glyph: renderable.glyph,
            color: renderable.color,
            bold: renderable.bold,
            z: renderable.z,
        };
        let should_replace = cells
            .get(&local)
            .map(|current| cell.z >= current.z)
            .unwrap_or(true);
        if should_replace {
            cells.add(local, cell);
        }
    }

    EntityView { cells }
}
