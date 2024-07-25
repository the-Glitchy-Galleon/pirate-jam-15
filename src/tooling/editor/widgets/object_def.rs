use crate::{
    game::objects::definitions::{ColorDef, ObjectDefKind},
    tooling::editor::{
        object_def_builder::{ObjectDefBuilder, Rot8},
        tilemap::Tilemap,
    },
};
use bevy::math::UVec2;
use bevy_egui::egui::{self, Color32, ScrollArea, Sense, Stroke, TextureId, Ui};

pub struct ObjectDefWidget {
    selected_id: Option<usize>,
    forced_dirty: bool,
}

impl ObjectDefWidget {
    pub fn new() -> Self {
        Self {
            selected_id: None,
            forced_dirty: false,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        defs: &mut Vec<ObjectDefBuilder>,
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
                    let def = defs[id].clone();

                    let new_def = self.show_def_widget(def, defs, tilemap, textures, ui);
                    if defs[id] != new_def {
                        has_changes = true;
                    }
                    defs[id] = new_def;
                }

                ui.separator();

                let mut delete = None;
                for (i, def) in defs.iter().enumerate() {
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
                    defs.remove(delete);
                    if Some(delete) == self.selected_id {
                        self.selected_id = None;
                    }
                }

                if ui.button("Add New").clicked() {
                    self.selected_id = Some(defs.len());
                    defs.push(ObjectDefBuilder {
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

    pub fn on_coord_select(&mut self, defs: &[ObjectDefBuilder], coord: UVec2) {
        if let Some((i, _)) = defs.iter().enumerate().find(|(_, d)| d.coord == coord) {
            self.selected_id = Some(i);
            self.forced_dirty = true;
        }
    }

    #[must_use]
    fn show_def_widget(
        &self,
        mut def: ObjectDefBuilder,
        defs: &[ObjectDefBuilder],
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
                        ui.add(egui::DragValue::new(obj).range(0..=defs.len() - 1));
                        if ui.button("X").clicked() {
                            delete = Some(i);
                        }
                        if let Some(def) = defs.get(*obj as usize) {
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

    pub fn selected_id(&self) -> Option<usize> {
        self.selected_id
    }
}
