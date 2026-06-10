use bevy::prelude::*;

pub const GAME_SECONDS_PER_REAL_SECOND: f32 = 60.0;
pub const RAIN_INTERVAL_GAME_SECS: f32 = 21600.0;
pub const RAIN_DURATION_GAME_SECS: f32 = 3600.0;
pub const RIVER_STRESS_RECOVER: f32 = 0.25;
pub const RIVER_RAIN_RECOVER: f32 = 2.4;

#[derive(Resource)]
pub struct SimClock {
    pub speed: f32,
    pub paused: bool,
    pub game_time_seconds: f64,
    pub weather: String,
    pub weather_timer: f32,
    pub river_stress: f32,
    pub log_lines: Vec<String>,
    tick_accum: f32,
}

impl Default for SimClock {
    fn default() -> Self {
        Self {
            speed: 1.0,
            paused: false,
            game_time_seconds: 0.0,
            weather: "晴天".into(),
            weather_timer: 0.0,
            river_stress: 0.0,
            log_lines: vec![
                format!("世界初始化完成，卡牌数：{}", crate::initial_spawn::initial_card_count()),
                "v2.25 起：蘑菇链 + 蘑菇农。".into(),
            ],
            tick_accum: 0.0,
        }
    }
}

impl SimClock {
    pub fn game_time_hhmm(&self) -> String {
        let day_seconds = (self.game_time_seconds as u64) % 86_400;
        let h = day_seconds / 3600;
        let m = (day_seconds % 3600) / 60;
        format!("{h:02}:{m:02}")
    }

    pub fn format_duration_hm(seconds: f32) -> String {
        let s = seconds.max(0.0) as u64;
        let h = s / 3600;
        let m = (s % 3600) / 60;
        format!("{h}:{m:02}")
    }

    pub fn rain_info(&self) -> String {
        if self.weather == "晴天" {
            let remain = (RAIN_INTERVAL_GAME_SECS - self.weather_timer).max(0.0);
            format!("距下雨：{}", Self::format_duration_hm(remain))
        } else {
            let remain = (RAIN_DURATION_GAME_SECS - self.weather_timer).max(0.0);
            format!("雨还会下：{}", Self::format_duration_hm(remain))
        }
    }

    pub fn world_state_text(&self) -> String {
        format!(
            "游戏时间：{}\n时间比例：现实1秒 = 游戏1分钟\n天气：{}\n{}\n河岸压力：{} / 100",
            self.game_time_hhmm(),
            self.weather,
            self.rain_info(),
            self.river_stress.round() as i32
        )
    }

    pub fn world_state_key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.game_time_hhmm(),
            self.weather,
            self.rain_info(),
            self.river_stress.round() as i32
        )
    }

    pub fn set_speed(&mut self, mult: f32) {
        self.paused = false;
        self.speed = mult.max(0.0);
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if !paused {
            self.speed = 1.0;
        }
    }

    pub fn push_log(&mut self, line: impl Into<String>) {
        self.log_lines.insert(0, line.into());
        if self.log_lines.len() > 80 {
            self.log_lines.truncate(80);
        }
    }

    pub fn tick_time_weather(&mut self, real_delta: f32) {
        if self.paused || self.speed <= 0.0 {
            return;
        }
        let scaled_real = real_delta * self.speed;
        let game_delta = scaled_real * GAME_SECONDS_PER_REAL_SECOND;
        self.game_time_seconds += game_delta as f64;
        self.weather_timer += game_delta;

        let recover_rate = if self.weather == "下雨" {
            RIVER_RAIN_RECOVER
        } else {
            RIVER_STRESS_RECOVER
        };
        if self.river_stress > 0.0 {
            self.river_stress = (self.river_stress - scaled_real * recover_rate).max(0.0);
        }

        if self.weather == "晴天" && self.weather_timer >= RAIN_INTERVAL_GAME_SECS {
            self.weather = "下雨".into();
            self.weather_timer = 0.0;
            self.river_stress = (self.river_stress - 25.0).max(0.0);
            self.push_log("下雨了 → 河岸压力下降，草皮生成，留在地上的浆果尝试发芽");
        } else if self.weather == "下雨" && self.weather_timer >= RAIN_DURATION_GAME_SECS {
            self.weather = "晴天".into();
            self.weather_timer = 0.0;
            self.push_log("雨停了");
        }
    }
}

pub fn advance_sim_ticks(
    time: Res<Time>,
    mut clock: ResMut<SimClock>,
    mut sim: ResMut<crate::grid_render::SimWorld>,
    mut events: ResMut<crate::sim_events::SimEventQueue>,
    mut stats: ResMut<crate::session_report::TickStats>,
    mut sim_stats: ResMut<crate::sim_events::SimStats>,
    playback: Res<crate::sim_events::MoveAnimPlayback>,
    mut move_anim_events: EventWriter<crate::sim_events::MoveAnimEvent>,
) {
    let real_delta = time.delta_secs();
    clock.tick_time_weather(real_delta);

    if clock.paused || clock.speed <= 0.0 {
        return;
    }
    // FIX: animation completion detection broken — blocking disabled
    // if playback.in_progress { return; }
    clock.tick_accum += real_delta * clock.speed;
    while clock.tick_accum >= crate::game_constants::TICK_SECONDS {
        // if playback.in_progress { break; }
        clock.tick_accum -= crate::game_constants::TICK_SECONDS;
        let t0 = std::time::Instant::now();
        let move_anims = sim.0.tick_once();
        let pending = sim.0.drain_pending_events();
        sim_stats.interactions_this_tick =
            crate::sim_events::count_tick_interactions(&pending);
        crate::sim_events::sync_sim_stats(&sim.0, &mut sim_stats);
        stats.record_tick_duration(t0.elapsed().as_secs_f32());
        stats.record_entity_count(sim.0.entities.len());
        for event in pending {
            events.push(event);
        }
        let had_move_anims = !move_anims.is_empty();
        for anim in move_anims {
            move_anim_events.send(anim);
        }
        if had_move_anims {
            break;
        }
    }
}
