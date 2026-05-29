use std::collections::HashMap;

#[derive(Hash)]
pub struct Coord {
    x: u32,
    y: u32
}

pub struct Grid<T, const W: usize, const H: usize> {
    cells: [[T; W]; H],
}

impl<T, const W: usize, const H: usize> Grid<T, W, H> {
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.cells.get(y)?.get(x)
    }
}

pub struct SparseGrid<T> {
    cells: HashMap<Coord, T>
}