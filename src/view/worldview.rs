use crate::data::{
    grid::{Grid, Vector},
    world::{TerrainType, World},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldViewTerrain {
    Blank,
    Grass,
    Shrubbery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldView {
    pub terrain: Grid<WorldViewTerrain>,
    pub current_region_name: &'static str,
}

pub fn from_world(world: &World, center: Vector, size: Vector) -> WorldView {
    let width = size.x.max(0) as usize;
    let height = size.y.max(0) as usize;
    let current_region_name = world
        .region_containing(center)
        .map(|region| region.name())
        .unwrap_or("");

    let terrain = Grid::from_fn(width, height, |x, y| {
        let coord = sample_coord(center, width, height, x, y);
        match world.terrain_at(coord).map(|terrain| terrain.kind()) {
            Some(TerrainType::Grass) => WorldViewTerrain::Grass,
            Some(TerrainType::Shrubbery) => WorldViewTerrain::Shrubbery,
            None => WorldViewTerrain::Blank,
        }
    });

    WorldView {
        terrain,
        current_region_name,
    }
}

fn sample_coord(center: Vector, width: usize, height: usize, x: usize, y: usize) -> Vector {
    Vector {
        x: center.x.saturating_add(x as i32 - (width / 2) as i32),
        y: center.y.saturating_add(y as i32 - (height / 2) as i32),
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{
        grid::{ORIGIN, Vector},
        world::World,
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
                    Some(WorldViewTerrain::Grass) => grass_count += 1,
                    Some(WorldViewTerrain::Shrubbery) => shrubbery_count += 1,
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
        let inside = from_world(&world, Vector { x: 511, y: 0 }, Vector { x: 1, y: 1 });
        let outside = from_world(&world, Vector { x: 512, y: 0 }, Vector { x: 1, y: 1 });

        assert_eq!(inside.current_region_name, "Bridgeport Outskirts");
        assert_eq!(inside.terrain.get(0, 0), Some(&WorldViewTerrain::Grass));
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
