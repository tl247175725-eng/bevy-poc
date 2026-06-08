//! Session summary — written to `session_report.txt` on game exit.

use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::sim_clock::SimClock;
use crate::sim_events::SimEvent;
use crate::world_state::WorldState;

#[derive(Resource, Default)]
pub struct TickStats {
    pub last_tick_ms: f32,
    pub avg_tick_ms: f32,
    pub max_tick_ms: f32,
    pub deaths: u64,
    pub reproduces: u64,
    pub migrates: u64,
    pub peak_entities: usize,
    pub tick_samples: u64,
    tick_total_ms: f64,
}

impl TickStats {
    pub fn record_tick_duration(&mut self, secs: f32) {
        let ms = secs * 1000.0;
        self.last_tick_ms = ms;
        self.tick_samples += 1;
        self.tick_total_ms += ms as f64;
        self.avg_tick_ms = (self.tick_total_ms / self.tick_samples as f64) as f32;
        if ms > self.max_tick_ms {
            self.max_tick_ms = ms;
        }
    }

    pub fn record_entity_count(&mut self, count: usize) {
        if count > self.peak_entities {
            self.peak_entities = count;
        }
    }

    pub fn record_sim_event(&mut self, event: &SimEvent) {
        match event {
            SimEvent::Death { .. } | SimEvent::Kill { .. } => self.deaths += 1,
            SimEvent::Reproduce { .. } => self.reproduces += 1,
            SimEvent::Migrate { .. } => self.migrates += 1,
            _ => {}
        }
    }
}

#[derive(Resource)]
pub struct SessionWallClock {
    pub started: Instant,
}

impl Default for SessionWallClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

pub struct SessionReport {
    pub wall_seconds: f64,
    pub game_day: u64,
    pub game_hhmm: String,
    pub tick_count: u64,
    pub entity_count: usize,
    pub peak_entities: usize,
    pub deaths: u64,
    pub reproduces: u64,
    pub migrates: u64,
    pub avg_tick_ms: f32,
    pub max_tick_ms: f32,
}

impl SessionReport {
    pub fn from_runtime(
        wall: &SessionWallClock,
        clock: &SimClock,
        world: &WorldState,
        stats: &TickStats,
    ) -> Self {
        let game_day = (clock.game_time_seconds as u64) / 86_400 + 1;
        Self {
            wall_seconds: wall.started.elapsed().as_secs_f64(),
            game_day,
            game_hhmm: clock.game_time_hhmm(),
            tick_count: world.tick_count,
            entity_count: world.entities.len(),
            peak_entities: stats.peak_entities.max(world.entities.len()),
            deaths: stats.deaths,
            reproduces: stats.reproduces,
            migrates: stats.migrates,
            avg_tick_ms: stats.avg_tick_ms,
            max_tick_ms: stats.max_tick_ms,
        }
    }

    pub fn format_text(&self) -> String {
        format!(
            "方寸商国 会话报告\n\
             ━━━━━━━━━━━━━━━━━━━━\n\
             运行时间：{:.0} 秒\n\
             游戏时间：第 {} 天 {}\n\
             总 tick 数：{}\n\
             实体数：{}（峰值 {}）\n\
             生态事件：死亡 {} 次 / 繁殖 {} 次 / 迁徙 {} 次\n\
             平均 tick 耗时：{:.2} ms\n\
             最大 tick 耗时：{:.2} ms\n",
            self.wall_seconds,
            self.game_day,
            self.game_hhmm,
            self.tick_count,
            self.entity_count,
            self.peak_entities,
            self.deaths,
            self.reproduces,
            self.migrates,
            self.avg_tick_ms,
            self.max_tick_ms,
        )
    }

    pub fn write_to(&self, path: PathBuf) -> std::io::Result<()> {
        fs::write(path, self.format_text())
    }
}

pub fn session_report_path() -> PathBuf {
    crate::assets_util::manifest_dir().join("session_report.txt")
}

pub fn write_session_report_on_exit(
    exit: EventReader<AppExit>,
    wall: Res<SessionWallClock>,
    clock: Res<SimClock>,
    sim: Res<crate::grid_render::SimWorld>,
    stats: Res<TickStats>,
) {
    if exit.is_empty() {
        return;
    }
    let report = SessionReport::from_runtime(&wall, &clock, &sim.0, &stats);
    if let Err(err) = report.write_to(session_report_path()) {
        bevy::log::warn!("failed to write session_report.txt: {err}");
    }
}
