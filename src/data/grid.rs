use std::collections::HashMap;

pub const ORIGIN: Vector = Vector { x: 0, y: 0 };

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self {
        let len = width.checked_mul(height).expect("grid dimensions overflow");
        Self {
            width,
            height,
            cells: vec![default; len],
        }
    }
}

impl<T> Grid<T> {
    pub fn from_fn(width: usize, height: usize, mut f: impl FnMut(usize, usize) -> T) -> Self {
        let len = width.checked_mul(height).expect("grid dimensions overflow");
        let mut cells = Vec::with_capacity(len);
        for y in 0..height {
            for x in 0..width {
                cells.push(f(x, y));
            }
        }

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            self.cells.get(y * self.width + x)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SparseGrid<T> {
    cells: HashMap<Vector, T>,
}

impl<T> SparseGrid<T> {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    pub fn add(&mut self, coord: Vector, value: T) -> &mut Self {
        self.cells.insert(coord, value);
        self
    }

    pub fn get(&self, coord: &Vector) -> Option<&T> {
        self.cells.get(coord)
    }
}

#[cfg(test)]
mod tests {
    use super::Grid;

    #[test]
    fn dynamic_grid_tracks_dimensions_and_bounds() {
        let grid = Grid::new(3, 2, '.');

        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.get(0, 0), Some(&'.'));
        assert_eq!(grid.get(2, 1), Some(&'.'));
        assert_eq!(grid.get(3, 0), None);
        assert_eq!(grid.get(0, 2), None);
    }

    #[test]
    fn dynamic_grid_can_be_built_from_coordinates() {
        let grid = Grid::from_fn(2, 2, |x, y| x + y * 10);

        assert_eq!(grid.get(0, 0), Some(&0));
        assert_eq!(grid.get(1, 0), Some(&1));
        assert_eq!(grid.get(0, 1), Some(&10));
        assert_eq!(grid.get(1, 1), Some(&11));
    }
}
