use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod audio;
pub mod easing;
pub mod global_ui_state;
pub mod grid;
pub mod level_asset;
pub mod logical_cursor;
pub mod raw_mesh;
pub mod tilemap;
pub mod tileset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
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
