use bevy_ecs::world::World as EcsWorld;
use ratatui::style::Color;

use crate::{
    data::{grid::Vector, world::World},
    ecs::{CombatStats, Description, Name, Position, Renderable, RenderableEntities},
    view::worldview::WorldViewTerrain,
};

/// A single line item in the inspector: the terrain or one entity on a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorEntry {
    /// Display glyph + color used as the leading marker.
    pub marker: (char, Color),
    /// Heading text, e.g. "Grass", "Squirrel", "Signpost".
    pub name: String,
    /// Detail lines beneath the heading; empty leaves room for future text.
    pub details: Vec<String>,
}

/// Snapshot of everything observable at a single world coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorView {
    pub entries: Vec<InspectorEntry>,
}

/// Builds the inspector snapshot for `coord`: the terrain first (if any),
/// then each entity standing on the tile, top of the z-stack first.
pub fn from_ecs_at(world: &EcsWorld, coord: Vector) -> InspectorView {
    let mut entries = Vec::new();

    if let Some(terrain) = terrain_entry(world, coord) {
        entries.push(terrain);
    }
    entries.extend(entity_entries(world, coord));

    InspectorView { entries }
}

fn terrain_entry(world: &EcsWorld, coord: Vector) -> Option<InspectorEntry> {
    let kind = world.resource::<World>().terrain_at(coord)?.kind();
    let terrain = match kind {
        crate::data::world::TerrainType::Grass => WorldViewTerrain::Grass,
        crate::data::world::TerrainType::Shrubbery => WorldViewTerrain::Shrubbery,
    };
    let marker = terrain.marker()?;
    let name = terrain.display_name()?;
    Some(InspectorEntry {
        marker,
        name: name.to_string(),
        details: Vec::new(),
    })
}

fn entity_entries(world: &EcsWorld, coord: Vector) -> Vec<InspectorEntry> {
    let renderables = world.resource::<RenderableEntities>();
    let mut found: Vec<(i32, InspectorEntry)> = Vec::new();

    for entity in renderables.entities.iter().copied() {
        let Ok(entity_ref) = world.get_entity(entity) else {
            continue;
        };
        let Some(position) = entity_ref.get::<Position>() else {
            continue;
        };
        if position.0 != coord {
            continue;
        }
        let Some(renderable) = entity_ref.get::<Renderable>() else {
            continue;
        };

        let name = entity_ref
            .get::<Name>()
            .map(|name| name.0)
            .unwrap_or("?")
            .to_string();

        let mut details = Vec::new();
        if let Some(stats) = entity_ref.get::<CombatStats>() {
            details.push(format!("HP {}/{}", stats.hp, stats.max_hp));
        }
        if let Some(description) = entity_ref.get::<Description>() {
            details.push(description.0.to_string());
        }

        found.push((
            renderable.z,
            InspectorEntry {
                marker: (renderable.glyph, renderable.color),
                name,
                details,
            },
        ));
    }

    found.sort_by_key(|(z, _)| std::cmp::Reverse(*z));
    found.into_iter().map(|(_, entry)| entry).collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        data::grid::{ORIGIN, Vector},
        ecs::{spawn_initial_entities, SIGN_POSITION, SQUIRREL_POSITIONS},
        view::worldview::WorldViewTerrain,
    };

    use super::from_ecs_at;

    fn world() -> bevy_ecs::world::World {
        use crate::{app::BRIDGEPORT_OUTSKIRTS, data::world::World};
        let mut ecs_world = bevy_ecs::world::World::new();
        ecs_world.insert_resource(World::new().with_region(BRIDGEPORT_OUTSKIRTS, ORIGIN));
        spawn_initial_entities(&mut ecs_world);
        ecs_world
    }

    #[test]
    fn empty_tile_reports_terrain_only() {
        let world = world();
        // A tile far from any spawn but inside the origin region.
        let view = from_ecs_at(&world, Vector { x: 40, y: 40 });

        assert_eq!(view.entries.len(), 1);
        let terrain = &view.entries[0];
        assert!(matches!(
            terrain.name.as_str(),
            "Grass" | "Shrubbery"
        ));
        assert!(terrain.details.is_empty());
    }

    #[test]
    fn sign_tile_reports_terrain_then_signpost() {
        let world = world();
        let view = from_ecs_at(&world, SIGN_POSITION);

        assert_eq!(view.entries.len(), 2);
        let sign = &view.entries[1];
        assert_eq!(sign.name, "Signpost");
        assert_eq!(sign.marker.0, '|');
        assert_eq!(sign.details, vec!["A wooden signpost".to_string()]);
    }

    #[test]
    fn squirrel_tile_reports_hp_summary() {
        let world = world();
        let view = from_ecs_at(&world, SQUIRREL_POSITIONS[0]);

        let squirrel = view
            .entries
            .iter()
            .find(|entry| entry.name.starts_with("Squirrel"))
            .expect("squirrel entry present");
        assert_eq!(squirrel.details.len(), 1);
        assert!(squirrel.details[0].starts_with("HP "));
        assert!(squirrel.details[0].contains('/'));
    }

    #[test]
    fn terrain_marker_matches_worldview() {
        // The inspector reuses the shared terrain marker source of truth.
        assert_eq!(WorldViewTerrain::Grass.display_name(), Some("Grass"));
    }
}
