# 因果抽象引擎 — 世界公理层落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P0（地基级别 —— 解决狐狸水上走、羊堆叠、感知跨介质等所有系统性荒谬）
**设计哲学**: 公理不是"另一套标签"。公理是处理标签组合的引擎，类比物理引擎处理质量/速度/力 —— 引擎不知道什么是"狐狸"或"水"，只知道组合律、穿越律、感知律、转化律四条不可配置的定律。所有具体参数来自标签。

---

## 架构总览

```
行为层 (tick_herbivore, tick_predator ...)
    │ 调用引擎查询 EntityProfile（不读原始标签）
    ▼
因果抽象引擎 (AxiomEngine) —— 纯函数，无状态
    ├── compose(&CellSlot, &EntityProfile) → Composition
    ├── traverse(&EntityProfile, from, to) → Traversal
    ├── perceive(&observer: &EntityProfile, &target: &EntityProfile, distance) → Perception
    ├── transform(&source: &EntityProfile, &target: &EntityProfile, action) → Transformation
    └── trace(event) → 写入 CausalStorage
    │
    │ 引擎读取 EntityProfile（标签预计算缓存）
    │ 不直接读原始标签 Vec<String>
    ▼
EntityProfile —— spawn 时从标签构建，动态标签变化时增量更新
CellComposition —— [[CellSlot; H]; W]，增量维护 current_size
CausalStorage —— Full / Ring(N) / Off 三级

标签系统 —— 完全自由的创作层，引擎只读取它产生的 Profile
WorldState —— 数据持有层
```

---

## P0-1：新建 `src/axioms/` 模块

### `src/axioms/mod.rs` —— 引擎入口

```rust
pub mod profile;
pub mod composition;
pub mod laws;
pub mod causality;

use profile::EntityProfile;
use composition::{CellSlot, CellComposition};
use causality::{CausalEvent, CausalStorage};
use crate::world_state::WorldState;

/// 因果抽象引擎 —— 纯函数集合
/// 不持有任何配置，所有参数来自 EntityProfile（预计算自标签）
pub struct AxiomEngine;

impl AxiomEngine {
    /// 从实体标签构建 profile（spawn 时调用一次）
    pub fn build_profile(entity: &crate::world_state::Entity, 
                         world: &WorldState) -> EntityProfile;

    /// 增量更新 profile（仅动态标签变化时，如 energy 消耗）
    pub fn update_profile_dynamic(profile: &mut EntityProfile, 
                                  entity: &crate::world_state::Entity);

    // ── 四条定律 ──
    pub fn compose(cell: &CellSlot, incoming: &EntityProfile) -> laws::Composition;
    pub fn traverse(profile: &EntityProfile, from: Medium, to: Medium) -> laws::Traversal;
    pub fn perceive(observer: &EntityProfile, target: &EntityProfile, distance: u8) -> laws::Perception;
    pub fn transform(source: &EntityProfile, target: &EntityProfile, 
                     action: laws::TransformAction) -> laws::Transformation;
    
    // ── 因果记录 ──
    pub fn trace(storage: &mut CausalStorage, event: CausalEvent);
}
```

**约束**：
- 引擎不 import `card_def` —— 不接触 CardDef
- 引擎不 import 任何 `systems/` —— 不接触行为函数
- 引擎不持有 `&WorldState` 的 mutable 引用（`trace` 只写 storage）
- `Medium` 是一个 newtype：`Medium(String)`，不是枚举 —— 支持任意新介质

---

### `src/axioms/profile.rs` —— EntityProfile

```rust
use smallvec::SmallVec;
use crate::spatial_index::EntityId;

/// 从标签预计算一次，引擎只读这个不读原始标签
#[derive(Clone)]
pub struct EntityProfile {
    pub entity_id: EntityId,
    pub type_name: String,              // 调试用

    // ── 组合律所需 ──
    pub size: u8,                       // 标签 size:N，缺省 1
    pub incorporeal: bool,              // 标签 capability.incorporeal

    // ── 穿越律所需 ──
    pub native_medium: Medium,          // 标签 medium:<name>，缺省 "land"
    pub bridges: SmallVec<[(Medium, Medium); 4]>,  // 标签 bridge:<from>-><to>
    pub is_omnimedium: bool,            // 标签 bridge:omnimedium

    // ── 感知律所需 ──
    pub channels: SmallVec<[ChannelDef; 4]>,
    pub cross_perception: SmallVec<[Medium; 2]>,   // bridge:perceive-><medium>
    pub visibility_mod: f32,            // 标签 visibility:<prop>(m=F)，默认 1.0
    pub keen_eyed_mod: f32,            // 标签 perception:keen_eyed(m=F)，默认 1.0
    
    // ── 转化律所需 ──
    pub energy: u32,                    // 标签 energy:N，缺省 0
    pub efficiencies: SmallVec<[(TransformAction, f32); 6]>,
    
    // ── 介质信息（从所在格获取，非标签） ──
    pub current_medium: Medium,         // 当前位置的介质
}

#[derive(Clone)]
pub struct ChannelDef {
    pub kind: String,    // "visual" / "olfactory" / "auditory" / 自定义
    pub range: u8,       // 标签 perception:<kind>(r=N)，缺省 1
}

pub type Medium = String;   // "land", "water", "air", "underground", "canopy", 任意
```

**build_profile 逻辑**：
```
1. size = 实体标签中匹配 "size:N" 取值 N，无则 1
2. incorporeal = 实体标签中是否有 "capability.incorporeal"
3. native_medium = 实体标签中匹配 "medium:<name>" → name，无则 "land"
4. bridges = 遍历实体标签，匹配 "bridge:<from>-><to>"，收集所有对
5. is_omnimedium = 实体标签中是否有 "bridge:omnimedium"
6. channels = 遍历实体标签，匹配 "perception:<kind>(r=N)"，收集所有通道
7. cross_perception = 遍历实体标签，匹配 "bridge:perceive-><medium>"，收集
8. visibility_mod = 标签 "visibility:tiny"→0.1, "visibility:transparent"→0.0，无则 1.0
9. keen_eyed_mod = 标签 "perception:keen_eyed(m=F)"→F，无则 1.0
10. energy = 标签 "energy:N" → N，无则 0
11. efficiencies = 遍历标签 "efficiency:<action>(m=F)"，收集
12. current_medium = 从当前格介质地图获取（不是标签）
```

---

### `src/axioms/composition.rs` —— 空间承载

```rust
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};

pub type Medium = String;

/// 每个格的承载状态
#[derive(Clone)]
pub struct CellSlot {
    pub medium: Medium,        // 该格介质
    pub max_size: u8,          // 介质标签 density:N → N，缺省 1
    pub current_size: u8,      // 增量维护，spawn/move/remove 时同步
}

pub struct CellComposition {
    pub grid: Box<[[CellSlot; GRID_WIDTH as usize]; GRID_HEIGHT as usize]>,
}

impl CellComposition {
    /// 从 WorldState.terrain + 介质标签初始化
    pub fn from_world(world: &WorldState) -> Self;

    /// 实体进入：current_size += profile.size
    /// 调用前必须先检查 compose() 是否允许
    pub fn occupy(&mut self, x: u8, y: u8, profile: &EntityProfile);
    
    /// 实体离开：current_size -= profile.size
    /// 注意：size 减后不能低于 0（防御性断言）
    pub fn vacate(&mut self, x: u8, y: u8, profile: &EntityProfile);
}
```

**from_world 逻辑**：
```
1. 遍历世界网格：
   - 如果 terrain_at(x,y) 是 "river"/"ford"/"pool" → medium = "water"
   - 否则 → medium = "land"
   - 每格上方自动有 "air" 层（但不占用 CellSlot —— air 不参与承载）
2. max_size = 从该介质定义中读取 density:N 标签，无则默认值：
   - land → 3
   - water → 2
   - underground → 1
   - canopy → 2
3. current_size = 遍历所有实体，对本格 occupant 求和
```

**重要**：air 介质不参与空间承载（`incorporeal` 概念在 air 中天然成立）。有树的格额外有 canopy 介质层，需单独计算承载。

---

### `src/axioms/laws.rs` —— 四条纯函数定律

```rust
use super::profile::{EntityProfile, ChannelDef, Medium};
use super::composition::CellSlot;
use smallvec::SmallVec;

// ─────────── 组合律 ───────────

pub enum Composition {
    Allowed { remaining: u8 },
    Denied { current: u8, max: u8 },
}

/// 定律：sum(entity.size) ≤ cell.max_size
pub fn compose(cell: &CellSlot, incoming: &EntityProfile) -> Composition {
    if incoming.incorporeal {
        return Composition::Allowed { remaining: cell.max_size - cell.current_size };
    }
    let new_total = cell.current_size + incoming.size;
    if new_total <= cell.max_size {
        Composition::Allowed { remaining: cell.max_size - new_total }
    } else {
        Composition::Denied { current: cell.current_size, max: cell.max_size }
    }
}

// ─────────── 穿越律 ───────────

pub enum Traversal {
    Allowed,
    Denied { from: Medium, to: Medium, missing: String },
}

/// 定律：from==to → 允许；bridge:from->to → 允许；medium.to → 允许；omnimedium → 允许
pub fn traverse(profile: &EntityProfile, from: &Medium, to: &Medium) -> Traversal {
    if from == to {
        return Traversal::Allowed;
    }
    if profile.is_omnimedium {
        return Traversal::Allowed;
    }
    // 检查 native_medium 是否就是目标介质
    if &profile.native_medium == to {
        return Traversal::Allowed;
    }
    // 检查桥接
    for (bf, bt) in &profile.bridges {
        if bf == from && bt == to {
            return Traversal::Allowed;
        }
    }
    Traversal::Denied {
        from: from.clone(),
        to: to.clone(),
        missing: format!("bridge:{}->{}", from, to),
    }
}

// ─────────── 感知律 ───────────

pub enum Perception {
    Detected { channels: SmallVec<[PerceptionChannel; 4]> },
    Undetected,
}

pub struct PerceptionChannel {
    pub kind: String,
    pub effective_range: u8,
}

/// 定律：有效范围 = channel.range × keen_eyed_mod × visibility_mod × 介质传导，如果 ≥ distance 则可感知
pub fn perceive(observer: &EntityProfile, target: &EntityProfile, distance: u8,
                observer_medium_conduction: &[(String, f32)],
                target_medium_conduction: &[(String, f32)])
    -> Perception
{
    // 零成本剪枝：跨介质且无桥接 → 直接不可感知
    if observer.current_medium != target.current_medium 
       && !observer.cross_perception.contains(&target.current_medium) 
    {
        return Perception::Undetected;
    }

    let mut detected = SmallVec::new();
    for ch in &observer.channels {
        // 取两个介质传导系数的最小值
        let obs_cond = observer_medium_conduction.iter()
            .find(|(k, _)| k == &ch.kind).map(|(_, v)| *v).unwrap_or(1.0);
        let tgt_cond = target_medium_conduction.iter()
            .find(|(k, _)| k == &ch.kind).map(|(_, v)| *v).unwrap_or(1.0);
        let conduction = obs_cond.min(tgt_cond);

        let effective = (ch.range as f32 * observer.keen_eyed_mod 
                         * target.visibility_mod * conduction) as u8;
        if effective >= distance {
            detected.push(PerceptionChannel {
                kind: ch.kind.clone(),
                effective_range: effective,
            });
        }
    }
    if detected.is_empty() {
        Perception::Undetected
    } else {
        Perception::Detected { channels: detected }
    }
}

// ─────────── 转化律 ───────────

#[derive(Clone, PartialEq, Eq)]
pub enum TransformAction {
    Eat,
    Kill,
    Harvest,
    Decay,
    Grow,
    Spawn,
}

pub struct Transformation {
    pub energy_drawn: u32,
    pub energy_received: u32,
    pub energy_lost: u32,
}

/// 定律：energy_drawn = source.energy；received = drawn × efficiency；lost = drawn - received
pub fn transform(source: &EntityProfile, target: &EntityProfile, 
                 action: TransformAction) -> Transformation 
{
    let drawn = source.energy;
    let efficiency = target.efficiencies.iter()
        .find(|(act, _)| act == &action)
        .map(|(_, m)| *m)
        .unwrap_or(0.5);  // 默认 50% 转化率
    let received = ((drawn as f32) * efficiency) as u32;
    let lost = drawn - received;
    Transformation { energy_drawn: drawn, energy_received: received, energy_lost: lost }
}
```

**注意**：`observer_medium_conduction` 和 `target_medium_conduction` 来自介质的 `conduction:<channel>(m=F)` 标签。因为引擎不读 CardDef，这部分数据需要在 WorldState 初始化时预计算为一个 `HashMap<Medium, Vec<(ChannelKind, f32)>>`，传给 perception 计算。这个 HashMap 是只读的，在介质定义加载后构建一次。

---

### `src/axioms/causality.rs` —— 因果记录

```rust
use std::collections::VecDeque;

#[derive(Clone)]
pub struct CausalEvent {
    pub tick: u64,
    pub cause_entity_id: u64,
    pub cause_tag: String,         // 触发该事件的能力标签
    pub effect_entity_id: u64,
    pub effect_description: String, // "moved(5,3→5,4)" / "eaten" / "killed" / "spawned(grass)"
}

pub enum CausalStorage {
    Full { events: Vec<CausalEvent> },
    Ring { buf: VecDeque<CausalEvent>, cap: usize },
    Off,
}

impl CausalStorage {
    pub fn push(&mut self, event: CausalEvent) {
        match self {
            CausalStorage::Full { events } => events.push(event),
            CausalStorage::Ring { buf, cap } => {
                if buf.len() >= *cap { buf.pop_front(); }
                buf.push_back(event);
            }
            CausalStorage::Off => {}
        }
    }

    /// smoke test 使用 Ring(2000)，debug 使用 Full，release 使用 Off
    pub fn for_mode(is_smoke_test: bool, is_debug: bool) -> Self {
        if is_smoke_test { 
            CausalStorage::Ring { buf: VecDeque::with_capacity(2000), cap: 2000 }
        } else if is_debug {
            CausalStorage::Full { events: Vec::new() }
        } else {
            CausalStorage::Off
        }
    }
}
```

**约束**：引擎不持有 CausalStorage —— 它从 WorldState 传入，trace 只做 push。不分配额外的 String（effect_description 可以用 `&'static str` 或整数编码，这里为可读性先用 String，后续优化）。

---

## P0-2：修改现有文件

### `src/world_state.rs`

**Entity 增加字段**：
```rust
pub struct Entity {
    // ... 现有字段 ...
    pub profile: EntityProfile,  // 新增：标签预计算缓存
}
```

**WorldState 增加字段**：
```rust
pub struct WorldState {
    // ... 现有字段 ...
    pub cell_composition: CellComposition,     // 新增
    pub causal_storage: CausalStorage,         // 新增
    pub medium_conductions: HashMap<Medium, Vec<(String, f32)>>, // 新增：介质传导缓存
}
```

**修改 `spawn`**：
```rust
fn spawn(&mut self, type_name: &str, x: u8, y: u8) -> EntityId {
    // 1. 先构建 Entity（不含 profile），获取 id
    // 2. profile = AxiomEngine::build_profile(&entity, self)
    // 3. compose(&self.cell_composition.grid[y][x], &profile) 
    //    如果是 Denied → 寻找附近空位或返回错误
    // 4. self.cell_composition.occupy(x, y, &profile)
    // 5. entity.profile = profile
    // 6. entity.current_medium = cell_composition.grid[y][x].medium
}
```

**修改 `move_entity`**：
```rust
fn move_entity(&mut self, id: EntityId, x: u8, y: u8) -> MoveResult {
    // 1. 获取 entity.profile
    // 2. 旧格 = (entity.x, entity.y)
    // 3. traverse(&profile, &profile.current_medium, &cell_composition[y][x].medium)
    //    如果 Denied → 返回 MoveResult::Blocked（介质不兼容）
    // 4. compose(&cell_composition.grid[y][x], &profile)
    //    如果 Denied → 返回 MoveResult::Blocked（承载满）
    // 5. self.cell_composition.vacate(old_x, old_y, &profile)
    // 6. self.cell_composition.occupy(x, y, &profile)
    // 7. 更新 entity.x, entity.y
    // 8. entity.profile.current_medium = cell_composition.grid[y][x].medium
    // 9. 更新 spatial_index
    // 10. trace: CausalEvent { tick, cause_entity_id: id.0, cause_tag: "move", ... }
}
```

**修改 `remove_entity`**：
```rust
fn remove_entity(&mut self, id: EntityId) {
    // 1. 清理前调用 cell_composition.vacate(x, y, &entity.profile)
    // 2. 原有逻辑（从 entities map 移除、从 spatial_index 移除）
    // 3. trace: CausalEvent { ..., effect_description: "removed" }
}
```

### `src/spatial_index.rs`

**新增 `query_near_filtered`**：

```rust
/// 在 query_near 基础上，用感知律过滤掉不可感知的实体
pub fn query_near_filtered(
    &self,
    x: u8, y: u8, tag: &str, range: u8,
    observer_id: EntityId,
    world: &WorldState,
) -> Vec<EntityId> {
    let raw = self.query_near(x, y, tag, range);
    let Some(observer) = world.entities.get(&observer_id) else { return raw; };
    let obs_profile = &observer.profile;
    
    raw.into_iter().filter(|id| {
        let Some(target) = world.entities.get(id) else { return false; };
        let tgt_profile = &target.profile;
        let dist = chebyshev_distance(observer.x, observer.y, target.x, target.y) as u8;
        matches!(
            AxiomEngine::perceive(
                obs_profile, tgt_profile, dist,
                &world.get_medium_conduction(&obs_profile.current_medium),
                &world.get_medium_conduction(&tgt_profile.current_medium),
            ),
            laws::Perception::Detected { .. }
        )
    }).collect()
}
```

**保留旧接口 `query_near` 不变**。

### `src/systems/tick_herbivore.rs`

`try_eat_grass` 中：
```rust
// 修改前
let grass_near = world.spatial_index.query_near(x, y, "foodSource", 6);

// 修改后
let grass_near = world.spatial_index.query_near_filtered(x, y, "foodSource", 6, id, world);
```

`eat_grass_at` 中增加转化调用：
```rust
// 消耗后
let (src_profile, tgt_profile) = /* 获取 source 和 eater 的 profile */;
let result = AxiomEngine::transform(&src_profile, tgt_profile, TransformAction::Eat);
// 将 result.energy_received 加入到 eater 的 energy
// 更新 eater.profile.energy
// 将 result.energy_lost 加入环境元气池（当前可选/跳过）
```

### `src/systems/tick_predator.rs`

捕食者 patrol 查询猎物时使用 `query_near_filtered`。kill 时调用 `transform(Kill)`。

### `src/systems/tick_reproduction.rs`

spawn 前调用 `compose` 检查目标格承载力。spawn 后 `transform(Spawn)`。

### `src/lib.rs`

新增 `pub mod axioms;`

---

## P0-3：向后兼容

| 接口 | 策略 |
|---|---|
| `query_near` | 保留不变，行为系统逐步迁移到 `query_near_filtered` |
| `move_entity` | 签名不变，内部增加公理检查。返回 `MoveResult` 枚举代替原地修改 |
| `spawn` | 内部增加 compose 检查，找不到空位时向外寻找 |
| 现有 167 个测试 | 全部应继续 PASS（引擎逻辑不影响纯数据测试） |
| smoke test | 应 PASS：介质隔离后食草动物只吃同介质食物，捕食者只追同介质猎物 |

---

## P1：介质传导缓存

在 `WorldState` 中新增：

```rust
/// 从介质定义中预计算的传导系数表
/// key = 介质名，value = [(通道名, 传导系数)]
pub medium_conductions: HashMap<String, Vec<(String, f32)>>,
```

**初始化**（在 WorldState 构建时）：
- 读取所有介质定义（可以是一个 RON 文件或硬编码初始值）
- 对每个介质，提取 `conduction:<channel>(m=F)` 标签
- 无标签的通道默认 `m=1.0`
- 默认值：
  - water: `[("visual", 0.5), ("olfactory", 1.2), ("auditory", 1.5)]`
  - land: `[("visual", 1.0), ("olfactory", 1.0), ("auditory", 1.0)]`
  - air: `[("visual", 1.5), ("olfactory", 0.3), ("auditory", 0.5)]`
  - underground: `[("visual", 0.0), ("olfactory", 0.8), ("auditory", 1.2)]`

---

## P1：标签命名规范（维护文档）

引擎通过标签模式匹配读取参数。新增标签需遵守以下命名约定：

```
size:N                              # 抽象尺寸，N 为整数。无标签默认 1
medium:<name>                       # 原生介质。无标签默认 land
bridge:<from>-><to>                 # 跨介质穿越桥接
bridge:omnimedium                   # 全介质存在（恶魔、幽灵）
bridge:perceive-><medium>           # 跨介质感知桥接
capability:incorporeal              # 不占物理承载
perception:<channel>(r=N)           # 感知通道及范围
perception:keen_eyed(m=F)           # 感知修正系数
visibility:<prop>(m=F)              # 可感知修正（tiny=0.1, transparent=0.0 等）
energy:N                            # 能量值
efficiency:<action>(m=F)            # 转化效率（action=eat/kill/harvest 等）
density:N                           # 介质标签：承载密度
conduction:<channel>(m=F)           # 介质标签：感知传导系数
```

---

## 涉及文件总清单

| 文件 | 改动类型 | 说明 |
|---|---|---|
| `src/axioms/mod.rs` | 新建 | AxiomEngine + build_profile + update_profile_dynamic |
| `src/axioms/profile.rs` | 新建 | EntityProfile 定义 + 构建逻辑 |
| `src/axioms/composition.rs` | 新建 | CellSlot + CellComposition |
| `src/axioms/laws.rs` | 新建 | compose/traverse/perceive/transform 纯函数 |
| `src/axioms/causality.rs` | 新建 | CausalEvent + CausalStorage |
| `src/lib.rs` | 修改 | + `pub mod axioms;` |
| `src/world_state.rs` | 修改 | Entity +profile；WorldState +cell_composition +causal_storage +medium_conductions；spawn/move_entity/remove_entity 接入引擎 |
| `src/spatial_index.rs` | 新增 | `query_near_filtered` |
| `src/systems/tick_herbivore.rs` | 修改 | 使用 query_near_filtered + transform(Eat) |
| `src/systems/tick_predator.rs` | 修改 | 使用 query_near_filtered + transform(Kill) |
| `src/systems/tick_reproduction.rs` | 修改 | spawn 前 compose 检查 |
| `src/systems/main_tick.rs` | 修改 | tick 开始时/结束时因果存储切换？不需要——causal_storage 在 WorldState 中 |

---

## 验收标准

1. `cargo test` — 167+ 测试全部 PASS
2. `cargo run --release -- --smoke-test` — SMOKE: PASS
3. `cargo run --release` 游戏启动：
   - 狐狸不在水上走（traverse 拦截：缺少 bridge:land->water）
   - 狼不追水里的鱼（perceive 拦截：cross-perception 缺失）
   - 羊不堆叠超过承载（compose 拦截：sum(size) > max_size）
   - 水牛可以入水（bridge:land->water 标签生效）
   - 鱼不被陆上捕食者感知（介质隔离）

## 约束

- 不碰 Observer/Rete 架构
- 不碰 `card_defs.ron` 中现有标签（向后兼容：无新标签 → 默认行为 = 旧行为）
- 旧的 `query_near` 保留不删
- `compose` 的 Denied 不应导致 panic —— 返回 MoveResult::Blocked，调用方决定降级行为（如停在原地）
- EntityProfile 的 `build_profile` 中标签解析失败 → 使用默认值，不 panic
