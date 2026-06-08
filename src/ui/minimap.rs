//! egui minimap — Godot `minimap_panel.gd` equivalent.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::grid_render::SimWorld;
use crate::terrain_colors::cell_color_with_stress;
use crate::sim_clock::SimClock;
use crate::ui_interaction::SelectionState;
use crate::visual_config::{CELL_SIZE, PANEL_WIDTH};
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::viewport_layout::ViewportLayout;

const MINIMAP_H: f32 = 140.0;

pub fn minimap_panel_system(
    mut contexts: EguiContexts,
    sim: Res<SimWorld>,
    clock: Res<SimClock>,
    selection: Res<SelectionState>,
    layout: Res<ViewportLayout>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    let panel_w = crate::visual_config::panel_width_for(layout.window_width);
    let cell_px = (panel_w - 24.0) / GRID_WIDTH as f32;
    let map_h = cell_px * GRID_HEIGHT as f32;

    egui::Area::new("minimap".into())
        .fixed_pos(egui::pos2(
            layout.window_width - panel_w + 8.0,
            layout.window_height - MINIMAP_H - map_h - 8.0,
        ))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("小地图").size(11.0).weak());
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(panel_w - 16.0, map_h.min(MINIMAP_H * 2.0)),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(rect);
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let (r, g, b, _) = cell_color_with_stress(
                        &sim.0,
                        x,
                        y,
                        clock.river_stress,
                    );
                    let c = egui::Color32::from_rgb(r, g, b);
                    let px = rect.min.x + x as f32 * cell_px;
                    let py = rect.min.y + y as f32 * cell_px;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(px, py), egui::vec2(cell_px, cell_px)),
                        0.0,
                        c,
                    );
                }
            }
            if let Some(t) = &selection.target {
                let sx = rect.min.x + t.cell_x as f32 * cell_px;
                let sy = rect.min.y + t.cell_y as f32 * cell_px;
                painter.rect_stroke(
                    egui::Rect::from_min_size(egui::pos2(sx, sy), egui::vec2(cell_px, cell_px)),
                    0.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(22, 129, 58)),
                );
            }
        });

    let _ = CELL_SIZE;
    let _ = PANEL_WIDTH;
}
