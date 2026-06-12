//! 元数值 — 世界基础量纲
//!
//! 所有游戏中的数值都必须从此文件的常量派生。
//! 禁止在任何其他文件中出现无法追溯到此处常量的裸数字。

pub const TICK_SECONDS: f32 = 0.5;
pub const TICKS_PER_SECOND: f32 = 2.0;
pub const SECONDS_PER_MINUTE: u32 = 60;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const HOURS_PER_DAY: u32 = 24;
pub const TICKS_PER_SECOND_U64: u64 = 2;
pub const TICKS_PER_DAY: u64 = TICKS_PER_SECOND_U64 * SECONDS_PER_MINUTE as u64 * MINUTES_PER_HOUR as u64 * HOURS_PER_DAY as u64;

pub const GRID_CELL_SIZE: f32 = 1.0;
pub const BASE_HP: i32 = 1;
pub const BASE_ENERGY: u32 = 1;

pub fn size_to_weight(size: u8) -> u8 {
    match size { 1 => 1, 2 => 3, 3 => 5, 4 => 8, _ => size.saturating_mul(2) }
}

pub fn size_to_speed_mod(size: u8) -> f32 {
    match size { 1 => 1.5, 2 => 1.2, 3 => 1.0, 4 => 0.7, _ => 1.0 }
}

pub const BASE_MOVE_SPEED: f32 = GRID_CELL_SIZE;

pub fn entity_move_speed(size: u8) -> f32 {
    BASE_MOVE_SPEED * size_to_speed_mod(size)
}

pub fn entity_sprint_speed(size: u8, sprint_tier: f32) -> f32 {
    BASE_MOVE_SPEED * size_to_speed_mod(size) / sprint_tier
}

pub fn impact_damage(weight: u8, speed: f32) -> i32 {
    (weight as f32 * speed).ceil() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_per_day_is_derived() {
        assert_eq!(TICKS_PER_DAY, 172800);
    }

    #[test]
    fn smaller_entities_move_faster() {
        assert!(entity_move_speed(1) > entity_move_speed(3));
        assert!(entity_move_speed(3) > entity_move_speed(4));
    }

    #[test]
    fn all_speeds_are_positive() {
        for s in 1..=10u8 { assert!(entity_move_speed(s) > 0.0); }
    }

    #[test]
    fn impact_damage_derives() {
        assert_eq!(impact_damage(5, 1.0), 5);
        assert_eq!(impact_damage(1, 0.5), 1);
        assert_eq!(impact_damage(8, 0.7), 6);
    }
}
