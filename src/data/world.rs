use crate::data::grid::Vector;

use super::grid::{Grid, SparseGrid};

const WORLD_REGION_WIDTH: usize = 1024;
const WORLD_REGION_HEIGHT: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub enum TerrainType {
    Grass
}


#[derive(Debug, Clone, Copy)]
pub struct Terrain {
    kind: TerrainType
}

#[derive(Debug)]
pub struct Region {
    name: &'static str,
    terrain: Grid<Terrain, WORLD_REGION_WIDTH, WORLD_REGION_HEIGHT>
}

impl Region {
    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug)]
pub struct World {
    regions: SparseGrid<Region>
}

impl World {
    pub fn new() -> Self {
        Self {
            regions: SparseGrid::new()
        }
    }

    pub fn with_region(mut self, name: &'static str, centered_at: Vector) -> Self {
        self.regions.add(
            centered_at,
            Region {
                name: name,
                terrain: Grid::new(Terrain { kind: TerrainType::Grass })
            }
        );
        self
    }

    pub fn region_at(&self, coord: Vector) -> &Region {
        self.regions.get(&coord).expect("No region at given coordinates")
    }
}