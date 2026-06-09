# 反应层统一行为引擎 — tick_reactive

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P1（架构推进——消除物种分支代码，所有动物用同一个行为引擎）

---

## 背景

当前行为层有四个按物种分类的文件：

```
tick_predator.rs      → 狼/狐的巡逻捕猎
tick_herbivore.rs     → 羊/鹿/兔的吃草
tick_cover_forager.rs → 田鼠/竹鼠的灌木觅食
tick_aquatic.rs       → 鱼/水虫的水中行为
```

每个文件里都有"如果是狼就…"、"如果是兔子就…"的分支。加一个新动物要改代码。

目标：合并为一个 `tick_reactive`，通过 EntityProfile 中的标签参数区分行为，不加代码分支。

---

## 设计依据

1. **三层认知架构**（游戏 AI 经典范式）：反射层（无行为引擎）→ 反应层（统一转向引擎）→ 深思层（PlayerMind）。本次只做反应层。
2. **Reynolds Steering Behaviors**（1999，行业标准）：Seek / Flee / Wander / Cohesion / Separation，本次适配为离散网格版本。
3. **Panksepp 情感系统**（神经科学）：SEEKING / FEAR 是所有可移动生物的绝对共性层。差异只在触发条件和强度。
4. **DF / RimWorld 验证**：所有动物共用同一 ThinkTree，差异在参数不在代码。

---

## 架构

```
tick_reactive(entity):
  1. 从 EntityProfile 读取行为参数
  2. 在 query_near_filtered 范围内寻找触发目标
  3. 按优先级排序当前有效的驱动
  4. 最高优先级决定本 tick 行为
  5. 执行 move_toward / flee_from / stay
```

### 与现有公理层的关系

```
tick_reactive (本 handoff)
  │ 决定"往哪走"——基于标签的 Seek/Flee/Wander 选择
  │
  ├─→ query_near_filtered  (公理层 perceive 过滤)
  ├─→ move_entity          (公理层 traverse + compose 检查)
  └─→ transform(Eat/Kill)  (公理层转化律)
```

---

## 实体标签新增（在 CardDef 中，不改代码只加标签）

### 驱动标签格式

```
drive:<behavior>(target=<tag>, range=N, priority=N, condition=<state>)
```

### 所有动物需加的标签

| 卡牌 | 新增标签 |
|---|---|
| sheep | `drive:seek(target=foodSource, range=6, priority=2)`, `drive:flee(target=predator, range=4, priority=3)`, `drive:flock(target=sheep, range=4, priority=1)` |
| wolf | `drive:seek(target=herbivore, range=8, priority=2)`, `drive:seek(target=smallPrey, range=8, priority=2)`, `drive:flee(target=tiger, range=6, priority=3)`, `drive:flock(target=wolf, range=6, priority=1)`, `drive:return_den(priority=1)` |
| deer | `drive:seek(target=foodSource, range=6, priority=2)`, `drive:flee(target=predator, range=5, priority=3)`, `drive:flock(target=deer, range=5, priority=1)` |
| rabbit | `drive:seek(target=foodSource, range=4, priority=2)`, `drive:flee(target=predator, range=4, priority=3)`, `drive:hide(target=grass, priority=3)` |
| fox | `drive:seek(target=herbivore, range=6, priority=2)`, `drive:seek(target=smallPrey, range=6, priority=2)`, `drive:scavenge(target=corpse, range=4, priority=2)`, `drive:flee(target=wolf, range=5, priority=3)`, `drive:return_den(priority=1)` |
| waterBuffalo | `drive:seek(target=foodSource, range=5, priority=2)`, `drive:flee(target=predator, range=4, priority=3)`, `drive:flock(target=waterBuffalo, range=3, priority=1)` |
| fieldMouse | `drive:seek(target=bush, range=4, priority=2)`, `drive:flee(target=predator, range=3, priority=3)`, `drive:hide(target=bush, priority=3)` |
| bambooRat | `drive:seek(target=grass, range=2, priority=2)`, `drive:seek(target=underground, range=2, priority=2)`, `drive:flee(target=predator, range=3, priority=3)` |
| pheasant | `drive:seek(target=foodSource, range=5, priority=2)`, `drive:flee(target=predator, range=4, priority=3)`, `drive:flock(target=pheasant, range=4, priority=1)` |
| fish | `drive:seek(target=algae, range=3, priority=2)`, `drive:flee(target=predator, range=3, priority=3)`, `drive:flock(target=fish, range=3, priority=1)` |
| waterBug | `drive:seek(target=algae, range=2, priority=2)`, `drive:flee(target=fish, range=2, priority=3)` |

### 两种特殊 drive

`drive:hide(target=X, priority=N)` — 当 detection range 内有 predator 且脚下有 X 标签的实体 → 隐藏（设置 `hidden_in_grass` 或 `in_burrow`）。这是 rabbit 钻草、mouse 钻灌木的逻辑。

`drive:return_den(priority=1)` — 已吃饱 + 不在巢穴 → `move_toward(den_x, den_y)`。这是狼/狐回巢的逻辑。priority=1 表示最低——只在没有 seek/flee 时触发。

---

## P0：新建 `src/systems/tick_reactive.rs`

```rust
use crate::axioms::{EntityProfile, TransformAction};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, wander};
use crate::world_state::{EcologyState, WorldState};

/// 反应层统一引擎 —— 所有有限智能实体每 tick 调用一次
pub fn tick_reactive(world: &mut WorldState, id: EntityId, delta: f32) {
    let (x, y, profile) = {
        let e = &world.entities[&id];
        (e.x, e.y, e.profile.clone())
    };

    // 1. 收集当前有效的驱动
    let drives = active_drives(world, id, x, y, &profile);

    // 2. 按 priority 降序排列
    let mut sorted: Vec<_> = drives.iter().collect();
    sorted.sort_by_key(|d| -(d.priority as i32));

    // 3. 执行最高优先级
    if let Some(drive) = sorted.first() {
        execute_drive(world, id, x, y, drive, &profile, delta);
    } else {
        // 无驱动 → 闲逛
        wander(world, id, x, y, world.tick_count);
        if let Some(e) = world.entities.get_mut(&id) {
            e.ecology_state = EcologyState::Idle;
        }
    }
}

struct ActiveDrive {
    behavior: DriveBehavior,
    target: Option<(EntityId, u8, u8)>,  // (target_id, tx, ty)
    priority: u8,
    range: u8,
}

enum DriveBehavior {
    Seek,       // move_toward(target)
    Flee,       // flee_from(target)
    Flock,      // move_toward(同类平均位置)
    Hide,       // 钻入/隐藏
    ReturnDen,  // 回巢
    Wander,     // 闲逛
    Idle,       // 不动
}

fn active_drives(world: &mut WorldState, id: EntityId, x: u8, y: u8, 
                 profile: &EntityProfile) -> Vec<ActiveDrive> {
    let mut drives = Vec::new();
    let entity = &world.entities[&id];

    // 从 profile.efficiencies 中已有的数据 + 新增的 drive 参数构建
    // drive 参数从 EntityProfile 中读取（在 build_profile 时从标签解析）
    
    for drive_def in &profile.drives {
        // 检查条件：是否已吃饱？
        if drive_def.condition_fed && entity.fed_today {
            continue;
        }
        
        match drive_def.behavior {
            DriveBehavior::Seek => {
                let targets = world.query_near_filtered(x, y, &drive_def.target_tag, 
                    drive_def.range, id);
                if let Some(&tid) = targets.first() {
                    if let Some(pos) = world.spatial_index.position(tid) {
                        drives.push(ActiveDrive {
                            behavior: DriveBehavior::Seek,
                            target: Some((tid, pos.0, pos.1)),
                            priority: drive_def.priority,
                            range: drive_def.range,
                        });
                    }
                }
            }
            DriveBehavior::Flee => {
                let threats = world.query_near_filtered(x, y, &drive_def.target_tag,
                    drive_def.range, id);
                if let Some(&tid) = threats.first() {
                    if let Some(pos) = world.spatial_index.position(tid) {
                        drives.push(ActiveDrive {
                            behavior: DriveBehavior::Flee,
                            target: Some((tid, pos.0, pos.1)),
                            priority: drive_def.priority,
                            range: drive_def.range,
                        });
                    }
                }
            }
            DriveBehavior::Flock => {
                let mates = world.query_near_filtered(x, y, &drive_def.target_tag,
                    drive_def.range, id);
                if mates.len() >= 2 {  // 至少2个同类才触发群聚
                    let avg = average_position(world, &mates);
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Flock,
                        target: Some((EntityId(0), avg.0, avg.1)),
                        priority: drive_def.priority,
                        range: drive_def.range,
                    });
                }
            }
            DriveBehavior::Hide => {
                let threats = world.query_near_filtered(x, y, "predator", 
                    drive_def.range, id);
                if !threats.is_empty() 
                   && world.has_tag_at(x, y, &drive_def.target_tag) 
                {
                    drives.push(ActiveDrive {
                        behavior: DriveBehavior::Hide,
                        target: None,
                        priority: drive_def.priority,
                        range: drive_def.range,
                    });
                }
            }
            DriveBehavior::ReturnDen => {
                if entity.fed_today {
                    if let Some(den_id) = entity.den_id {
                        if let Some(pos) = world.spatial_index.position(den_id) {
                            if crate::world_rules::chebyshev_distance(x, y, pos.0, pos.1) > 1 {
                                drives.push(ActiveDrive {
                                    behavior: DriveBehavior::ReturnDen,
                                    target: Some((den_id, pos.0, pos.1)),
                                    priority: drive_def.priority,
                                    range: 0,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    drives
}

fn execute_drive(world: &mut WorldState, id: EntityId, x: u8, y: u8,
                 drive: &ActiveDrive, profile: &EntityProfile, delta: f32) {
    match drive.behavior {
        DriveBehavior::Seek | DriveBehavior::Flock | DriveBehavior::ReturnDen => {
            if let Some((_, tx, ty)) = drive.target {
                if x == tx && y == ty {
                    // 到达目标 → 执行交互
                    match drive.behavior {
                        DriveBehavior::Seek => try_consume(world, id, drive.target.unwrap().0),
                        _ => {}
                    }
                } else {
                    move_toward(world, id, x, y, tx, ty);
                    if let Some(e) = world.entities.get_mut(&id) {
                        e.ecology_state = EcologyState::SeekingFood;
                    }
                }
            }
        }
        DriveBehavior::Flee => {
            if let Some((_, tx, ty)) = drive.target {
                flee_from(world, id, x, y, tx, ty);
                if let Some(e) = world.entities.get_mut(&id) {
                    e.ecology_state = EcologyState::Fleeing;
                }
            }
        }
        DriveBehavior::Hide => {
            if let Some(e) = world.entities.get_mut(&id) {
                e.in_burrow = true;
                e.ecology_state = EcologyState::Burrowed;
            }
        }
        DriveBehavior::Wander => {
            wander(world, id, x, y, world.tick_count);
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::Wandering;
            }
        }
        DriveBehavior::Idle => {
            if let Some(e) = world.entities.get_mut(&id) {
                e.ecology_state = EcologyState::Idle;
            }
        }
    }
}

fn try_consume(world: &mut WorldState, eater_id: EntityId, target_id: EntityId) {
    // 尝试吃掉目标（如果目标还有效且未 consume）
    // 调用 transform(Eat) / transform(Kill)
    let (eater_profile, target_profile) = {
        let eater = &world.entities[&eater_id];
        let target = &world.entities[&target_id];
        (eater.profile.clone(), target.profile.clone())
    };
    let result = crate::axioms::AxiomEngine::transform(
        &target_profile, &eater_profile, TransformAction::Eat);
    // 标记目标 consumed，更新 eater energy
    if let Some(target) = world.entities.get_mut(&target_id) {
        target.consumed = true;
        target.hp = 0;
    }
    world.remove_entity(target_id);
    if let Some(eater) = world.entities.get_mut(&eater_id) {
        eater.fed_today = true;
        eater.profile.energy = eater.profile.energy.saturating_add(result.energy_received);
        eater.ecology_state = EcologyState::Idle;
    }
}
```

---

## P0：修改 EntityProfile 增加 drive 定义

```rust
// axioms/profile.rs 新增

#[derive(Clone, Debug)]
pub struct DriveDef {
    pub behavior: DriveBehavior,
    pub target_tag: String,       // 目标标签，如 "foodSource" / "predator" / "sheep"
    pub range: u8,
    pub priority: u8,
    pub condition_fed: bool,      // true = 只在未吃饱时激活
}
```

`build_profile` 中解析 `drive:*` 标签：

```
标签: "drive:seek(target=foodSource, range=6, priority=2)"
  → DriveDef { behavior: Seek, target_tag: "foodSource", range: 6, priority: 2, condition_fed: true }

标签: "drive:flee(target=predator, range=4, priority=3)"
  → DriveDef { behavior: Flee, target_tag: "predator", range: 4, priority: 3, condition_fed: false }

标签: "drive:flock(target=sheep, range=4, priority=1)"
  → DriveDef { behavior: Flock, target_tag: "sheep", range: 4, priority: 1, condition_fed: false }

标签: "drive:hide(target=grass, priority=3)"
  → DriveDef { behavior: Hide, target_tag: "grass", range: 4, priority: 3, condition_fed: false }
  (range 默认 4，hide 的 range 表示"多大范围内有 predator 就触发")

标签: "drive:return_den(priority=1)"
  → DriveDef { behavior: ReturnDen, target_tag: "", range: 0, priority: 1, condition_fed: false }
  (已吃饱条件在 execute 中检查)
```

标签解析函数 `parse_drives(tags: &[String]) -> Vec<DriveDef>`，类似现有的 `parse_bridges` / `parse_channels`。

---

## P0：修改 main_tick 接入

```rust
// systems/main_tick.rs

// 替换原来的 flush_herbivore_tick + flush_predator_patrol
fn flush_reactive_tick(world: &mut WorldState, delta: f32) {
    let ids: Vec<EntityId> = world.entities.iter()
        .filter(|(_, e)| e.needs_grazing_tick || e.needs_patrol)
        .map(|(id, _)| *id)
        .collect();
    for id in ids {
        if let Some(e) = world.entities.get_mut(&id) {
            e.needs_grazing_tick = false;
            e.needs_patrol = false;
        }
        tick_reactive(world, id, delta);
    }
}
```

`main_tick` 中的：
```
mark_baseline_predator_patrol(world);
mark_baseline_herbivore_tick(world);
flush_predator_patrol(world, delta);
flush_herbivore_tick(world, delta);
```
替换为：
```
mark_baseline_reactive_tick(world);
flush_reactive_tick(world, delta);
```

`mark_baseline_reactive_tick`：对所有 `is_autonomous` 的实体标记 `needs_grazing_tick = true`（统一标记，不区分捕食者/食草）。

---

## P0：删除旧文件

删除以下四个文件（逻辑全部合并到 `tick_reactive.rs`）：

- `src/systems/tick_predator.rs`
- `src/systems/tick_herbivore.rs`
- `src/systems/tick_cover_forager.rs`
- `src/systems/tick_aquatic.rs`

---

## P1：修改 event_registry

`EventRegistry::tick_non_predator_ecology` 中旧的分发逻辑：

```rust
// 旧
match ecosystem_behavior_key(def, &type_name) {
    BEHAVIOR_HERBIVORE_GRAZER => tick_one_grazer(...),
    BEHAVIOR_COVER_FORAGER => tick_cover_forager(...),
    "traveler" => tick_ambient_wander(...),
    ...
}

// 新
// 所有非捕食者实体统一走 tick_reactive
pub fn tick_non_predator_ecology(world, id, def, delta) {
    tick_reactive(world, id, delta);
}
```

捕食者也走同一个：

```rust
pub fn tick_entity_ecology(world, id, delta) {
    // predator / mesopredator 也走 tick_reactive——它们的 seek target 是 herbivore
    tick_reactive(world, id, delta);
}
```

---

## P1：标签解析（profile.rs 新增）

在 `EntityProfile` 中新增字段：

```rust
pub drives: SmallVec<[DriveDef; 6]>,
```

`build_profile` 末尾调用：

```rust
drives: parse_drives(tags),
```

`parse_drives` 函数：解析 `drive:<behavior>(target=<tag>, range=N, priority=N)` 格式。与现有的 `parse_bridges` / `parse_channels` / `parse_efficiencies` 同模式。

---

## 修改文件清单

| 文件 | 改动 |
|---|---|
| `src/systems/tick_reactive.rs` | **新建**：统一反应层引擎 |
| `src/systems/mod.rs` | 移除旧文件引用，加入 `pub mod tick_reactive;` |
| `src/systems/main_tick.rs` | 替换 mark/flush 为统一 reactive 版本 |
| `src/axioms/profile.rs` | EntityProfile 加 `drives: Vec<DriveDef>`；新增 `parse_drives` |
| `src/event_registry.rs` | 简化 `tick_non_predator_ecology` → 直接调用 `tick_reactive` |
| `src/lib.rs` | 移除旧函数的 pub export（如 `flush_herbivore_tick`） |
| `src/systems/tick_predator.rs` | **删除** |
| `src/systems/tick_herbivore.rs` | **删除** |
| `src/systems/tick_cover_forager.rs` | **删除** |
| `src/systems/tick_aquatic.rs` | **删除** |
| `assets/card_defs.ron` | 所有动物卡牌加 `drive:*` 标签（按上表） |

---

## 验收标准

1. `cargo test` — 所有现有测试 PASS
2. `cargo run --release -- --smoke-test` — SMOKE: PASS
3. 游戏启动后运行：
   - 羊吃草、狼追羊、兔躲草——行为与改动前一致
   - 狐狸走水现象仍然被公理层阻止（不是本 handoff 的职责，但验证不退化）
   - 羊不堆叠超过承载（公理层 compose 持续生效）
4. 添加新动物卡只需加 `drive:*` 标签，不写新代码——验证方式：临时改 sheep 的 `drive:seek(target=...)` 为 `target=bush`，看羊是否走向灌木。

## 向后兼容

- `EcologyState` 枚举不变
- `Entity` 的 `needs_grazing_tick` / `needs_patrol` 字段保留（用于标记）
- `move_toward` / `flee_from` / `wander` 函数签名不变
- `query_near_filtered` 不变
- CardDef 中现有的 `tags` 字段不变，只新增 `drive:*` 标签

## 约束

- 不碰 `tick_environment.rs` / `tick_reproduction.rs` / `tick_containment.rs`
- 不碰 `card_defs.ron` 中已有的非行为标签
- 不碰 Player AI
- 不碰 Observer/Rete
