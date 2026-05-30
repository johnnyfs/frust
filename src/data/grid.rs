use std::collections::HashMap;

pub const ORIGIN: Vector = Vector { x: 0, y: 0 };

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Vector {
    pub x: u32,
    pub y: u32
}

#[derive(Debug)]
pub struct Grid<T, const W: usize, const H: usize> {
    cells: [[T; W]; H],
}

impl<T, const W: usize, const H: usize> Grid<T, W, H> {
    pub fn new(default: T) -> Self where T: Copy {
        Self {
            cells: [[default; W]; H]
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.cells.get(y)?.get(x)
    }
}

#[derive(Debug)]
pub struct SparseGrid<T> {
    cells: HashMap<Vector, T>
}

impl<T> SparseGrid<T> {
    pub fn new() -> Self {
        Self { cells: HashMap::new() }
    }

    pub fn add(&mut self, coord: Vector, value: T) -> &mut Self {
        self.cells.insert(coord, value);
        self
    }

    pub fn get(&self, coord: &Vector) -> Option<&T> {
        self.cells.get(coord)
    }
}