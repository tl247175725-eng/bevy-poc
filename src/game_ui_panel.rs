//! Godot `game_ui.gd` right panel — egui `SidePanel`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::assets_util::{assets_dir, font_available, ui_font_asset_path};
use crate::grid_render::SimWorld;
use crate::selection_info::{build_panel_with_stress, PanelContent};
use crate::session_report::TickStats;
use crate::sim_clock::SimClock;
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::ui_interaction::{select_containment_entry, SelectionState};
use crate::terrain_colors::river_stress_label;
use crate::visual_config::panel_width_for;
use crate::viewport_layout::ViewportLayout;
const DEFERRED_CAMP_MSG: &str = "经营层未实现：营地/篝火/堆叠";
const DEFERRED_COMMERCE_MSG: &str = "经营层未实现：贸易/菇棚/铜器";

const TITLE: &str = "方寸商国 · MVP v2.24";
const GOAL: &str = "本轮目标：夜归机制、营地堆叠上限倒塌、逃跑受击2次反击、营地食物整理。";
const RULE_DRAG: &str = "左键：物理拖拽，碰撞砸击；松手后停在拖到的位置。";
const RULE_GHOST: &str = "右键：幽灵叠放 / 关系变化。";
const SPEED_OPTIONS: [f32; 5] = [1.0, 1.5, 2.0, 2.5, 3.0];

const BOX_BG: egui::Color32 = egui::Color32::from_rgb(245, 234, 214);
const BOX_BORDER: egui::Color32 = egui::Color32::from_rgb(212, 196, 168);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(107, 83, 55);
const SPEED_ACTIVE: egui::Color32 = egui::Color32::from_rgb(107, 199, 92);

#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

/// Spawn a world-space label (`Text2d` + font/color + transform).
pub fn spawn_text2d(
    content: impl Into<String>,
    font: &UiFont,
    font_size: f32,
    color: Color,
    transform: Transform,
) -> (Text2d, TextFont, TextColor, Transform) {
    (
        Text2d::new(content),
        TextFont {
            font: font.0.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
        transform,
    )
}

pub fn setup_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    crate::assets_util::ensure_ui_font();
    let font = if font_available() {
        asset_server.load(ui_font_asset_path())
    } else {
        Handle::default()
    };
    commands.insert_resource(UiFont(font));
}

pub fn setup_egui_fonts(mut contexts: EguiContexts) {
    if !font_available() {
        return;
    }
    let path = assets_dir().join(ui_font_asset_path());
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "msyh".to_owned(),
        egui::FontData {
            font: std::borrow::Cow::Owned(bytes),
            index: 0,
            tweak: Default::default(),
        },
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "msyh".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("msyh".to_owned());
    ctx.set_fonts(fonts);
}

fn speed_label(mult: f32) -> String {
    if (mult.fract()).abs() < f32::EPSILON {
        format!("{}×", mult as i32)
    } else {
        format!("{mult}×")
    }
}

fn rule_box(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(BOX_BG)
        .stroke(egui::Stroke::new(1.0, BOX_BORDER))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(12.0));
        });
}

pub fn game_ui_panel_system(
    mut contexts: EguiContexts,
    mut clock: ResMut<SimClock>,
    mut selection: ResMut<SelectionState>,
    sim: Res<SimWorld>,
    layout: Res<ViewportLayout>,
    stats: Res<TickStats>,
    events: Res<SimEventQueue>,
) {
    let mut containment_click: Option<EntityId> = None;
    let panel_w = panel_width_for(layout.window_width);

    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    egui::SidePanel::right("game_panel")
        .resizable(false)
        .exact_width(panel_w)
        .show(ctx, |ui| {
            ui.heading(TITLE);
            ui.add_space(4.0);

            rule_box(ui, GOAL);
            rule_box(ui, RULE_DRAG);
            rule_box(ui, RULE_GHOST);

            ui.horizontal_wrapped(|ui| {
                for speed in SPEED_OPTIONS {
                    let active = !clock.paused && (clock.speed - speed).abs() < f32::EPSILON;
                    let label = egui::RichText::new(speed_label(speed));
                    let label = if active {
                        label.color(egui::Color32::WHITE).strong()
                    } else {
                        label
                    };
                    if ui
                        .add(
                            egui::Button::new(label)
                                .fill(if active {
                                    SPEED_ACTIVE
                                } else {
                                    BOX_BG
                                })
                                .stroke(egui::Stroke::new(1.0, BOX_BORDER)),
                        )
                        .clicked()
                    {
                        clock.set_speed(speed);
                    }
                }
                let pause_active = clock.paused;
                let pause_label = egui::RichText::new("暂停");
                let pause_label = if pause_active {
                    pause_label.color(egui::Color32::WHITE).strong()
                } else {
                    pause_label
                };
                let pause_clicked = ui
                    .add(
                        egui::Button::new(pause_label)
                            .fill(if pause_active {
                                SPEED_ACTIVE
                            } else {
                                BOX_BG
                            })
                            .stroke(egui::Stroke::new(1.0, BOX_BORDER)),
                    )
                    .clicked();
                if pause_clicked {
                    clock.set_paused(!pause_active);
                }
            });

            ui.collapsing("时间与天气", |ui| {
                ui.label(format!("游戏时间：{}", clock.game_time_hhmm()));
                ui.label("时间比例：现实1秒 = 游戏1分钟");
                ui.label(format!("天气：{}", clock.weather));
                ui.label(clock.rain_info());
                ui.add(
                    egui::ProgressBar::new((clock.river_stress / 100.0).clamp(0.0, 1.0)).text(
                        format!(
                            "河岸压力：{}/100（{}）",
                            clock.river_stress.round() as i32,
                            river_stress_label(clock.river_stress)
                        ),
                    ),
                );
            });

            ui.horizontal(|ui| {
                ui.add_enabled(false, egui::Button::new("贸易"));
                ui.add_enabled(false, egui::Button::new("招募"));
                ui.label(
                    egui::RichText::new(format!(
                        "{} · {}",
                        DEFERRED_CAMP_MSG,
                        DEFERRED_COMMERCE_MSG
                    ))
                    .size(10.0)
                    .weak(),
                );
            });

            ui.collapsing("当前选中", |ui| {
                let content = selection
                    .target
                    .as_ref()
                    .map(|t| build_panel_with_stress(&sim.0, t, clock.river_stress))
                    .unwrap_or_else(|| PanelContent {
                        title: "点击卡牌或空地。".into(),
                        ..default()
                    });

                ui.label(egui::RichText::new(&content.title).size(15.0));
                if !content.lines.is_empty() {
                    ui.label(
                        egui::RichText::new(content.lines.join("\n"))
                            .size(13.0)
                            .color(TEXT_MUTED),
                    );
                }
                if !content.containment.is_empty() {
                    ui.label(egui::RichText::new("容纳").size(11.0).color(TEXT_MUTED));
                    for entry in &content.containment {
                        if ui
                            .link(format!("【{}】", entry.display_name))
                            .clicked()
                        {
                            containment_click = Some(entry.entity_id);
                        }
                    }
                }
            });

            ui.collapsing("运行状态", |ui| {
                let active_events = sim.0.pending_events.len() + events.pending_len();
                ui.label(format!("实体数：{}", sim.0.entities.len()));
                ui.label(format!("tick 耗时：{:.2} ms", stats.last_tick_ms));
                ui.label(format!("活跃事件：{active_events}"));
                ui.label(format!(
                    "死亡/繁殖/迁徙：{}/{}/{}",
                    stats.deaths, stats.reproduces, stats.migrates
                ));
            });

            ui.collapsing("日志", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in clock.log_lines.iter() {
                            ui.label(egui::RichText::new(line).size(12.0));
                        }
                    });
            });
        });

    if let Some(entity_id) = containment_click {
        select_containment_entry(&sim.0, entity_id, &mut selection);
    }
}

pub fn panel_content_for_test(
    world: &crate::world_state::WorldState,
    selection: &SelectionState,
) -> PanelContent {
    selection
        .target
        .as_ref()
        .map(|t| build_panel_with_stress(world, t, 0.0))
        .unwrap_or_default()
}
