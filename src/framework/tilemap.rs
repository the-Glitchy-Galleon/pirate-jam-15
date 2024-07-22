use super::grid::Grid;
use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
use serde::{Deserialize, Serialize};

pub const TILE_SIZE_X: f32 = 1.0;
pub const TILE_SIZE_Y: f32 = 1.0;
pub const TILE_DIMS: Vec2 = Vec2::new(TILE_SIZE_X, TILE_SIZE_Y);
pub const SLOPE_HEIGHT: f32 = 0.5;
pub const WALL_HEIGHT: f32 = 1.0;
pub const MAX_NUM_WALLS: usize = 6;

#[derive(Debug, Reflect, Default, Clone)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
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
#[derive(Default, Reflect, Debug, Clone)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct VertData {
    pub elevation: u32,
}

impl VertData {
    pub const DEFAULT: VertData = VertData { elevation: 0 };
}

#[derive(Asset, Reflect, Debug, Clone)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct Tilemap {
    face_grid: Grid,
    vert_grid: Grid,
    face_data: Vec<FaceData>,
    vert_data: Vec<VertData>,

    #[cfg_attr(not(target_family = "wasm"), serde(default))]
    random_value: bool,
}

impl Tilemap {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        Some(Self {
            face_grid: Grid::new(width, height)?,
            vert_grid: Grid::new(width + 1, height + 1)?,
            face_data: vec![FaceData::DEFAULT; (width * height) as usize],
            vert_data: vec![VertData::DEFAULT; ((width + 1) * (height + 1)) as usize],
            random_value: false,
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

    pub fn vert_iter(&self) -> VertIter {
        VertIter::new(
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
    pub fn face_id_to_center_pos(&self, fid: u32) -> Option<Vec2> {
        let dims = self.face_grid.dims();
        let offset = -(self.size() * 0.5);
        let x = fid % dims.x;
        let y = fid / dims.x;
        (x < dims.x && y < dims.y)
            .then_some(Vec2::new(offset.x + x as f32 + 0.5, offset.y + y as f32 + 0.5) * TILE_DIMS)
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
}

pub struct VertIter<'a> {
    verts: &'a [VertData],
    stride: usize,
    offset: Vec2,
    index: usize,
}
impl<'a> VertIter<'a> {
    pub fn new(verts: &'a [VertData], stride: u32, offset: Vec2) -> Self {
        Self {
            verts,
            stride: stride as usize,
            offset,
            index: 0,
        }
    }
}
impl<'a> Iterator for VertIter<'a> {
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

#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[rustfmt::skip]
pub enum Pnormal3 {NOO,ONO,OON,POO,OPO,OOP}

#[rustfmt::skip]
impl Pnormal3 {
    pub fn from_normal(normal: Vec3) -> Option<Self> {
        fn e(a: f32, b: f32) -> bool {
            (a - b).abs() < f32::EPSILON
        }
        Some(match normal {
            normal if e(normal.x, -1.0) && e(normal.y,  0.0) && e(normal.z,  0.0) => Pnormal3::NOO,
            normal if e(normal.x,  0.0) && e(normal.y, -1.0) && e(normal.z,  0.0) => Pnormal3::ONO,
            normal if e(normal.x,  0.0) && e(normal.y,  0.0) && e(normal.z, -1.0) => Pnormal3::OON,
            normal if e(normal.x,  1.0) && e(normal.y,  0.0) && e(normal.z,  0.0) => Pnormal3::POO,
            normal if e(normal.x,  0.0) && e(normal.y,  1.0) && e(normal.z,  0.0) => Pnormal3::OPO,
            normal if e(normal.x,  0.0) && e(normal.y,  0.0) && e(normal.z,  1.0) => Pnormal3::OOP,
            _ => None?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[rustfmt::skip]
pub enum Pnormal2 {NO,ON,PO,OP}

#[rustfmt::skip]
impl Pnormal2 {
    pub fn from_normal(normal: Vec3) -> Option<Self> {
        fn e(a: f32, b: f32) -> bool {
            (a - b).abs() < f32::EPSILON
        }
        Some(match normal {
            normal if e(normal.x, -1.0) && e(normal.y,  0.0) => Pnormal2::NO,
            normal if e(normal.x,  0.0) && e(normal.y, -1.0) => Pnormal2::ON,
            normal if e(normal.x,  1.0) && e(normal.y,  0.0) => Pnormal2::PO,
            normal if e(normal.x,  0.0) && e(normal.y,  1.0) => Pnormal2::OP,
            _ => None?,
        })
    }
}
