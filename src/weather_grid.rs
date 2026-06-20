//! 天气网格 —— 64×64 WeatherCell 管理 + 完整 tick
//!
//! 设计: 不直接依赖 terrain_ecology/WorldState
//! 由调用方（main_tick 或 step4）自己提供水面/海拔/抬升数据
//! 未来接入仿真层时通过闭包或 trait 注入

use crate::weather::{self, WeatherCell};

pub struct WeatherGrid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<WeatherCell>,
}

impl WeatherGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, cells: vec![WeatherCell::default(); (width*height) as usize] }
    }
    pub fn get(&self, x: u32, y: u32) -> &WeatherCell {
        &self.cells[(y*self.width+x) as usize]
    }
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut WeatherCell {
        &mut self.cells[(y*self.width+x) as usize]
    }
}

/// 推进所有格子的天气。调用方提供地形查询闭包。
pub fn tick_all_weather(
    grid: &mut WeatherGrid,
    sun_elevation: f32,
    is_water: impl Fn(u32, u32) -> bool,
    elevation_m: impl Fn(u32, u32) -> f32,
    lift: impl Fn(u32, u32) -> f32,
) {
    let w = grid.width; let h = grid.height;
    let old_temps: Vec<f32> = grid.cells.iter().map(|c| c.temperature).collect();
    let old_vapors: Vec<f32> = grid.cells.iter().map(|c| c.vapor_pressure).collect();

    for y in 0..h { for x in 0..w {
        let idx = (y*w+x) as usize;
        let elev = elevation_m(x, y);
        let baseline = weather::baseline_temperature(0.3, sun_elevation, elev);
        let water = is_water(x, y);
        let water_temp = baseline;

        let mut nt=vec![]; let mut nv=vec![];
        for (dx,dy) in &[(1i32,0),(-1,0),(0,1),(0,-1)] {
            let nx=x as i32+dx; let ny=y as i32+dy;
            if nx>=0 && nx<w as i32 && ny>=0 && ny<h as i32 {
                let ni=(ny as u32*w+nx as u32) as usize;
                nt.push((*dx as f32,*dy as f32,old_temps[ni]));
                nv.push((*dx as f32,*dy as f32,old_vapors[ni]));
            }
        }
        weather::tick_weather_cell(&mut grid.cells[idx], baseline, &nt, &nv, water, water_temp, lift(x,y), 0.,0.);
    }}
}

#[derive(Debug, Clone, Default)]
pub struct WeatherSummary {
    pub avg_temp: f32,
    pub avg_cloud: f32,
    pub avg_precip: f32,
    pub max_wind: f32,
}

pub fn grid_summary(grid: &WeatherGrid) -> WeatherSummary {
    let mut s = WeatherSummary::default();
    let mut total_temp = 0f32; let mut total_cloud = 0f32; let mut total_precip = 0f32;
    for c in &grid.cells {
        total_temp += c.temperature; total_cloud += c.cloud_cover; total_precip += c.precipitation;
        s.max_wind = s.max_wind.max(c.wind_speed);
    }
    let n = grid.cells.len() as f32;
    s.avg_temp = total_temp / n; s.avg_cloud = total_cloud / n; s.avg_precip = total_precip / n;
    s
}
