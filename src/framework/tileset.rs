use crate::framework::grid::Grid;
use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
use serde::{Deserialize, Serialize};

pub const TILESET_PATH_DIFFUSE: &str = "level/tileset_diff.png";
pub const TILESET_PATH_NORMAL: &str = "level/tileset_norm.png";
pub const TILESET_TEXTURE_DIMS: [u32; 2] = [256, 256];
pub const TILESET_TILE_DIMS: [u32; 2] = [128, 128];
pub const TILESET_TILE_NUM: [u32; 2] = [
    ((TILESET_TEXTURE_DIMS[0] as f32 / TILESET_TILE_DIMS[0] as f32) + 0.5) as u32,
    ((TILESET_TEXTURE_DIMS[1] as f32 / TILESET_TILE_DIMS[1] as f32) + 0.5) as u32,
];

#[derive(Resource, Asset, TypePath)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct Tileset {
    grid: Grid,
}

impl Tileset {
    pub fn new(dims: UVec2) -> Option<Self> {
        Some(Self {
            grid: Grid::new(dims)?,
        })
    }

    pub fn id_to_uvs(&self, tid: u32) -> Option<Rect> {
        let coord = self.grid.try_id_to_coord(tid)?;
        let dims = self.grid.dims();
        Some(Rect {
            min: Vec2::new(
                (coord.x + 0) as f32 / dims.x as f32,
                (coord.y + 0) as f32 / dims.y as f32,
            ),
            max: Vec2::new(
                (coord.x + 1) as f32 / dims.x as f32,
                (coord.y + 1) as f32 / dims.y as f32,
            ),
        })
    }

    // in case the tileset shrinks and some use other tiles
    pub fn id_to_uvs_or_default(&self, tid: u32) -> Rect {
        match self.id_to_uvs(tid) {
            Some(rect) => rect,
            None => self.id_to_uvs(0).unwrap_or(Rect::default()),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }
}
