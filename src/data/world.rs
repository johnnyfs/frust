use super::grid::{Coord, Grid, SparseGrid};

enum TerrainType {
    Grass
}

struct Terrain {
    kind: TerrainType
}

struct Region {
    name: &'static str,
    terrain: Grid<Terrain>
}

struct World {
    regions: SparseGrid<Region>
}