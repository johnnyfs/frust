use crate::data::grid::{Grid, SparseGrid, Vector};

pub const WORLD_REGION_WIDTH: usize = 320;
pub const WORLD_REGION_HEIGHT: usize = 192;
const SHRUBBERY_DENSITY: i64 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Grass,
    Shrubbery,
}

#[derive(Debug, Clone, Copy)]
pub struct Terrain {
    kind: TerrainType,
}

impl Terrain {
    pub fn kind(&self) -> TerrainType {
        self.kind
    }
}

#[derive(Debug)]
pub struct Region {
    name: &'static str,
    terrain: Grid<Terrain>,
}

impl Region {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn terrain_at(&self, x: usize, y: usize) -> Option<&Terrain> {
        self.terrain.get(x, y)
    }
}

#[derive(Debug)]
pub struct World {
    regions: SparseGrid<Region>,
    player_position: Vector,
}

impl World {
    pub fn new() -> Self {
        Self {
            regions: SparseGrid::new(),
            player_position: Vector { x: 0, y: 0 },
        }
    }

    pub fn with_region(mut self, name: &'static str, centered_at: Vector) -> Self {
        self.regions.add(
            centered_at,
            Region {
                name: name,
                terrain: Grid::from_fn(WORLD_REGION_WIDTH, WORLD_REGION_HEIGHT, |x, y| Terrain {
                    kind: terrain_kind_for_tile(centered_at, x, y),
                }),
            },
        );
        self
    }

    pub fn region_at(&self, coord: Vector) -> Option<&Region> {
        self.regions.get(&coord)
    }

    pub fn region_containing(&self, coord: Vector) -> Option<&Region> {
        self.region_at(region_center_for(coord))
    }

    pub fn terrain_at(&self, coord: Vector) -> Option<&Terrain> {
        let region_center = region_center_for(coord);
        let region = self.region_at(region_center)?;
        let (x, y) = local_tile_for(coord, region_center)?;
        region.terrain_at(x, y)
    }

    pub fn player_position(&self) -> Vector {
        self.player_position
    }

    pub fn move_player_by(&mut self, delta: Vector) {
        self.player_position = Vector {
            x: self.player_position.x.saturating_add(delta.x),
            y: self.player_position.y.saturating_add(delta.y),
        };
    }
}

fn terrain_kind_for_tile(region_center: Vector, x: usize, y: usize) -> TerrainType {
    let left = region_center.x as i64 - WORLD_REGION_WIDTH as i64 / 2;
    let top = region_center.y as i64 - WORLD_REGION_HEIGHT as i64 / 2;
    let world_x = left + x as i64;
    let world_y = top + y as i64;
    let scatter = world_x
        .wrapping_mul(13)
        .wrapping_add(world_y.wrapping_mul(7))
        .wrapping_add(6)
        .rem_euclid(SHRUBBERY_DENSITY);

    if scatter == 0 {
        TerrainType::Shrubbery
    } else {
        TerrainType::Grass
    }
}

fn region_center_for(coord: Vector) -> Vector {
    Vector {
        x: region_axis_center(coord.x, WORLD_REGION_WIDTH),
        y: region_axis_center(coord.y, WORLD_REGION_HEIGHT),
    }
}

fn region_axis_center(value: i32, size: usize) -> i32 {
    let size = size as i32;
    let half = size / 2;
    value.saturating_add(half).div_euclid(size) * size
}

fn local_tile_for(coord: Vector, region_center: Vector) -> Option<(usize, usize)> {
    let left = region_center.x as i64 - WORLD_REGION_WIDTH as i64 / 2;
    let top = region_center.y as i64 - WORLD_REGION_HEIGHT as i64 / 2;
    let x = coord.x as i64 - left;
    let y = coord.y as i64 - top;

    if x >= 0 && x < WORLD_REGION_WIDTH as i64 && y >= 0 && y < WORLD_REGION_HEIGHT as i64 {
        Some((x as usize, y as usize))
    } else {
        None
    }
}
