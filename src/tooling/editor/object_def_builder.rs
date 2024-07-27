use crate::{
    framework::tilemap::Tilemap,
    game::objects::definitions::{ColorDef, ObjectDef, ObjectDefKind, Tag},
};
use bevy::{math::UVec2, reflect::Reflect};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize, PartialEq, Eq)]
pub enum Rot8 {
    #[default]
    D0,
    D45,
    D90,
    D135,
    D180,
    D225,
    D270,
    D315,
}
impl Rot8 {
    #[rustfmt::skip]
    pub fn as_str(self) -> &'static str {
        match self {
            Rot8::D0   =>  "0",
            Rot8::D45  =>  "45",
            Rot8::D90  =>  "90",
            Rot8::D135 => "135",
            Rot8::D180 => "180",
            Rot8::D225 => "225",
            Rot8::D270 => "270",
            Rot8::D315 => "315",
        }
    }
}
impl Into<f32> for Rot8 {
    fn into(self) -> f32 {
        match self {
            Rot8::D0 => (TAU / 8.) * 0.,
            Rot8::D45 => (TAU / 8.) * 1.,
            Rot8::D90 => (TAU / 8.) * 2.,
            Rot8::D135 => (TAU / 8.) * 3.,
            Rot8::D180 => (TAU / 8.) * 4.,
            Rot8::D225 => (TAU / 8.) * 5.,
            Rot8::D270 => (TAU / 8.) * 6.,
            Rot8::D315 => (TAU / 8.) * 7.,
        }
    }
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq)]
pub struct ObjectDefBuilder {
    pub kind: ObjectDefKind,
    pub coord: UVec2,
    pub rotation: Rot8,
    pub color: ColorDef,
    pub number: u32,
    pub obj_refs: Vec<u32>,
    pub coord_refs: Vec<UVec2>,
    pub tags: Vec<Tag>,
}

impl Default for ObjectDefBuilder {
    fn default() -> Self {
        Self {
            kind: ObjectDefKind::Camera,
            coord: Default::default(),
            rotation: Default::default(),
            color: ColorDef::Void,
            number: 0,
            obj_refs: Vec::new(),
            coord_refs: Default::default(),
            tags: Vec::new(),
        }
    }
}

impl ObjectDefBuilder {
    pub fn build(&self, tilemap: &Tilemap) -> ObjectDef {
        let position = tilemap
            .face_id_to_center_pos_3d(tilemap.face_grid().coord_to_id(self.coord))
            .unwrap_or_default();

        let pos_refs = self
            .coord_refs
            .iter()
            .map(|c| {
                tilemap
                    .face_id_to_center_pos_3d(tilemap.face_grid().coord_to_id(*c))
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        ObjectDef {
            kind: self.kind,
            position,
            rotation: self.rotation.into(),
            color: self.color,
            obj_refs: self.obj_refs.clone(),
            pos_refs,
            tags: self.tags.clone(),
        }
    }
}
