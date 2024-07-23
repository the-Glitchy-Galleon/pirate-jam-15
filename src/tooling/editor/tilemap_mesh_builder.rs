use crate::framework::{
    prelude::*,
    tilemap::{SLOPE_HEIGHT, TILE_SIZE_X, TILE_SIZE_Y, WALL_HEIGHT},
    tileset::Tileset,
};
use bevy::{prelude::*, utils::HashSet};
use bevy_rapier3d::geometry::{Collider, ComputedColliderShape, VHACDParameters};
use itertools::izip;
use std::collections::VecDeque;

pub struct RawMeshBuilder<'a> {
    tilemap: &'a Tilemap,
}
impl<'a> RawMeshBuilder<'a> {
    pub fn new(tilemap: &'a Tilemap) -> Self {
        Self { tilemap }
    }

    pub fn make_ground_mesh(&self, tileset: &Tileset) -> RawMesh {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut uvs = vec![];
        let map = self.tilemap;

        let offset = -(map.size() * 0.5);
        let dims = map.dims();
        for y in 0..dims.y {
            for x in 0..dims.x {
                let fid = y * dims.x + x;
                let vc = [
                    UVec2::new(x + 0, y + 0),
                    UVec2::new(x + 1, y + 0),
                    UVec2::new(x + 1, y + 1),
                    UVec2::new(x + 0, y + 1),
                ];
                for (coord, vid) in izip!(vc, vc.map(|c| (dims.x + 1) * c.y + c.x)) {
                    let vert = &map.vert_data()[vid as usize];
                    vertices.push([
                        offset.x + coord.x as f32 * TILE_SIZE_X,
                        vert.elevation as f32 * SLOPE_HEIGHT,
                        offset.y + coord.y as f32 * TILE_SIZE_Y,
                    ]);
                }
                let face = &map.face_data()[fid as usize];
                let tile = tileset.id_to_uvs_or_default(face.tile_id);
                uvs.extend(&[
                    [tile.min.x, tile.min.y],
                    [tile.max.x, tile.min.y],
                    [tile.max.x, tile.max.y],
                    [tile.min.x, tile.max.y],
                ]);

                indices.extend(&[0, 3, 2, 2, 1, 0].map(|i| i + fid * 4));
            }
        }
        let normals = vertex_normals(&vertices, &indices);

        RawMesh {
            vertices,
            indices,
            normals,
            uvs,
        }
    }

    fn flood_fill(&self, coord: UVec2, visited: &mut HashSet<UVec2>) -> (u32, Vec<UVec2>) {
        if !self.tilemap.face_grid().is_coord_in_grid(coord) {
            error!("coords outside grid: {coord}");
            return (0, vec![]);
        }
        let map = self.tilemap;
        let mut queue = VecDeque::new();
        let mut island = Vec::new();
        let wall_height = map
            .face_unchecked(map.face_grid().coord_to_id(coord))
            .data
            .wall_height;

        queue.push_back(coord);
        visited.insert(coord);

        while let Some(coord) = queue.pop_front() {
            island.push(coord);

            for coord in map
                .face_grid()
                .neighbor_coords_4(coord)
                .into_iter()
                .flatten()
            {
                if !map.face_grid().is_coord_in_grid(coord) {
                    error!("neighbor coords outside grid: {coord}");
                    continue;
                }

                let height = map
                    .face_unchecked(map.face_grid().coord_to_id(coord))
                    .data
                    .wall_height;
                if !visited.contains(&coord) && height == wall_height {
                    visited.insert(coord);
                    queue.push_back(coord);
                }
            }
        }

        (wall_height, island)
    }

    // Todo: maybe split by concavity aswell to save collider-generation time
    fn find_islands(&self) -> Vec<(u32, Vec<UVec2>)> {
        let mut visited = HashSet::new();
        let mut islands = Vec::new();
        let dims = self.tilemap.dims();
        for y in 0..dims.y {
            for x in 0..dims.x {
                let coord = UVec2::new(x, y);
                if !visited.contains(&coord) {
                    let island = self.flood_fill(coord, &mut visited);
                    if !island.1.is_empty() {
                        islands.push(island);
                    }
                }
            }
        }
        islands
    }

    fn make_island_mesh(&self, coords: &[UVec2], height: u32, tileset: &Tileset) -> RawMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        let mut idx = 0;

        let offset = -(self.tilemap.size() * 0.5);
        for coord in coords {
            let fid = self.tilemap.face_grid().coord_to_id(*coord);
            let face = &self.tilemap.face_data()[fid as usize];
            let x = coord.x as f32 + offset.x;
            let z = coord.y as f32 + offset.y;

            let y_inc = ((self.tilemap.face_base_elevation(fid) as f32 * SLOPE_HEIGHT)
                / WALL_HEIGHT) as u32;
            let base_y = y_inc as f32 * WALL_HEIGHT;

            for h in 0..height {
                let y = base_y + h as f32 * WALL_HEIGHT;
                let ny = base_y + (h + 1) as f32 * WALL_HEIGHT;

                const S: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
                const D: [(f32, f32); 4] = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
                const T: [usize; 4] = [1, 2, 3, 0];
                for ((sx, sz), t, (dx, dz)) in izip!(S, T, D) {
                    let x = x + sx;
                    let z = z + sz;
                    let nx = x + dx;
                    let nz = z + dz;

                    vertices.extend(&[[x, y, z], [nx, y, nz], [nx, ny, nz], [x, ny, z]]);
                    normals.extend(&[[dx, 0.0, dz]; 4]);

                    let tile =
                        tileset.id_to_uvs_or_default(face.wall_side_tile_ids[t + (h * 4) as usize]);
                    uvs.extend(&[
                        [tile.max.x, tile.max.y],
                        [tile.min.x, tile.max.y],
                        [tile.min.x, tile.min.y],
                        [tile.max.x, tile.min.y],
                    ]);
                    indices.extend(&[idx + 3, idx + 2, idx + 0, idx + 2, idx + 1, idx + 0]);
                    idx += 4;
                }
            }

            // Top face
            let y = base_y + height as f32 * WALL_HEIGHT;
            vertices.extend(&[
                [x, y, z],
                [x + 1.0, y, z],
                [x + 1.0, y, z + 1.0],
                [x, y, z + 1.0],
            ]);
            normals.extend(&[[0.0, 1.0, 0.0]; 4]);

            let tile = tileset.id_to_uvs_or_default(face.wall_top_tile_id);
            uvs.extend(&[
                [tile.min.x, tile.min.y],
                [tile.max.x, tile.min.y],
                [tile.max.x, tile.max.y],
                [tile.min.x, tile.max.y],
            ]);

            let base_index = idx;
            indices.extend(&[
                base_index + 3,
                base_index + 2,
                base_index + 0,
                base_index + 2,
                base_index + 1,
                base_index + 0,
            ]);
            idx += 4;
        }

        RawMesh {
            vertices,
            indices,
            normals,
            uvs,
        }
    }

    pub fn make_wall_meshes(&self, tileset: &Tileset) -> Vec<RawMesh> {
        let islands = self.find_islands();
        let mut meshes = vec![];

        for (height, coords) in islands {
            if height == 0 {
                continue;
            }
            let mesh = self.make_island_mesh(&coords, height, tileset);
            meshes.push(mesh);
        }

        meshes
    }

    pub fn build_rapier_heightfield_collider(&self) -> Collider {
        let map = self.tilemap;
        let mut heights = vec![];
        let dims = self.tilemap.dims();
        for y in 0..dims.y + 1 {
            for x in 0..dims.x + 1 {
                let vid = y * (dims.x + 1) + x;
                heights.push(map.vert_data()[vid as usize].elevation as f32);
            }
        }

        let size = map.size();
        let scale = Vec3::new(size.x, SLOPE_HEIGHT, size.y);
        let dims = map.dims();
        Collider::heightfield(heights, (dims.y + 1) as usize, (dims.x + 1) as usize, scale)
    }
}

pub fn build_rapier_convex_collider_for_preview(mesh: &Mesh) -> Collider {
    Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap()
}

/// Really should use release build for this
pub fn build_rapier_convex_collider_for_export(mesh: &Mesh) -> Collider {
    Collider::from_bevy_mesh(
        mesh,
        &ComputedColliderShape::ConvexDecomposition(VHACDParameters {
            resolution: 256,
            ..Default::default()
        }),
    )
    .unwrap()
}

fn triangle_normal(verts: [[f32; 3]; 3]) -> [f32; 3] {
    let (v0, v1, v2) = (verts[0], verts[1], verts[2]);
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    normalize_or_zero(cross_product(e1, e2))
}

#[rustfmt::skip]
fn cross_product(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {[
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
]}

fn normalize_or_zero(v: [f32; 3]) -> [f32; 3] {
    match (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() {
        0.0 => [0.0, 0.0, 0.0],
        len => [v[0] / len, v[1] / len, v[2] / len],
    }
}

fn vertex_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
    let mut counts = vec![0; vertices.len()];

    for i in (0..indices.len()).step_by(3) {
        let idx = [indices[i + 0], indices[i + 1], indices[i + 2]].map(|i| i as usize);
        let verts = [vertices[idx[0]], vertices[idx[1]], vertices[idx[2]]];
        let normal = triangle_normal(verts);
        for idx in idx {
            normals[idx][0] += normal[0];
            normals[idx][1] += normal[1];
            normals[idx][2] += normal[2];
            counts[idx] += 1;
        }
    }
    for i in 0..normals.len() {
        if counts[i] > 0 {
            normals[i][0] /= counts[i] as f32;
            normals[i][1] /= counts[i] as f32;
            normals[i][2] /= counts[i] as f32;
            normals[i] = normalize_or_zero(normals[i]);
        }
    }
    normals
}

#[test]
fn checking() {
    let tilemap = Tilemap::new(UVec2::new(3, 3)).unwrap();
    assert_eq!(
        tilemap.face_grid().neighbor_coords_4(UVec2::new(0, 0)),
        [None, Some(UVec2::new(1, 0)), None, Some(UVec2::new(0, 1)),]
    );
}
