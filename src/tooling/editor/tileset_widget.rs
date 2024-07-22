use bevy::math::UVec2;
use bevy_egui::egui::{self, Color32, Rect, ScrollArea, Stroke, TextureId, Ui, Vec2};

pub struct TilesetWidget {
    texture: TextureId,
    texture_dims: UVec2,
    tile_dims: UVec2,
    selected_tile: Option<UVec2>,
    zoom: f32,
}

impl TilesetWidget {
    pub fn new(texture: TextureId, texture_dims: UVec2, tile_dims: UVec2) -> Self {
        Self {
            texture,
            tile_dims,
            texture_dims,
            selected_tile: None,
            zoom: 1.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<UVec2> {
        let mut result = None;
        let tile_count = self.texture_dims / self.tile_dims;

        if ui.input(|i| i.modifiers.ctrl) {
            let delta = ui.input(|i| i.raw_scroll_delta.y);
            self.zoom = f32::clamp(self.zoom + delta * 0.005, 0.5, 5.0);
        }

        let tile_dims_f32 = Vec2::new(self.tile_dims.x as f32, self.tile_dims.y as f32);
        let tile_dims_f32 = tile_dims_f32 * self.zoom;

        ScrollArea::both().show_viewport(ui, |ui, _viewport| {
            let response = ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                self.texture,
                [
                    self.texture_dims.x as f32 * self.zoom,
                    self.texture_dims.y as f32 * self.zoom,
                ],
            )));

            let image_origin = response.rect.min;

            if response.hovered() {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let pos = pos - image_origin.to_vec2();
                    let coord = Vec2::new(pos.x, pos.y) / tile_dims_f32;
                    let coord = UVec2::new(coord.x as u32, coord.y as u32);

                    if ui.input(|i| i.pointer.any_click()) {
                        if coord.x < tile_count.x && coord.y < tile_count.y {
                            if self.selected_tile != Some(coord) {
                                self.selected_tile = Some(coord);
                                result = Some(coord);
                            }
                        }
                    }

                    let coord_f32 = Vec2::new(coord.x as f32, coord.y as f32);
                    let tl = image_origin + coord_f32 * tile_dims_f32;
                    let br = tl + tile_dims_f32;
                    let preview_rect = Rect::from_min_max(tl, br);
                    ui.painter()
                        .rect_stroke(preview_rect, 1.0, Stroke::new(3.0, Color32::BLUE));
                }
            }

            if let Some(coord) = self.selected_tile {
                let coord_f32 = Vec2::new(coord.x as f32, coord.y as f32);
                let tl = image_origin + coord_f32 * tile_dims_f32;
                let br = tl + tile_dims_f32;
                let selection_rect = Rect::from_min_max(tl, br);
                ui.painter()
                    .rect_stroke(selection_rect, 1.0, Stroke::new(3.0, Color32::RED));
            }
        });
        result
    }
}
