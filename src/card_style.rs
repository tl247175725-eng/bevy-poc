use bevy::prelude::*;

use crate::card_def::CardDef;

#[derive(Clone, Copy)]
pub struct CardStyle {
    pub bg: Color,
    pub border: Color,
    pub text: Color,
}

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

fn hex(bg: &str, border: &str) -> CardStyle {
    CardStyle {
        bg: color_hex(bg),
        border: color_hex(border),
        text: color_hex("2c2117"),
    }
}

fn hex_light(bg: &str, border: &str) -> CardStyle {
    CardStyle {
        bg: color_hex(bg),
        border: color_hex(border),
        text: color_hex("fffdf5"),
    }
}

/// Godot `card_base.gd _card_style()` — full table from visual spec §二.
pub fn card_style(type_name: &str, def: &CardDef) -> CardStyle {
    match type_name {
        "grass" => hex("c9f28b", "5d9d36"),
        "dryGrass" => hex("ead179", "aa7d22"),
        "tree" => hex_light("2f7f43", "18552b"),
        "bush" => hex_light("3f8f55", "1f5f36"),
        "algae" => hex_light("4a9e6a", "2a6e42"),
        "humus" => hex_light("5a4030", "3a2818"),
        "stone" => hex("d1d3d4", "777b80"),
        "shard" | "tri" | "square" => hex("c5c8ca", "6f7478"),
        "wood" => hex("c58b4b", "7b4d25"),
        "twig" => hex("d0a15d", "8b5f2c"),
        "woodStruct" => hex("b47b45", "6c4323"),
        "hut" => hex("d8b76a", "8a6226"),
        "fire" => hex("ffb156", "c53a1e"),
        "mountain" => hex_light("9e9fa1", "555759"),
        "sheep" => hex("f1dfbd", "9b7351"),
        "lamb" => hex("fff1cb", "b9905a"),
        "rabbit" => hex("e9cfae", "ad7e55"),
        "pheasant" => hex("c8a858", "8a6020"),
        "pheasantChick" => hex("e8d0a0", "a88040"),
        "bambooRat" => hex("9a8a70", "5a4a38"),
        "deer" => hex("e8dcc8", "8b6914"),
        "deerFawn" => hex("f5ecd8", "a8844a"),
        "fieldMouse" => hex("c9b8a0", "6a5040"),
        "fieldMousePup" => hex("e0d4c4", "8a7060"),
        "fox" => hex_light("e8a060", "a85820"),
        "foxCub" => hex("f0c090", "c07830"),
        "waterBuffalo" => hex_light("6a5a50", "3a2a20"),
        "waterBuffaloCalf" => hex_light("9a8a7a", "5a4a40"),
        "wolf" => hex_light("7b403d", "5c1f25"),
        "wolfCub" => hex_light("98605a", "6a2f31"),
        "foxDen" => hex("8a6040", "4a3020"),
        "wolfDen" => hex_light("6f5140", "3d2c25"),
        "waterBug" => hex("8ab84a", "5a8828"),
        "fish" => CardStyle {
            bg: color_hex("8ed0e8"),
            border: color_hex("3a88a8"),
            text: color_hex("2c2117"),
        },
        "shellfish" => hex("e8c8a0", "a88860"),
        "sheepMeat" => hex("e68475", "a43d36"),
        "rabbitMeat" => hex("d9856a", "9a3a28"),
        "deerMeat" => hex("d47868", "9a4030"),
        "fishMeat" => hex("c8a0a0", "886868"),
        "berry" => hex_light("d84d78", "8d2645"),
        "mushroom" => hex("f6d6db", "b75b73"),
        "cookmeat" => hex("f2a66d", "a75227"),
        "acorn" => hex("c8a060", "8a6030"),
        "pineCone" => hex("a88858", "685028"),
        "caltropFruit" => hex_light("8ab878", "4a7848"),
        "lotusSeed" => hex("f0d8b0", "c0a070"),
        "wildYamRoot" => hex("d8b878", "a88848"),
        "landBug" => hex("8a8a48", "5a5a28"),
        "sheepCorpse" => hex_light("8f6c67", "563938"),
        "deerCorpse" => hex_light("7a6358", "4a3828"),
        "wolfCorpse" => hex_light("5c3830", "3a1e18"),
        "bucket" => hex("d8b76a", "7b5b2d"),
        "waterbucket" => hex("8fd0f5", "2879a9"),
        "spear" => hex("c8a870", "7a5225"),
        "knife" => hex("e7e9ec", "59616a"),
        "axe" => hex("e0d0ad", "6c4c2c"),
        "hammer" => hex("d5d9dd", "5e6670"),
        "player" => hex("cfe7ff", "3571a8"),
        "traveler" => hex("e7dcff", "7152a6"),
        "taoyuanElder" => hex("e8e0d0", "6a5a4a"),
        "taoyuanForager" => hex("dce8d0", "5a6a48"),
        "taoyuanYouth" => hex("e0e8f0", "4a6080"),
        "oak" => hex_light("6a8a48", "3a5a28"),
        "pine" => hex_light("4a7a58", "2a5038"),
        "waterCaltrop" => hex_light("7ab878", "4a8848"),
        "lotus" => hex_light("f0b8d8", "c878a8"),
        "birdNest" => hex("b89868", "786040"),
        "wildYam" => hex_light("9a7848", "6a5030"),
        _ => {
            let mut style = CardStyle {
                bg: color_hex("fffdf5"),
                border: color_hex("6b5337"),
                text: color_hex("2c2117"),
            };
            let (r, g, b, a) = def.color_f32();
            if a > 0.0 {
                style.bg = Color::srgba(r, g, b, a);
                style.border = Color::srgba(r * 0.55, g * 0.55, b * 0.55, a);
            }
            style
        }
    }
}
