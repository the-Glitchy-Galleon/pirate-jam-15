use crate::game::object_def::{ColorDef, ObjectDef, ObjectDefKind, Tag};
use bevy::{math::UVec2, reflect::Reflect};
use bevy_egui::egui::{self, Color32, ScrollArea, Sense, Stroke, TextureId, Ui};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

use super::tilemap::Tilemap;

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

pub struct ObjectDefWidget {
    selected_id: Option<usize>,
    defs: Vec<ObjectDefBuilder>,
    forced_dirty: bool,
}

impl ObjectDefWidget {
    pub fn new(defs: Vec<ObjectDefBuilder>) -> Self {
        Self {
            selected_id: None,
            defs,
            forced_dirty: false,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        tilemap: &Tilemap,
        textures: &[TextureId; ObjectDefKind::COUNT],
    ) -> bool {
        let mut has_changes = false;
        if self.forced_dirty {
            has_changes = true;
            self.forced_dirty = false;
        }

        ScrollArea::both()
            .id_source("object_def_widget_scroll")
            .show_viewport(ui, |ui, _viewport| {
                ui.heading("Object Editor");

                ui.separator();

                if let Some(id) = self.selected_id {
                    let def = self.defs[id].clone();

                    let new_def = self.show_def_widget(def, tilemap, textures, ui);
                    if self.defs[id] != new_def {
                        has_changes = true;
                    }
                    self.defs[id] = new_def;
                }

                ui.separator();

                let mut delete = None;
                for (i, def) in self.defs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.button("[X]").clicked() {
                            delete = Some(i);
                        }
                        if ui
                            .add(egui::SelectableLabel::new(
                                Some(i) == self.selected_id,
                                format!(
                                    "{:03}: {} at {{{}:{}}}",
                                    i,
                                    def.kind.as_str(),
                                    def.coord.x,
                                    def.coord.y
                                ),
                            ))
                            .clicked()
                        {
                            self.selected_id = Some(i);
                            has_changes = true;
                        }
                    });
                }
                if let Some(delete) = delete {
                    self.defs.remove(delete);
                    if Some(delete) == self.selected_id {
                        self.selected_id = None;
                    }
                }

                if ui.button("Add New").clicked() {
                    self.selected_id = Some(self.defs.len());
                    self.defs.push(ObjectDefBuilder {
                        coord: tilemap.face_grid().dims() / 2,
                        ..Default::default()
                    });
                    has_changes = true;
                }
                ui.separator();
                ui.add_space(5.0);
            });
        has_changes
    }

    pub fn on_coord_select(&mut self, coord: UVec2) {
        if let Some((i, _)) = self.defs.iter().enumerate().find(|(_, d)| d.coord == coord) {
            self.selected_id = Some(i);
            self.forced_dirty = true;
        }
    }

    #[must_use]
    fn show_def_widget(
        &self,
        mut def: ObjectDefBuilder,
        tilemap: &Tilemap,
        textures: &[TextureId; ObjectDefKind::COUNT],
        ui: &mut Ui,
    ) -> ObjectDefBuilder {
        const W: u32 = 4;
        let h = (textures.len() as u32 + W - 1) / W;

        for y in 0..h {
            ui.horizontal(|ui| {
                for x in 0..W {
                    let i = y * W + x;
                    if i >= textures.len() as u32 {
                        ui.add_space(32.0);
                        continue;
                    }

                    let response = ui.add(
                        egui::widgets::Image::new(egui::load::SizedTexture::new(
                            textures[i as usize],
                            [32.0, 32.0],
                        ))
                        .sense(Sense::click()),
                    );

                    if response.clicked() {
                        def.kind = ObjectDefKind::VARIANTS[i as usize];
                    }

                    if def.kind == ObjectDefKind::VARIANTS[i as usize] {
                        ui.painter().rect_stroke(
                            response.rect,
                            1.0,
                            Stroke::new(2.0, Color32::RED),
                        );
                    } else if response.hovered() {
                        ui.painter().rect_stroke(
                            response.rect,
                            1.0,
                            Stroke::new(3.0, Color32::BLUE),
                        );
                    }
                }
            });
        }

        ui.separator();

        ui.heading(def.kind.as_str());
        ui.horizontal(|ui| {
            ui.label("Coord (Position)");
            ui.add(
                egui::DragValue::new(&mut def.coord.x).range(0..=tilemap.face_grid().dims().x - 1),
            );
            ui.add(
                egui::DragValue::new(&mut def.coord.y).range(0..=tilemap.face_grid().dims().y - 1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Rotation");
            egui::ComboBox::from_label("°")
                .selected_text(def.rotation.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut def.rotation, Rot8::D0, Rot8::D0.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D45, Rot8::D45.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D90, Rot8::D90.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D135, Rot8::D135.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D180, Rot8::D180.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D225, Rot8::D225.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D270, Rot8::D270.as_str());
                    ui.selectable_value(&mut def.rotation, Rot8::D315, Rot8::D315.as_str());
                });
        });
        ui.horizontal(|ui| {
            ui.label("Color");
            egui::ComboBox::from_label("")
                .selected_text(def.color.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut def.color, ColorDef::Void, ColorDef::Void.as_str());
                    ui.selectable_value(&mut def.color, ColorDef::Red, ColorDef::Red.as_str());
                    ui.selectable_value(&mut def.color, ColorDef::Green, ColorDef::Green.as_str());
                    ui.selectable_value(&mut def.color, ColorDef::Blue, ColorDef::Blue.as_str());
                    ui.selectable_value(
                        &mut def.color,
                        ColorDef::Yellow,
                        ColorDef::Yellow.as_str(),
                    );
                    ui.selectable_value(
                        &mut def.color,
                        ColorDef::Magenta,
                        ColorDef::Magenta.as_str(),
                    );
                    ui.selectable_value(&mut def.color, ColorDef::Cyan, ColorDef::Cyan.as_str());
                    ui.selectable_value(&mut def.color, ColorDef::White, ColorDef::White.as_str());
                });
        });
        ui.horizontal(|ui| {
            ui.label("Number");
            ui.add(egui::DragValue::new(&mut def.number).range(0..=256));
        });

        ui.horizontal(|ui| {
            ui.label("Obj Refs");
            ui.vertical(|ui| {
                let mut delete = None;
                for (i, obj) in def.obj_refs.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(obj).range(0..=self.defs.len() - 1));
                        if ui.button("X").clicked() {
                            delete = Some(i);
                        }
                        if let Some(def) = self.defs.get(*obj as usize) {
                            ui.label(format!(
                                "({} at {{{}:{}}})",
                                def.kind.as_str(),
                                def.coord.x,
                                def.coord.y
                            ));
                        } else {
                            ui.colored_label(Color32::RED, "Object doesn't exist");
                        }
                    });
                }
                if let Some(delete) = delete {
                    def.obj_refs.remove(delete);
                }
                if ui.button("+").clicked() {
                    def.obj_refs.push(0);
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Coord Refs");
            ui.vertical(|ui| {
                let mut delete = None;
                for (i, coord) in def.coord_refs.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut coord.x)
                                .range(0..=tilemap.face_grid().dims().x - 1),
                        );
                        ui.add(
                            egui::DragValue::new(&mut coord.y)
                                .range(0..=tilemap.face_grid().dims().y - 1),
                        );
                        if ui.button("X").clicked() {
                            delete = Some(i);
                        }
                    });
                }
                if let Some(delete) = delete {
                    def.coord_refs.remove(delete);
                }
                if ui.button("+").clicked() {
                    def.coord_refs.push(UVec2::ZERO);
                }
            })
        });
        def
    }

    pub fn defs(&self) -> &[ObjectDefBuilder] {
        &self.defs
    }

    pub fn selected_id(&self) -> Option<usize> {
        self.selected_id
    }
}
