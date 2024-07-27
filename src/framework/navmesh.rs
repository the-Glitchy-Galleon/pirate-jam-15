use bevy::prelude::*;
use itertools::Itertools;
use polyanya::{Polygon, Vertex};

pub struct ObjectObstacle {
    pub coord: UVec2, // top-left
    pub dims: UVec2,
}

/// Todo: Call this when an obstacle object is destroyed to recreate the navmesh.
pub fn create_grid_mesh_with_holes(
    dims: UVec2,
    walls: &[bool],
    objects: &[ObjectObstacle],
    padding: f32,
) -> (Vec<Vertex>, Vec<Polygon>) {
    let mut vertices = vec![None; ((dims.x + 1) * (dims.y + 1)) as usize];
    let mut polygons = vec![None; ((dims.x) * (dims.y)) as usize];

    let offset = dims.as_vec2() * -0.5;

    let is_hole = |x: u32, y: u32| -> bool {
        if walls[(y * dims.x + x) as usize] {
            return true;
        }
        for obj in objects {
            if x >= obj.coord.x
                && x < obj.coord.x + obj.dims.x
                && y >= obj.coord.y
                && y < obj.coord.y + obj.dims.y
            {
                return true;
            }
        }
        false
    };

    let mut num_polys = 0;
    for y in 0..dims.y {
        for x in 0..dims.x {
            if !is_hole(x, y) {
                polygons[(y * dims.x + x) as usize] = Some((num_polys, vec![]));
                num_polys += 1;
            }
        }
    }

    let mut num_verts = 0;
    for y in 0..=dims.y {
        for x in 0..=dims.x {
            let coord = IVec2::new(x as i32, y as i32);
            let mut list = vec![];
            let mut holes = [false; 4];

            const VERT_TO_POLY_CCW: [IVec2; 4] = [
                IVec2::new(-1, -1),
                IVec2::new(-1, 0),
                IVec2::new(0, 0),
                IVec2::new(0, -1),
            ];
            for (i, d) in VERT_TO_POLY_CCW.into_iter().enumerate() {
                let coord = coord + d;
                if coord.x < 0
                    || coord.y < 0
                    || coord.x as u32 >= dims.x
                    || coord.y as u32 >= dims.y
                {
                    holes[i] = true;
                } else {
                    let id = (coord.y as u32 * dims.x + coord.x as u32) as usize;
                    match polygons[id] {
                        Some((id, _)) => {
                            list.push(id as isize);
                        }
                        None => {
                            holes[i] = true;
                        }
                    }
                }
            }
            // brain:
            match (holes[0], holes[1], holes[2], holes[3]) {
                (true, true, true, true) => list.push(-1),
                (true, true, true, false) => list.insert(0, -1),
                (true, true, false, true) => list.insert(0, -1),
                (true, true, false, false) => list.insert(0, -1),
                (true, false, true, true) => list.insert(0, -1),
                (true, false, true, false) => {
                    list.insert(1, -1);
                    list.insert(0, -1)
                }
                (true, false, false, true) => list.insert(0, -1),
                (true, false, false, false) => list.insert(0, -1),
                (false, true, true, true) => list.push(-1),
                (false, true, true, false) => list.insert(1, -1),
                (false, true, false, true) => {
                    list.push(-1);
                    list.insert(0, -1);
                }
                (false, true, false, false) => list.insert(1, -1),
                (false, false, true, true) => list.push(-1),
                (false, false, true, false) => list.insert(2, -1),
                (false, false, false, true) => list.push(-1),
                (false, false, false, false) => {}
            }

            let mut pad = Vec2::ZERO;
            if holes[0] || holes[1] {
                pad.x += padding
            }
            if holes[1] || holes[2] {
                pad.y -= padding;
            }
            if holes[2] || holes[3] {
                pad.x -= padding;
            }
            if holes[3] || holes[0] {
                pad.y += padding;
            }

            let pos = Vec2::new(coord.x as f32, coord.y as f32) + offset + pad;

            let id = (y * (dims.x + 1) + x) as usize;
            vertices[id] = Some((num_verts, pos, list));
            num_verts += 1
        }
    }
    dbg!(&num_verts);

    for y in 0..dims.y {
        for x in 0..dims.x {
            let coord = IVec2::new(x as i32, y as i32);
            match &mut polygons[(y * dims.x + x) as usize] {
                Some((_, list)) => {
                    const POLY_TO_VERT_CLOCKWISE_QUESTIONMARK_EXCLAMATIONMARK: [IVec2; 4] = [
                        IVec2::new(1, 0),
                        IVec2::new(1, 1),
                        IVec2::new(0, 1),
                        IVec2::new(0, 0),
                    ];
                    for d in POLY_TO_VERT_CLOCKWISE_QUESTIONMARK_EXCLAMATIONMARK {
                        let coord = coord + d;
                        let id = (coord.y as u32 * (dims.x + 1) + coord.x as u32) as usize;
                        if let Some((id, _, _)) = vertices[id] {
                            list.push(id as u32);
                        }
                    }
                }
                None => {}
            }
        }
    }

    let vertices = vertices
        .into_iter()
        .flatten()
        .map(|(_, pos, polys)| Vertex::new(pos, polys))
        .collect_vec();

    let polygons = polygons
        .into_iter()
        .flatten()
        .map(|(_, vertices)| Polygon::new(vertices, false))
        .collect_vec();

    (vertices, polygons)
}
