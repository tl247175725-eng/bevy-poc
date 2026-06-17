use bevy::prelude::*;

use crate::card_def::CardDef;

#[derive(Clone, Copy)]
pub struct CardStyle {
    pub bg: Color,
    pub border: Color,
    pub text: Color,
}

#[allow(dead_code)]
fn color_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::WHITE;
    }
    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);
    let r = parse(&hex[0..2]);
    let g = parse(&hex[2..4]);
    let b = parse(&hex[4..6]);
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

#[allow(dead_code)]
fn hex(bg: &str, border: &str) -> CardStyle {
    CardStyle {
        bg: color_hex(bg),
        border: color_hex(border),
        text: color_hex("2c2117"),
    }
}

#[allow(dead_code)]
fn hex_light(bg: &str, border: &str) -> CardStyle {
    CardStyle {
        bg: color_hex(bg),
        border: color_hex(border),
        text: color_hex("fffdf5"),
    }
}

/// Godot `card_base.gd _card_style()` — full table from visual spec §二.
pub fn card_style(_type_name: &str, def: &CardDef) -> CardStyle {
    let (r, g, b, a) = def.color_f32();
    let bg = if a > 0.0 {
        Color::srgba(r, g, b, a)
    } else {
        Color::srgb(1.0, 0.99, 0.96) // 默认浅米色
    };

    // Border = 背景变暗 60%
    let border = Color::srgb(
        (r * 0.6).max(0.0),
        (g * 0.6).max(0.0),
        (b * 0.6).max(0.0),
    );

    // 文字色：亮背景用深色，暗背景用浅色
    let brightness = 0.299 * r + 0.587 * g + 0.114 * b;
    let text = if brightness > 0.5 {
        Color::srgb(0.17, 0.13, 0.09) // 深色文字
    } else {
        Color::srgb(1.0, 0.99, 0.96) // 浅色文字
    };

    CardStyle { bg, border, text }
}
