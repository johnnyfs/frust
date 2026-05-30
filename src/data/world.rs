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
    Clearing,
}

/// All terrain variants in stable id order.
pub const TERRAIN_TYPES: [TerrainType; 8] = [
    TerrainType::Grass,
    TerrainType::Shrubbery,
    TerrainType::Forest,
    TerrainType::Path,
    TerrainType::Road,
    TerrainType::River,
    TerrainType::Pond,
    TerrainType::Clearing,
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
            TerrainType::Clearing => 7,
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
            TerrainType::Clearing => "Clearing",
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

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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

    /// Paints a single cell, returning its previous terrain when the value
    /// actually changed (so edits can be recorded for undo). Returns `None`
    /// outside any region or when the cell already holds `kind`.
    pub fn replace_terrain(&mut self, coord: Vector, kind: TerrainType) -> Option<TerrainType> {
        let region_center = region_center_for(coord);
        let (x, y) = local_tile_for(coord, region_center)?;
        let region = self.regions.get_mut(&region_center)?;
        let old = region.terrain_at(x, y)?.kind();
        if old == kind {
            return None;
        }
        region.set_terrain_at(x, y, kind);
        Some(old)
    }

    /// Paints the selected terrain over the inclusive world-coordinate rectangle
    /// spanned by `a` and `b`, returning each changed cell with its prior
    /// terrain. Cells outside any loaded region are skipped.
    pub fn fill_rect_recording(
        &mut self,
        a: Vector,
        b: Vector,
        kind: TerrainType,
    ) -> Vec<(Vector, TerrainType)> {
        let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
        let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
        let mut edits = Vec::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                let coord = Vector { x, y };
                if let Some(old) = self.replace_terrain(coord, kind) {
                    edits.push((coord, old));
                }
            }
        }
        edits
    }

    /// Convenience wrapper: returns true when at least one cell was painted.
    pub fn fill_rect(&mut self, a: Vector, b: Vector, kind: TerrainType) -> bool {
        !self.fill_rect_recording(a, b, kind).is_empty()
    }

    /// Flood fills the contiguous run of like terrain reachable from `coord`
    /// (4-connected, bounded to its region) with `kind`, returning each changed
    /// cell with its prior terrain. Empty when the start is outside a region or
    /// already matches `kind`.
    pub fn flood_fill_recording(
        &mut self,
        coord: Vector,
        kind: TerrainType,
    ) -> Vec<(Vector, TerrainType)> {
        let region_center = region_center_for(coord);
        let Some((start_x, start_y)) = local_tile_for(coord, region_center) else {
            return Vec::new();
        };
        let Some(region) = self.regions.get_mut(&region_center) else {
            return Vec::new();
        };
        let Some(target) = region.terrain_at(start_x, start_y).map(|terrain| terrain.kind()) else {
            return Vec::new();
        };
        if target == kind {
            return Vec::new();
        }

        let (left, top) = region_origin(region_center);
        let width = region.terrain().width();
        let height = region.terrain().height();
        let mut stack = vec![(start_x, start_y)];
        let mut edits = Vec::new();
        while let Some((x, y)) = stack.pop() {
            match region.terrain_at(x, y).map(|terrain| terrain.kind()) {
                Some(current) if current == target => {}
                _ => continue,
            }
            region.set_terrain_at(x, y, kind);
            edits.push((
                Vector {
                    x: left + x as i32,
                    y: top + y as i32,
                },
                target,
            ));
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < width {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < height {
                stack.push((x, y + 1));
            }
        }
        edits
    }

    /// Convenience wrapper: returns true when at least one cell was filled.
    pub fn flood_fill(&mut self, coord: Vector, kind: TerrainType) -> bool {
        !self.flood_fill_recording(coord, kind).is_empty()
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

/// Top-left world coordinate of the region centered at `region_center`.
fn region_origin(region_center: Vector) -> (i32, i32) {
    (
        region_center.x - WORLD_REGION_WIDTH as i32 / 2,
        region_center.y - WORLD_REGION_HEIGHT as i32 / 2,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::grid::ORIGIN;

    fn all_grass_world() -> World {
        let mut world = World::new();
        let terrain = Grid::new(
            WORLD_REGION_WIDTH,
            WORLD_REGION_HEIGHT,
            Terrain::new(TerrainType::Grass),
        );
        world.insert_region(ORIGIN, Region::new("test", "Test", terrain));
        world
    }

    fn kind_at(world: &World, x: i32, y: i32) -> Option<TerrainType> {
        world.terrain_at(Vector { x, y }).map(|terrain| terrain.kind())
    }

    #[test]
    fn fill_rect_paints_inclusive_rectangle_in_any_corner_order() {
        let mut world = all_grass_world();
        assert!(world.fill_rect(
            Vector { x: 2, y: 3 },
            Vector { x: 0, y: 1 },
            TerrainType::Road
        ));
        for y in 1..=3 {
            for x in 0..=2 {
                assert_eq!(kind_at(&world, x, y), Some(TerrainType::Road));
            }
        }
        assert_eq!(kind_at(&world, 3, 3), Some(TerrainType::Grass));
    }

    #[test]
    fn flood_fill_is_bounded_by_differing_terrain() {
        let mut world = all_grass_world();
        // Ring of road around the origin pocket.
        world.fill_rect(Vector { x: -1, y: -1 }, Vector { x: 1, y: 1 }, TerrainType::Road);
        world.set_terrain(ORIGIN, TerrainType::Grass);

        assert!(world.flood_fill(ORIGIN, TerrainType::Forest));
        assert_eq!(kind_at(&world, 0, 0), Some(TerrainType::Forest));
        // The road ring stops the fill.
        assert_eq!(kind_at(&world, 1, 1), Some(TerrainType::Road));
        // Grass beyond the ring is untouched.
        assert_eq!(kind_at(&world, 5, 5), Some(TerrainType::Grass));
    }

    #[test]
    fn flood_fill_noop_when_target_matches_selection() {
        let mut world = all_grass_world();
        assert!(!world.flood_fill(ORIGIN, TerrainType::Grass));
    }

    #[test]
    fn fills_outside_a_region_are_noops() {
        let mut world = all_grass_world();
        let far = Vector { x: 100_000, y: 0 };
        assert!(!world.flood_fill(far, TerrainType::Road));
        assert!(!world.fill_rect(far, far, TerrainType::Road));
    }
}
