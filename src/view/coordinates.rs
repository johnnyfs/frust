use crate::data::grid::Vector;

pub fn viewport_origin(center: Vector, size: Vector) -> Vector {
    Vector {
        x: center.x.saturating_sub(size.x.max(0) / 2),
        y: center.y.saturating_sub(size.y.max(0) / 2),
    }
}

pub fn local_to_world(center: Vector, size: Vector, local: Vector) -> Vector {
    let origin = viewport_origin(center, size);
    Vector {
        x: origin.x.saturating_add(local.x),
        y: origin.y.saturating_add(local.y),
    }
}

pub fn world_to_local(center: Vector, size: Vector, coord: Vector) -> Option<Vector> {
    let width = size.x.max(0) as i64;
    let height = size.y.max(0) as i64;
    let origin = viewport_origin(center, size);
    let x = coord.x as i64 - origin.x as i64;
    let y = coord.y as i64 - origin.y as i64;

    if x >= 0 && x < width && y >= 0 && y < height {
        Some(Vector {
            x: x as i32,
            y: y as i32,
        })
    } else {
        None
    }
}
