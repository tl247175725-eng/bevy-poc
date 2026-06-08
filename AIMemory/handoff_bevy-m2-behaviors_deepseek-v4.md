# Bevy 迁移 M2：全部生态行为 + 地形 + 容纳

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0
**基准**: Godot 全部 behavior 文件 + `ecosystem_manager.gd` + `environment_manager.gd` + `population_manager.gd`

---

## M2 目标

把 Godot 全部生态行为逻辑迁移到 Bevy。M1 有了卡定义和标签判定，M2 让它们动起来——吃草、捕猎、繁殖、腐解、水生、容纳溢出。

---

## 一、WorldState 扩展

### 1.1 运行时实体状态

当前 POC 的 `WorldState` 需扩展。每张卡除了 `CardDef` 还有运行时数据：

```rust
pub struct EntityState {
    pub id: EntityId,
    pub type_name: String,       // 对应 CardDef.type_name
    pub x: i32, y: i32,          // 格坐标
    pub hp: i32,
    pub sex: Option<String>,     // "公" / "母"
    pub age: f32,                // 生长进度
    pub fed_today: bool,
    pub starve_days: i32,
    pub in_den: bool,
    pub in_tree: bool,
    pub in_pool: bool,
    pub in_ground: bool,
    pub carrying: Option<EntityId>,
    pub stunned: bool,
    pub hunt_cooldown: f32,
    pub eat_time: f32,
    pub perish_ticks: i32,
    pub produce_timer: f32,
}
```

### 1.2 Tick 常量

参照 Godot `game_state.gd`，在 Rust 中建常量模块：

```rust
pub const REAL_TICK: f32 = 1.0 / 60.0;
pub const LIVING_GRASS_CAP: i32 = 30;
pub const SHEEP_FEAR_RANGE: i32 = 3;
pub const WILDPREY_FEAR_RANGE: i32 = 5;
pub const FIRE_FEAR_RANGE: i32 = 2;
pub const RABBIT_WOLF_FEAR_DIST: i32 = 3;
pub const SHEEP_EAT_SECONDS: f32 = 3.0;
pub const PERISHABLE_TICKS: i32 = 90;
pub const ALGAE_REGEN_SECONDS: f32 = 15.0;
pub const POPULATION_REPRO_CYCLE_SECONDS: f32 = 30.0;
pub const PROLIFIC_REPRO_CYCLE_SECONDS: f32 = 3.0;
pub const PROLIFIC_LITTER_SIZE: i32 = 3;
pub const CLIFF_WEATHER_INTERVAL: f32 = 60.0;
pub const YAM_SPREAD_INTERVAL: f32 = 60.0;
// ... 其余从 Godot 全量迁移
```

---

## 二、食草动物系统

**参照 Godot**: `scripts/world/behaviors/herbivore_grazer_behavior.gd`

### 2.1 行为 Profile 分发

`herbivore_grazer_profile(entity)` 返回枚举：

```rust
enum GrazerProfile { Juvenile, Rabbit, Deer, Sheep, Slow, Pheasant }
```

判定同 Godot `herbivore_grazer_profile`：
- `capability.escape_small` → Rabbit
- `capability.escape_fast` ∧ `wildPrey` → Deer  
- `capability.move_slow` → Slow (waterBuffalo)
- `flocking` ∧ `omnivore.small` → Pheasant
- `can_reproduce` ∧ `can_forage` → Sheep
- 其他 → Juvenile

### 2.2 各 Profile Tick 逻辑

**羊 (Sheep)** — `_tick_sheep`：
- 逃跑中 → 继续逃，无路径 → 等草
- 玩家邻格 → 躲开
- 狼在 `SHEEP_FEAR_RANGE` 内 → 逃离
- 跟随照料者（幼羊）
- 找最近草 → 移动 → 邻格吃 → mark_ecology_fed
- 无草 → 等草

**鹿 (Deer)** — `_tick_deer`：
- 同上，但恐惧距离用 `WILDPREY_FEAR_RANGE`
- 惧火
- 逃狼

**兔 (Rabbit)** — `_tick_rabbit`：
- 玩家邻格 → 逃
- 草内隐藏判定
- 狼在恐惧距 → 躲草/逃
- 找食（草 + 灌丛浆果 + 橡子 + 松塔 + 菱角果 + 莲子 + landBug）

**雉鸡 (Pheasant)** — 走 Rabbit pipeline，差异：不隐藏草丛

**水牛 (WaterBuffalo)** — 缓行，成体不可猎（`tough`），幼体可猎

### 2.3 通用判定（WorldRules 中已有，M2 调用）

- `flee_from_threat(entity, threat)` — 计算最佳逃跑方向
- `nearest_wolf(entity, range)` — SpatialIndex 查询
- `mark_ecology_fed(entity)` — 标记饱腹
- `flocking_blocks_reproduction(adults)` — M1 已有

---

## 三、捕食者系统

**参照 Godot**: `scripts/world/behaviors/predator_den_behavior.gd` + `capability_behavior_pipeline.gd`

### 3.1 狼 (Wolf)

**优先级管线**: flee → fire → den_work → hunt

- Flee: 玩家邻格 → 躲开；惧火 → 远离篝火
- Den work: 
  - 叼肉 → 回窝交付（距离 2 内可交）
  - 无窝且应建窝 → 筑窝
  - 窝内 → 冷却倒数
  - 找肉源 → `find_meat_source`
- Hunt: `best_hunt_target` → 移动 → 邻格攻击
  - `pack_hunter` 单狼只猎 `smallPrey`
  - 猎杀 → 猎物变尸体，叼肉

### 3.2 狐 (Fox)

**管线**: flee → fire → den_work → scavenge → hunt

- Flee: 躲人 + 躲狼
- Den work: 同狼，但窝是灌木狐窝
- Scavenge: 尸体 → 取肉（限 2 只/天）
- Hunt: 同狼，猎物为 `smallPrey`（田鼠、兔、雉鸡、竹鼠）

### 3.3 窝系统

- `den_for(card)` → 返回该卡的窝实体
- `should_build_den(card)` → 是否应建窝
- `update_den_work(card, delta)` → 建窝进度 + 入窝
- 狼窝：地下 overlay，`underground_state`
- 狐窝：占灌木，灌木消失变狐窝

---

## 四、覆盖觅食系统

**参照 Godot**: `scripts/world/behaviors/field_mouse_behavior.gd`

### 4.1 田鼠 (FieldMouse)

- 掘穴中 → idle
- 玩家邻格 → 躲/掘穴
- 狼在恐惧距 → 钻进灌丛 → 无灌丛则掘穴
- 灌丛中 → 吃虫（灌丛蚯蚓/甲虫）
- 无灌丛无虫 → 等微型生物恢复
- 觅食周期 + 饱歇
- 繁殖：公母 + 灌丛虫够

### 4.2 竹鼠 (BambooRat)

- 掘穴中 → idle
- 威胁 → 掘穴
- 觅食：近树格草皮（树周有虫/根）
- 同格 `in_ground` 的 `wildYam` → 拱山药 → `mark_ecology_fed`

---

## 五、水生系统

**参照 Godot**: `scripts/world/behaviors/aquatic_ecology_behavior.gd`

### 5.1 水藻 (Algae)

- 每个水格自动恢复（`ALGAE_REGEN_SECONDS`）
- 仅空格 spawn
- 被水虫和贝消耗

### 5.2 水虫 (WaterBug)

- 仅水格移动（M2 强化——A1 已做水格约束）
- 找藻 → 移动 → 啃藻 → `mark_ecology_fed`
- 无藻 → 游荡
- 迁入：全局 < 4 只 → 水格 spawn

### 5.3 鱼 (Fish)

- 仅水格移动
- 找水虫 → 移动 → 捕食 → `mark_ecology_fed`
- 无虫 → 游荡
- 迁入：全局 < 3 条 → 水格 spawn

### 5.4 贝 (Shellfish)

- 不移动（`sessile`）
- 同格有藻 → 滤食稳藻
- 无藻 → 待藻

---

## 六、分解 + 环境系统

**参照 Godot**: `scripts/world/environment_manager.gd`

### 6.1 尸体 → 腐殖土

- 尸体自然腐解倒计时（大尸体 vs 小尸体）
- 倒计时到 → 移除尸体，同格生成 humus
- humus：加速草恢复，N 游戏日后消散

### 6.2 腐坏 (Perishable)

- `perishable` 卡 → `PERISHABLE_TICKS` 倒计时
- 到零 → 自毁

### 6.3 陆虫 (LandBug)

- 尸体吸引（4 格内），同尸体最多 1 只
- 尸体格上 → 腐解速度 ×2
- 无尸体 → 游荡
- 全局上限 8

### 6.4 自然掉落

- 树林掉树枝（同 Godot `_tick_natural_drops`）
- 栎树/松树（`in_tree`）产橡子/松塔（同 `_tick_contained_producers`）

### 6.5 草皮再生 + 干草

- 河岸格：草恢复（`RIPARIAN_GRASS_INTERVAL`）
- 火旁草 → 变干草（`FIRE_DRY_GRASS_RANGE`）

### 6.6 岩壁风化 + 山药蔓延

- cliff 格 → `CLIFF_WEATHER_INTERVAL` → 邻格掉 shard
- wildYam (`in_ground`) → `YAM_SPREAD_INTERVAL` → 邻格新生（上限 12）

---

## 七、繁殖系统

**参照 Godot**: `scripts/world/population_manager.gd`

### 7.1 通用繁殖判定

- 公母配对
- 同种成体计数
- 生态压力（无捕食者近距）
- 对应标签门槛（`flocking`、`pack_hunter`、`prolific`）

### 7.2 各物种参数

| 物种 | 周期 | 门槛 | 每胎 |
|------|------|------|------|
| 羊 | 30s | flocking ≥ 3 | 1 |
| 鹿 | 30s | 无狼威胁 | 1 |
| 狼 | 30s | 有窝 | 1-2 |
| 狐 | 30s | 有窝 | 1 |
| 兔 | 3s | prolific | 3 |
| 田鼠 | 30s | 灌丛虫够 | 2-3 |
| 雉鸡 | 30s | flocking ≥ 3 | 1-2 |
| 竹鼠 | 30s | 无捕食 | 2-3 |
| 水牛 | 30s | 草够 | 1 |

### 7.3 生长

- 幼体 `age` 累计 → 达标 → 转成体
- 狼/狐幼体需肉喂养才成长

---

## 八、容纳 + 溢出

### 8.1 树容纳

- `in_tree` 实体：栎树、松树、鸟巢
- 点树 → UI 显示容纳列表
- 实体仍 tick（生产橡子/松塔）

### 8.2 水潭容纳

- `in_pool` 实体：藻、鱼、贝、水虫、菱角、莲
- 点水格 → UI 显示容纳列表

### 8.3 地下容纳

- `in_ground` 实体：野山药
- 点林下格 → UI 显示地下内容

### 8.4 采收交互

- 菱角/莲：人走池旁 → 空手采 → 出可食物 → 宿主冷却再生
- 野山药：人站格上 → 挖 → 出 `wildYamRoot`
- 贝：浅水摸 → 出 fishMeat
- 栎树/松树：点树摘橡子/松塔

---

## 九、主 Tick 循环

```rust
fn main_tick(
    world_state: &mut WorldState,
    spatial_index: &mut SpatialIndex,
    delta: f32,
) {
    // 1. 环境（每帧）
    grass_regen(world_state, spatial_index, delta);
    tick_natural_drops(world_state, spatial_index, delta);
    tick_contained_producers(world_state, spatial_index, delta);
    tick_perishable(world_state, delta);
    tick_corpses(world_state, delta);
    tick_land_bugs(world_state, spatial_index, delta);
    tick_cliff_weathering(world_state, spatial_index, delta);
    tick_underground_spread(world_state, delta);
    tick_pool_harvest_regen(world_state, delta);
    tick_aquatic(world_state, spatial_index, delta);

    // 2. 行为（每帧，已改 bucket——只 tick being+autonomous 实体）
    for entity in world_state.active_entities() {
        tick_entity(entity, world_state, spatial_index, delta);
    }

    // 3. 繁殖（每 REPRO_CYCLE）
    tick_reproduction(world_state, delta);

    // 4. 索引同步
    spatial_index.sync(world_state);
}
```

---

## 十、断言（M2 新增 50 条）

对标 Godot 993 条 L0 中的核心行为断言：

| 范围 | 数量 | 示例 |
|------|------|------|
| 食草行为 | 10 | 羊吃草后草消失、鹿逃狼、兔躲草、雉鸡不钻草 |
| 捕食行为 | 10 | 狼猎羊产出尸体、狐清腐取肉、pack_hunter 门槛 |
| 水生行为 | 8 | 鱼吃虫、贝滤食、藻再生、水虫迁入 |
| 分解 | 6 | 尸体→humus、perishable 自毁、landBug 趋尸 |
| 繁殖 | 8 | 羊 flocking 门槛、兔 prolific 高产、狼有窝才生 |
| 容纳 | 8 | 树内栎树、池内鱼贝、地下山药 |

---

## 十一、验收

- `cargo test`：100+ 断言 PASS（POC 20 + M1 30 + M2 50）
- `cargo run --release`：格网窗口，羊吃草狼猎鹿狐清腐，水潭鱼游贝滤，树内容纳，各色正确
- `cargo run --release -- --bench`：avg_tick_ms < 5ms

## 约束

- M1 数据层不动
- 每个行为函数单独一个 `src/systems/tick_*.rs` 文件
- 所有判定走 `world_rules.rs`，不允许系统内重复实现
- 记 FIX_LOG.md
