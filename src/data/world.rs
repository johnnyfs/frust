use bevy_ecs::prelude::Resource;

use crate::data::grid::{Grid, SparseGrid, Vector};

pub const WORLD_REGION_WIDTH: usize = 320;
pub const WORLD_REGION_HEIGHT: usize = 192;
const SHRUBBERY_DENSITY: i64 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Grass,
    Shrubbery,
    Forest,
    Path,
    Road,
    River,
    Pond,
}

/// All terrain variants in stable id order.
pub const TERRAIN_TYPES: [TerrainType; 7] = [
    TerrainType::Grass,
    TerrainType::Shrubbery,
    TerrainType::Forest,
    TerrainType::Path,
    TerrainType::Road,
    TerrainType::River,
    TerrainType::Pond,
];

impl TerrainType {
    /// Stable numeric id used by the resource encoding.
    pub fn id(self) -> u8 {
        match self {
            TerrainType::Grass => 0,
            TerrainType::Shrubbery => 1,
            TerrainType::Forest => 2,
            TerrainType::Path => 3,
            TerrainType::Road => 4,
            TerrainType::River => 5,
            TerrainType::Pond => 6,
        }
    }

    /// Resolves a terrain variant from its stable id.
    pub fn from_id(id: u8) -> Option<TerrainType> {
        TERRAIN_TYPES.into_iter().find(|terrain| terrain.id() == id)
    }

    /// Human-readable terrain name.
    pub fn name(self) -> &'static str {
        match self {
            TerrainType::Grass => "Grass",
            TerrainType::Shrubbery => "Shrubbery",
            TerrainType::Forest => "Forest",
            TerrainType::Path => "Path",
            TerrainType::Road => "Road",
            TerrainType::River => "River",
            TerrainType::Pond => "Pond",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terrain {
    kind: TerrainType,
}

impl Terrain {
    pub fn new(kind: TerrainType) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> TerrainType {
        self.kind
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    id: String,
    name: String,
    terrain: Grid<Terrain>,
}

impl Region {
    /// Creates a region from an explicit terrain grid.
    pub fn new(id: impl Into<String>, name: impl Into<String>, terrain: Grid<Terrain>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            terrain,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn terrain(&self) -> &Grid<Terrain> {
        &self.terrain
    }

    pub fn terrain_at(&self, x: usize, y: usize) -> Option<&Terrain> {
        self.terrain.get(x, y)
    }

    pub fn set_terrain_at(&mut self, x: usize, y: usize, kind: TerrainType) -> bool {
        self.terrain.set(x, y, Terrain::new(kind))
    }
}

#[derive(Debug, Resource)]
pub struct World {
    regions: SparseGrid<Region>,
}

impl World {
    pub fn new() -> Self {
        Self {
            regions: SparseGrid::new(),
        }
    }

    pub fn with_region(mut self, name: impl Into<String>, centered_at: Vector) -> Self {
        let name = name.into();
        let id = slugify(&name);
        self.regions.add(
            centered_at,
            Region::new(
                id,
                name,
                Grid::from_fn(WORLD_REGION_WIDTH, WORLD_REGION_HEIGHT, |x, y| {
                    Terrain::new(terrain_kind_for_tile(centered_at, x, y))
                }),
            ),
        );
        self
    }

    /// Inserts (or replaces) a region centered at the given coordinate.
    pub fn insert_region(&mut self, centered_at: Vector, region: Region) -> &mut Self {
        self.regions.add(centered_at, region);
        self
    }

    pub fn with_inserted_region(mut self, centered_at: Vector, region: Region) -> Self {
        self.insert_region(centered_at, region);
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

    /// Paints terrain at a world coordinate. Returns true when the coordinate
    /// falls inside a loaded region (painting outside is a no-op).
    pub fn set_terrain(&mut self, coord: Vector, kind: TerrainType) -> bool {
        let region_center = region_center_for(coord);
        let Some((x, y)) = local_tile_for(coord, region_center) else {
            return false;
        };
        let Some(region) = self.regions.get_mut(&region_center) else {
            return false;
        };
        region.set_terrain_at(x, y, kind)
    }
}

/// Converts a region name into a stable lowercase, underscore-separated id.
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    slug
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
