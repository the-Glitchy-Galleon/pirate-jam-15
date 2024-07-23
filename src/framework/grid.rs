use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
use serde::{Deserialize, Serialize};

#[derive(Asset, Debug, Clone, Copy, Reflect)]
#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct Grid {
    dims: UVec2,
}

impl Grid {
    pub fn new(dims: UVec2) -> Option<Self> {
        (dims.x > 0 && dims.y > 0).then_some(Self { dims })
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

    pub fn resize(&mut self, dims: UVec2, start: IVec2) -> ResizeIter {
        let iter = ResizeIter::new(self.dims, dims, start);
        self.dims = dims;
        iter
    }
    pub fn resize_anchored(&mut self, dims: UVec2, anchor: Anchor2) -> ResizeIter {
        let iter = ResizeIter::new_anchored(self.dims, dims, anchor);
        self.dims = dims;
        iter
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Anchor2 {
    x: Anchor,
    y: Anchor,
}

#[rustfmt::skip]
impl Anchor2 {
    pub const TOP_LEFT:     Self = Anchor2 { x: Anchor::Start,  y: Anchor::Start  };
    pub const TOP:          Self = Anchor2 { x: Anchor::Center, y: Anchor::Start  };
    pub const TOP_RIGHT:    Self = Anchor2 { x: Anchor::End,    y: Anchor::Start  };
    pub const LEFT:         Self = Anchor2 { x: Anchor::Start,  y: Anchor::Center };
    pub const CENTER:       Self = Anchor2 { x: Anchor::Center, y: Anchor::Center };
    pub const RIGHT:        Self = Anchor2 { x: Anchor::End,    y: Anchor::Center };
    pub const BOTTOM_LEFT:  Self = Anchor2 { x: Anchor::Start,  y: Anchor::End    };
    pub const BOTTOM:       Self = Anchor2 { x: Anchor::Center, y: Anchor::End    };
    pub const BOTTOM_RIGHT: Self = Anchor2 { x: Anchor::End,    y: Anchor::End    };
}

impl Anchor2 {
    pub fn new(x: Anchor, y: Anchor) -> Self {
        Self { x, y }
    }
    pub fn map(self, a: UVec2, b: UVec2) -> IVec2 {
        IVec2::new(self.x.map(a.x, b.x), self.y.map(a.y, b.y))
    }
    #[rustfmt::skip]
    pub fn description_str(&self) -> &str {
        match (self.x, self.y) {
            (Anchor::Start,  Anchor::Start)  => "Top Left",
            (Anchor::Start,  Anchor::Center) => "Top",
            (Anchor::Start,  Anchor::End)    => "Top Right",
            (Anchor::Center, Anchor::Start)  => "Left",
            (Anchor::Center, Anchor::Center) => "Center",
            (Anchor::Center, Anchor::End)    => "Right",
            (Anchor::End,    Anchor::Start)  => "Bottom Left",
            (Anchor::End,    Anchor::Center) => "Bottom",
            (Anchor::End,    Anchor::End)    => "Bottom Right",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Center,
    End,
}
impl Anchor {
    fn map(self, a: u32, b: u32) -> i32 {
        match self {
            Anchor::Start => 0,
            Anchor::Center => (a as i32 - b as i32) / 2,
            Anchor::End => a as i32 - b as i32,
        }
    }
}
pub struct ResizeIter {
    src_dims: UVec2,
    dst_dims: UVec2,
    start: IVec2,
    i: u32,
}
impl ResizeIter {
    pub fn new(src: UVec2, dst: UVec2, start: IVec2) -> Self {
        Self {
            src_dims: src,
            dst_dims: dst,
            start,
            i: 0,
        }
    }
    pub fn new_anchored(src: UVec2, dst: UVec2, anchor: Anchor2) -> Self {
        Self::new(src, dst, anchor.map(src, dst))
    }
}

impl Iterator for ResizeIter {
    type Item = Option<u32>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.dst_dims.element_product() {
            return None;
        }
        let dst = UVec2::new(self.i % self.dst_dims.x, self.i / self.dst_dims.x);
        let src = IVec2::new(dst.x as i32 + self.start.x, dst.y as i32 + self.start.y);

        let item = if src.x >= 0 && src.y >= 0 {
            let src = UVec2::new(src.x as u32, src.y as u32);
            if src.x < self.src_dims.x && src.y < self.src_dims.y {
                Some(Some(src.y * self.src_dims.x + src.x))
            } else {
                Some(None)
            }
        } else {
            Some(None)
        };
        self.i += 1;
        item
    }
}

#[test]
fn test_resize() {
    let mut grid = Grid::new(UVec2::new(3, 3)).unwrap();
    let mut map = grid.resize_anchored(UVec2::new(2, 2), Anchor2::TOP_LEFT);
    assert_eq!(map.next(), Some(Some(0)));
    assert_eq!(map.next(), Some(Some(1)));
    assert_eq!(map.next(), Some(Some(3)));
    assert_eq!(map.next(), Some(Some(4)));
    assert_eq!(map.next(), None);

    let mut map = grid.resize_anchored(UVec2::new(3, 3), Anchor2::BOTTOM_RIGHT);
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(Some(0)));
    assert_eq!(map.next(), Some(Some(1)));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(Some(2)));
    assert_eq!(map.next(), Some(Some(3)));
    assert_eq!(map.next(), None);

    let mut map = grid.resize_anchored(UVec2::new(5, 5), Anchor2::CENTER);
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));

    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(Some(0)));
    assert_eq!(map.next(), Some(Some(1)));
    assert_eq!(map.next(), Some(Some(2)));
    assert_eq!(map.next(), Some(None));

    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(Some(3)));
    assert_eq!(map.next(), Some(Some(4)));
    assert_eq!(map.next(), Some(Some(5)));
    assert_eq!(map.next(), Some(None));

    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(Some(6)));
    assert_eq!(map.next(), Some(Some(7)));
    assert_eq!(map.next(), Some(Some(8)));
    assert_eq!(map.next(), Some(None));

    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), Some(None));
    assert_eq!(map.next(), None);
}
