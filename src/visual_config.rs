//! Godot `main.tscn` / `visual-spec-from-godot.md` constants.

pub const CELL_SIZE: f32 = 56.0;
pub const CARD_SIZE: f32 = 50.0;
pub const CARD_OFFSET: f32 = 3.0;
pub const STACK_OFFSET_Y: f32 = 4.0;
/// Godot border rect = CARD_SIZE + 4, offset -2 → 1px ring each side.
pub const CARD_BORDER_PAD: f32 = 2.0;
pub const SELECTION_RING_SIZE: f32 = CARD_SIZE + 8.0;
pub const SELECTION_RING_WIDTH: f32 = 3.0;

/// Godot `main.tscn` GameUI fixed width.
pub const PANEL_WIDTH: f32 = 360.0;
pub const PANEL_MIN_WIDTH: f32 = PANEL_WIDTH;
pub const PANEL_BORDER_PX: f32 = 3.0;

pub const START_PLAYER_X: u8 = 24;
pub const START_PLAYER_Y: u8 = 14;

/// WorldArea background `#efe2c8`
pub const WORLD_BG: (f32, f32, f32) = (0.937, 0.886, 0.784);
/// `game_ui.gd PANEL_BG` `#fff7e6`
pub const PANEL_BG: (f32, f32, f32) = (1.0, 0.969, 0.902);
/// Left border `#6b5337`
pub const PANEL_BORDER: (f32, f32, f32) = (0.420, 0.325, 0.216);
/// `game_ui.gd BOX_BG` `#f5ead6`
pub const BOX_BG: (f32, f32, f32) = (0.961, 0.918, 0.839);
/// Card label default `#2c2117`
pub const TEXT_DARK: (f32, f32, f32) = (0.173, 0.129, 0.090);
/// Panel title `#322318`
pub const PANEL_TEXT_DARK: (f32, f32, f32) = (0.196, 0.133, 0.094);
pub const TEXT_MUTED: (f32, f32, f32) = (0.420, 0.325, 0.216);
/// Selection ring `#16813a`
pub const SELECTION_RING: (f32, f32, f32) = (0.086, 0.506, 0.227);

pub fn world_width() -> f32 {
    crate::world_rules::GRID_WIDTH as f32 * CELL_SIZE
}

pub fn world_height() -> f32 {
    crate::world_rules::GRID_HEIGHT as f32 * CELL_SIZE
}

pub fn panel_width_for(window_width: f32) -> f32 {
    (PANEL_WIDTH * (window_width / 1280.0).clamp(0.85, 1.25)).max(PANEL_MIN_WIDTH)
}

pub fn world_area_width_for(window_width: f32) -> f32 {
    (window_width - panel_width_for(window_width)).max(1.0)
}
