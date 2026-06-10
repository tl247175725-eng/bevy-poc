//! Godot `world_rules_terrain.gd` — fixed map ecology (elevation, pools, wetlands).
//!
//! Not procedural sandbox: `_build_map_ecology()` is deterministic from grid constants.

use std::collections::{HashMap, HashSet};

use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};

pub const ELEV_SURFACE_TOP: i32 = 4;
pub const ELEV_DEN: i32 = -1;
pub const ELEV_FORD: i32 = -1;
pub const ELEV_RIVER: i32 = -4;
pub const ELEV_DARK_RIVER: i32 = -300;

const POOL_DEPTH_RING_MAX: i32 = 5;
const WETLAND_OUTER_RINGS: i32 = 6;

#[derive(Debug, Clone, Default)]
pub struct MapEcology {
    pub ready: bool,
    pub pool_source: (u8, u8),
    pub elevation: Vec<Vec<i32>>,
    pub pool_cells: HashSet<(u8, u8)>,
    pub wetland_cells: HashSet<(u8, u8)>,
    pub water_flow_cells: HashSet<(u8, u8)>,
    pub riparian_cells: HashSet<(u8, u8)>,
    pub underground_river: HashMap<(u8, u8), String>,
}

impl MapEcology {
    pub fn ensure(&mut self) {
        if self.ready {
            return;
        }
        self.build_map_ecology();
        self.ready = true;
    }

    pub fn elevation_at(&self, x: u8, y: u8) -> i32 {
        self.elevation
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .copied()
            .unwrap_or(0)
    }

    pub fn is_pool_cell(&self, x: u8, y: u8) -> bool {
        self.pool_cells.contains(&(x, y))
    }

    pub fn is_wetland_cell(&self, x: u8, y: u8) -> bool {
        self.wetland_cells.contains(&(x, y))
    }

    pub fn is_riparian_grass_cell(&self, x: u8, y: u8) -> bool {
        self.riparian_cells.contains(&(x, y))
    }

    pub fn pool_manhattan_dist(&self, x: u8, y: u8) -> i32 {
        if self.pool_source.0 == 255 {
            return 999;
        }
        let (sx, sy) = self.pool_source;
        (x as i32 - sx as i32).abs() + (y as i32 - sy as i32).abs()
    }

    pub fn underground_river_role(&self, x: u8, y: u8) -> &str {
        self.underground_river
            .get(&(x, y))
            .map(String::as_str)
            .unwrap_or("")
    }

    /// Godot `base_cell_type`.
    pub fn base_cell_type(&self, x: u8, y: u8) -> &'static str {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return "";
        }
        if x == 0 || y == 0 || x == GRID_WIDTH - 1 || y == GRID_HEIGHT - 1 {
            return "barren";
        }
        if self.is_pool_cell(x, y) {
            return "pool";
        }
        if self.is_wetland_cell(x, y) {
            return "wetland";
        }
        "land"
    }

    pub fn is_cell_walkable(&self, x: u8, y: u8) -> bool {
        let t = self.base_cell_type(x, y);
        !t.is_empty() && !matches!(t, "barren" | "pool")
    }

    fn build_map_ecology(&mut self) {
        self.elevation.clear();
        self.water_flow_cells.clear();
        self.riparian_cells.clear();
        self.underground_river.clear();
        self.pool_cells.clear();
        self.wetland_cells.clear();

        let max_y = GRID_HEIGHT as i32 - 2;
        self.pool_source = (GRID_WIDTH / 2, (max_y - 1) as u8);

        for y in 0..GRID_HEIGHT {
            let mut row = Vec::with_capacity(GRID_WIDTH as usize);
            for _x in 0..GRID_WIDTH {
                let elev = if y > 0 && (y as i32) < max_y {
                    let num = max_y - y as i32;
                    let den = (max_y - 1).max(1);
                    ((num as f32 / den as f32) * ELEV_SURFACE_TOP as f32).round() as i32
                } else {
                    0
                };
                row.push(elev);
            }
            self.elevation.push(row);
        }

        let max_dist = POOL_DEPTH_RING_MAX + WETLAND_OUTER_RINGS;
        let (sx, sy) = self.pool_source;

        for y in 1..GRID_HEIGHT - 1 {
            for x in 1..GRID_WIDTH - 1 {
                let dist = (x as i32 - sx as i32).abs() + (y as i32 - sy as i32).abs();
                if dist > max_dist {
                    continue;
                }
                if dist <= POOL_DEPTH_RING_MAX {
                    let cell = (x, y);
                    self.pool_cells.insert(cell);
                    self.elevation[y as usize][x as usize] = pool_elevation_for_dist(dist);
                    self.water_flow_cells.insert(cell);
                    if dist == 0 {
                        self.underground_river
                            .insert(cell, "dark_river_pool".to_string());
                    }
                } else {
                    let cell = (x, y);
                    self.wetland_cells.insert(cell);
                    self.riparian_cells.insert(cell);
                    if self.elevation[y as usize][x as usize] > 1 {
                        self.elevation[y as usize][x as usize] = 1;
                    }
                }
            }
        }
    }
}

fn pool_elevation_for_dist(dist: i32) -> i32 {
    if dist <= 0 {
        ELEV_DARK_RIVER
    } else if dist <= POOL_DEPTH_RING_MAX {
        -6 + dist
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_source_bottom_center() {
        let mut eco = MapEcology::default();
        eco.ensure();
        assert_eq!(eco.pool_source, (18, 21));
    }

    #[test]
    fn pool_cells_at_least_ten() {
        let mut eco = MapEcology::default();
        eco.ensure();
        assert!(eco.pool_cells.len() >= 10);
    }

    #[test]
    fn elevation_higher_in_north() {
        let mut eco = MapEcology::default();
        eco.ensure();
        assert!(eco.elevation_at(10, 2) > eco.elevation_at(10, 20));
    }

    #[test]
    fn dark_river_source_elevation() {
        let mut eco = MapEcology::default();
        eco.ensure();
        let (sx, sy) = eco.pool_source;
        assert_eq!(eco.elevation_at(sx, sy), ELEV_DARK_RIVER);
    }

    #[test]
    fn outer_pool_ring_elevation_minus_one() {
        let mut eco = MapEcology::default();
        eco.ensure();
        let outer: Vec<_> = eco
            .pool_cells
            .iter()
            .copied()
            .filter(|&(x, y)| eco.pool_manhattan_dist(x, y) == POOL_DEPTH_RING_MAX)
            .collect();
        assert!(!outer.is_empty(), "expected outer pool ring cells");
        for (x, y) in outer {
            assert_eq!(eco.elevation_at(x, y), -1);
        }
    }

    #[test]
    fn no_surface_river_columns() {
        let mut eco = MapEcology::default();
        eco.ensure();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                assert_ne!(eco.base_cell_type(x, y), "river");
            }
        }
    }
}
