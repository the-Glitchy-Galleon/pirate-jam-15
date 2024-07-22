use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
use serde::{Deserialize, Serialize};

#[derive(Asset, Reflect, Debug, Clone, Copy)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct Grid {
    dims: UVec2,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self {
            dims: UVec2::new(width, height),
        })
    }
    pub fn id_to_coord(&self, id: u32) -> UVec2 {
        UVec2::new(id % self.dims.x, id / self.dims.x)
    }
    pub fn try_id_to_coord(&self, id: u32) -> Option<UVec2> {
        let coord = UVec2::new(id % self.dims.x, id / self.dims.x);
        self.is_coord_in_grid(coord).then_some(coord)
    }
    pub fn coord_to_id(&self, coord: UVec2) -> u32 {
        coord.y * self.dims.x + coord.x
    }

    pub fn is_coord_in_grid(&self, coord: UVec2) -> bool {
        coord.x < self.dims.x && coord.y < self.dims.y
    }
    pub fn is_id_in_grid(&self, id: u32) -> bool {
        self.is_coord_in_grid(self.id_to_coord(id))
    }
    #[rustfmt::skip]
    pub fn neighbor_coords_4(&self, coord: UVec2) -> [Option<UVec2>; 4] {
        [
            if coord.x > 0             { Some(coord - UVec2::X) } else { None },
            if coord.x < self.dims.x-1 { Some(coord + UVec2::X) } else { None },
            if coord.y > 0             { Some(coord - UVec2::Y) } else { None },
            if coord.y < self.dims.y-1 { Some(coord + UVec2::Y) } else { None },
        ]
    }

    #[rustfmt::skip]
    // Todo: do some u32 wrapping shenanigans instead?
    pub fn neighbor_coords_8(&self, coord: UVec2) -> [Option<UVec2>; 8] {
        [
            if coord.x > 0             { Some(coord - UVec2::X) } else { None },
            if coord.x < self.dims.x-1 { Some(coord + UVec2::X) } else { None },
            if coord.y > 0             { Some(coord - UVec2::Y) } else { None },
            if coord.y < self.dims.y-1 { Some(coord + UVec2::Y) } else { None },
            if coord.x > 0             && coord.y > 0             { Some(coord - UVec2::X - UVec2::Y) } else { None },
            if coord.x > 0             && coord.y < self.dims.y-1 { Some(coord - UVec2::X + UVec2::Y) } else { None },
            if coord.x < self.dims.x-1 && coord.y > 0             { Some(coord + UVec2::X - UVec2::Y) } else { None },
            if coord.x < self.dims.x-1 && coord.y < self.dims.y-1 { Some(coord + UVec2::X + UVec2::Y) } else { None },
        ]
    }
    pub fn dims(&self) -> UVec2 {
        self.dims
    }
}
