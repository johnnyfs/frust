use ratatui::style::Color;

use crate::data::{
    grid::{Grid, Vector},
    world::{TerrainType, World},
};

use super::coordinates::local_to_world;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldViewTerrain {
    /// No loaded region covers this cell.
    Blank,
    /// A terrain tile from a loaded region.
    Filled(TerrainType),
}

/// Display glyph and color for a terrain tile. Single source of truth shared by
/// the viewport renderer and the tile inspector.
pub fn terrain_marker(kind: TerrainType) -> (char, Color) {
    let dark_green = Color::Rgb(0, 100, 0);
    let brown = Color::Rgb(139, 69, 19);
    match kind {
        TerrainType::Grass => ('.', Color::LightGreen),
        TerrainType::Shrubbery => ('*', dark_green),
        TerrainType::Forest => ('#', dark_green),
        TerrainType::Path => (':', brown),
        TerrainType::Road => (':', Color::DarkGray),
        TerrainType::River => ('=', Color::LightCyan),
        TerrainType::Pond => ('~', Color::LightCyan),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldView {
    pub terrain: Grid<WorldViewTerrain>,
    pub current_region_name: String,
}

pub fn from_world(world: &World, center: Vector, size: Vector) -> WorldView {
    let width = size.x.max(0) as usize;
    let height = size.y.max(0) as usize;
    let current_region_name = world
        .region_containing(center)
        .map(|region| region.name().to_string())
        .unwrap_or_default();

    let terrain = Grid::from_fn(width, height, |x, y| {
        let coord = local_to_world(
            center,
            size,
            Vector {
                x: x as i32,
                y: y as i32,
            },
        );
        match world.terrain_at(coord).map(|terrain| terrain.kind()) {
            Some(kind) => WorldViewTerrain::Filled(kind),
            None => WorldViewTerrain::Blank,
        }
    });

    WorldView {
        terrain,
        current_region_name,
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{
        grid::{ORIGIN, Vector},
        world::{TerrainType, WORLD_REGION_WIDTH, World},
    };

    use super::{WorldViewTerrain, from_world};

    #[test]
    fn origin_region_samples_grass_and_name() {
        let world = World::new().with_region("Bridgeport Outskirts", ORIGIN);
        let view = from_world(&world, ORIGIN, Vector { x: 40, y: 20 });

        assert_eq!(view.current_region_name, "Bridgeport Outskirts");
        assert_eq!(view.terrain.width(), 40);
        assert_eq!(view.terrain.height(), 20);
        let mut grass_count = 0;
        let mut shrubbery_count = 0;
        for y in 0..view.terrain.height() {
            for x in 0..view.terrain.width() {
                match view.terrain.get(x, y) {
                    Some(WorldViewTerrain::Filled(TerrainType::Grass)) => grass_count += 1,
                    Some(WorldViewTerrain::Filled(TerrainType::Shrubbery)) => shrubbery_count += 1,
                    other => panic!("expected region terrain, got {other:?}"),
                }
            }
        }
        assert!(grass_count > 0);
        assert!(shrubbery_count > 0);
    }

    #[test]
    fn missing_center_region_samples_blank_and_empty_name() {
        let world = World::new().with_region("Bridgeport Outskirts", ORIGIN);
        let view = from_world(&world, Vector { x: 1024, y: 0 }, Vector { x: 3, y: 3 });

        assert_eq!(view.current_region_name, "");
        for y in 0..view.terrain.height() {
            for x in 0..view.terrain.width() {
                assert_eq!(view.terrain.get(x, y), Some(&WorldViewTerrain::Blank));
            }
        }
    }

    #[test]
    fn origin_region_boundary_runs_from_negative_512_through_positive_511() {
        let world = World::new().with_region("Bridgeport Outskirts", ORIGIN);
        let half_width = WORLD_REGION_WIDTH as i32 / 2;
        let inside = from_world(
            &world,
            Vector {
                x: half_width - 1,
                y: 0,
            },
            Vector { x: 1, y: 1 },
        );
        let outside = from_world(
            &world,
            Vector {
                x: half_width,
                y: 0,
            },
            Vector { x: 1, y: 1 },
        );

        assert_eq!(inside.current_region_name, "Bridgeport Outskirts");
        assert!(matches!(
            inside.terrain.get(0, 0).copied(),
            Some(WorldViewTerrain::Filled(
                TerrainType::Grass | TerrainType::Shrubbery
            ))
        ));
        assert_eq!(outside.current_region_name, "");
        assert_eq!(outside.terrain.get(0, 0), Some(&WorldViewTerrain::Blank));
    }

    #[test]
    fn negative_sizes_clamp_to_empty_terrain_but_still_report_center_name() {
        let world = World::new().with_region("Bridgeport Outskirts", ORIGIN);
        let view = from_world(&world, ORIGIN, Vector { x: -1, y: -2 });

        assert_eq!(view.current_region_name, "Bridgeport Outskirts");
        assert_eq!(view.terrain.width(), 0);
        assert_eq!(view.terrain.height(), 0);
    }
}
