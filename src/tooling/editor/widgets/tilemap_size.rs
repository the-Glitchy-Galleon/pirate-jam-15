use bevy::math::UVec2;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::framework::grid::{Anchor, Anchor2};

pub const MAX_GRID_DIMS: UVec2 = UVec2::new(128, 128);

pub struct AnchorWidget {
    pub selected: Option<Anchor2>,
}
impl AnchorWidget {
    const CELL_SIZE: f32 = 32.0;
    pub fn show(&mut self, ui: &mut Ui) -> Option<Anchor2> {
        let canvas_size = Vec2::new(3.0 * Self::CELL_SIZE, 3.0 * Self::CELL_SIZE);
        let (response, painter) = ui.allocate_painter(canvas_size, Sense::click());
        let rect = response.rect;
        let mut result = None;

        let anchors = [(0, Anchor::Start), (1, Anchor::Center), (2, Anchor::End)];
        for (y, anchor_y) in anchors {
            for (x, anchor_x) in anchors {
                let anchor = Anchor2::new(anchor_x, anchor_y);
                let color = match self.selected {
                    Some(a) if a == anchor => Color32::RED,
                    _ => Color32::LIGHT_GRAY,
                };
                let cell_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + x as f32 * Self::CELL_SIZE,
                        rect.min.y + y as f32 * Self::CELL_SIZE,
                    ),
                    egui::Vec2::new(Self::CELL_SIZE, Self::CELL_SIZE),
                );
                painter.rect_filled(cell_rect, 3.0, color);
                painter.rect_stroke(cell_rect, 3.0, Stroke::new(1.0, Color32::DARK_GRAY));

                if response.clicked()
                    && cell_rect.contains(response.interact_pointer_pos().unwrap_or(Pos2::ZERO))
                {
                    self.selected = Some(anchor);
                    result = self.selected;
                }
            }
        }
        result
    }
}

pub struct TilemapSizeWidget {
    anchor_widget: AnchorWidget,
    pub old_dims: UVec2,
    pub sliders: UVec2,
    pub elevation: u32,
}

impl TilemapSizeWidget {
    pub fn new(sliders: UVec2, elevation: u32) -> Self {
        Self {
            anchor_widget: AnchorWidget {
                selected: Some(Anchor2::CENTER),
            },
            old_dims: sliders,
            sliders,
            elevation,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<(Anchor2, UVec2, u32)> {
        ui.separator();

        let mut result = None;

        ui.horizontal(|ui| {
            self.anchor_widget.show(ui);
            ui.vertical(|ui| {
                ui.label(format!(
                    "Anchor: {}",
                    self.anchor_widget.selected.unwrap().description_str()
                ));
                let mut update = false;
                ui.horizontal(|ui| {
                    ui.label("X");
                    let r = ui.add(egui::Slider::new(&mut self.sliders.x, 1..=MAX_GRID_DIMS.x));
                    if r.drag_stopped() || r.lost_focus() {
                        update = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Y");
                    let r = ui.add(egui::Slider::new(&mut self.sliders.y, 1..=MAX_GRID_DIMS.x));
                    if r.drag_stopped() || r.lost_focus() {
                        update = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Elevation");
                    ui.add(egui::Slider::new(&mut self.elevation, 0..=32));
                });
                if update {
                    if self.old_dims != self.sliders {
                        self.old_dims = self.sliders;
                        result = Some((
                            self.anchor_widget.selected.unwrap(),
                            self.sliders,
                            self.elevation,
                        ));
                    }
                }
            });
        });
        ui.separator();

        result
    }
    pub fn set_dims(&mut self, dims: UVec2) {
        self.old_dims = dims;
        self.sliders = dims;
    }
}
