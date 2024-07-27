use crate::framework::grid::Anchor2;
use crate::framework::grid::Grid;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const TILE_SIZE_X: f32 = 1.0;
pub const TILE_SIZE_Y: f32 = 1.0;
pub const TILE_DIMS: Vec2 = Vec2::new(TILE_SIZE_X, TILE_SIZE_Y);
pub const SLOPE_HEIGHT: f32 = 0.334;
// pub const SLOPE_HEIGHT: f32 = 1.0;
pub const WALL_HEIGHT: f32 = 1.0;
pub const MAX_NUM_WALLS: usize = 6;

#[derive(Default, Clone, Reflect, Serialize, Deserialize)]
pub struct FaceData {
    pub wall_height: u32,
    pub wall_top_tile_id: u32,
    pub wall_side_tile_ids: [u32; 4 * MAX_NUM_WALLS],
    pub tile_id: u32,
}
impl FaceData {
    pub const DEFAULT: FaceData = FaceData {
        wall_height: 0,
        tile_id: 0,
        wall_top_tile_id: 0,
        wall_side_tile_ids: [0; 4 * MAX_NUM_WALLS],
    };
}

#[derive(Default, Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct VertData {
    pub elevation: u32,
}

impl VertData {
    pub const DEFAULT: VertData = VertData { elevation: 0 };
}

#[derive(Asset, Clone, Reflect, Serialize, Deserialize)]
pub struct Tilemap {
    face_grid: Grid,
    vert_grid: Grid,
    face_data: Vec<FaceData>,
    vert_data: Vec<VertData>,
}

impl Tilemap {
    pub fn new(dims: UVec2, start_elevation: u32) -> Option<Self> {
        let vert_dims = dims + UVec2::ONE;
        Some(Self {
            face_grid: Grid::new(dims)?,
            vert_grid: Grid::new(vert_dims)?,
            face_data: vec![FaceData::DEFAULT; (dims.element_product()) as usize],
            vert_data: vec![
                VertData {
                    elevation: start_elevation
                };
                (vert_dims.element_product()) as usize
            ],
        })
    }

    pub fn dims(&self) -> UVec2 {
        self.face_grid.dims()
    }

    pub fn single_face_data(&self, fid: u32) -> Option<&FaceData> {
        self.face_data.get(fid as usize)
    }
    pub fn single_face_data_mut(&mut self, fid: u32) -> Option<&mut FaceData> {
        self.face_data.get_mut(fid as usize)
    }
    pub fn face_data(&self) -> &[FaceData] {
        &self.face_data
    }
    pub fn face_data_mut(&mut self) -> &mut [FaceData] {
        &mut self.face_data
    }
    pub fn vert_data(&self) -> &[VertData] {
        &self.vert_data
    }

    pub fn vert_data_mut(&mut self) -> &mut [VertData] {
        &mut self.vert_data
    }

    pub fn vert_iter(&self) -> VertPosIter {
        VertPosIter::new(
            &self.vert_data,
            self.vert_grid.dims().x,
            -(self.size() * 0.5),
        )
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(
            self.face_grid.dims().x as f32 * TILE_SIZE_X,
            self.face_grid.dims().y as f32 * TILE_SIZE_Y,
        )
    }

    pub fn face(&self, fid: u32) -> Option<Face> {
        self.face_grid.is_id_in_grid(fid).then_some(Face {
            fid,
            data: &self.face_data[fid as usize],
        })
    }
    pub fn face_unchecked(&self, fid: u32) -> Face {
        Face {
            fid,
            data: &self.face_data[fid as usize],
        }
    }

    pub fn face_neighbors(&self, fid: u32) -> NeighboringFaces4 {
        let coord = self.face_grid.id_to_coord(fid);
        let coords = self.face_grid.neighbor_coords_4(coord);
        NeighboringFaces4 {
            faces: coords.map(|c| c.and_then(|c| self.face(self.face_grid.coord_to_id(c)))),
        }
    }
    pub fn pos_to_face_id(&self, x: f32, z: f32) -> Option<u32> {
        let dims = self.face_grid.dims();
        let offset = self.size() * 0.5;
        let x = (offset.x + x / TILE_SIZE_X) as u32;
        let y = (offset.y + z / TILE_SIZE_Y) as u32;
        (x < dims.x && y < dims.y).then_some((y * dims.x) + x)
    }
    pub fn face_id_to_center_pos_2d(&self, fid: u32) -> Option<Vec2> {
        let dims = self.face_grid.dims();
        let coord = self.face_grid.id_to_coord(fid);
        (coord.x < dims.x && coord.y < dims.y)
            .then_some((self.to_pos_offset() + coord.as_vec2() + 0.5) * TILE_DIMS)
    }

    pub fn face_id_to_center_pos_3d(&self, fid: u32) -> Option<Vec3> {
        let pos = self.face_id_to_center_pos_2d(fid)?;
        Some(Vec3::new(pos.x, self.face_center_height(fid), pos.y))
    }

    pub fn to_pos_offset(&self) -> Vec2 {
        -(self.size() * 0.5)
    }

    pub fn face_id_to_vert_ids(&self, fid: u32) -> [u32; 4] {
        let dims = self.face_grid.dims();
        let stride = self.vert_grid.dims().x;
        let x = fid % dims.x;
        let y = fid / dims.x;
        [
            stride * (y + 0) + (x + 0),
            stride * (y + 0) + (x + 1),
            stride * (y + 1) + (x + 0),
            stride * (y + 1) + (x + 1),
        ]
    }

    /// Returns the lowest elevation across vertices for a face
    pub fn face_base_elevation(&self, fid: u32) -> u32 {
        self.face_id_to_vert_ids(fid)
            .into_iter()
            .map(|vid| self.vert_data[vid as usize].elevation)
            .fold(u32::MAX, |acc, x| acc.min(x))
    }

    pub fn face_center_height(&self, fid: u32) -> f32 {
        (self
            .face_id_to_vert_ids(fid)
            .into_iter()
            .map(|vid| self.vert_data[vid as usize].elevation)
            .fold(0, |acc, x| acc + x) as f32
            / 4.0)
            * SLOPE_HEIGHT
    }

    pub fn vert_neighbor_elevations(&mut self, vid: u32) -> impl Iterator<Item = u32> + '_ {
        self.vert_grid
            .neighbor_coords_8(self.vert_grid.id_to_coord(vid))
            .into_iter()
            .flatten()
            .map(|vcoord| self.vert_data[self.vert_grid.coord_to_id(vcoord) as usize].elevation)
    }

    pub fn face_grid(&self) -> &Grid {
        &self.face_grid
    }

    pub fn vert_grid(&self) -> &Grid {
        &self.vert_grid
    }

    pub fn faces(&self) -> FaceIter {
        FaceIter::new(&self.face_data)
    }

    pub fn resize_anchored(&mut self, dims: UVec2, anchor: Anchor2, elevation: u32) {
        let mut face_data = Vec::with_capacity(dims.element_product() as usize);
        let vert_dims = dims + UVec2::ONE;
        let mut vert_data = Vec::with_capacity(vert_dims.element_product() as usize);

        for fid in self.face_grid.resize_anchored(dims, anchor) {
            let val = match fid {
                Some(fid) => self.face_data[fid as usize].clone(),
                None => FaceData::default(),
            };
            face_data.push(val);
        }

        for vid in self.vert_grid.resize_anchored(vert_dims, anchor) {
            let val = match vid {
                Some(vid) => self.vert_data[vid as usize].clone(),
                None => VertData { elevation },
            };
            vert_data.push(val);
        }
        self.face_data = face_data;
        self.vert_data = vert_data;
    }
}

pub struct VertPosIter<'a> {
    verts: &'a [VertData],
    stride: usize,
    offset: Vec2,
    index: usize,
}
impl<'a> VertPosIter<'a> {
    pub fn new(verts: &'a [VertData], stride: u32, offset: Vec2) -> Self {
        Self {
            verts,
            stride: stride as usize,
            offset,
            index: 0,
        }
    }
}
impl<'a> Iterator for VertPosIter<'a> {
    type Item = Vec3;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.verts.len() {
            return None;
        }
        let x = self.index % self.stride;
        let y = self.index / self.stride;
        let vtx = &self.verts[self.index];
        self.index += 1;
        Some(Vec3::new(
            self.offset.x + x as f32,
            vtx.elevation as f32 * SLOPE_HEIGHT,
            self.offset.y + y as f32,
        ))
    }
}

pub struct Face<'a> {
    pub fid: u32,
    pub data: &'a FaceData,
}

pub struct NeighboringFaces4<'a> {
    pub faces: [Option<Face<'a>>; 4],
}

pub struct NeighboringFaces8<'a> {
    pub faces: [Option<Face<'a>>; 8],
}

pub struct FaceIter<'a> {
    data: &'a [FaceData],
    index: usize,
}

impl<'a> FaceIter<'a> {
    pub fn new(data: &'a [FaceData]) -> Self {
        Self { data, index: 0 }
    }
}
impl<'a> Iterator for FaceIter<'a> {
    type Item = &'a FaceData;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }
        let next = &self.data[self.index];
        self.index += 1;
        Some(next)
    }
}
