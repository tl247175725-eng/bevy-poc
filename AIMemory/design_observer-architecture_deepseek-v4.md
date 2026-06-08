# Observer 事件驱动架构设计

**状态**: 设计文档，M2 落地后转 handoff
**日期**: 2026-06-06
**前提**: Bevy 0.14+ 原生 Observer + Trigger 系统

---

## 一、为什么做

当前 M2 用 bucket tick——按 `being` 标签取活跃卡，每帧全部轮询一遍。0.0024ms，完全够。

但终局场景（商业枢纽 + 阶级涌现 + 数千条标签关系联动），轮询的 O(n) 基数会涨。Observer 把模型从"每帧问所有卡"变成"卡变了才触发规则"。O(n) → O(变化量)。

**现在做不是因为不够快。是因为代码还小——重构成本最低。**

---

## 二、核心思路

**不改变任何行为逻辑。** 只改"怎么调度"。

```
当前（M2 轮询式）：
  main_tick:
    for each entity in active_entities:
      tick_herbivore(entity)
      tick_predator(entity)
      ...

Observer 版（事件驱动）：
  卡移动 → Trigger<OnMove> → Observer 自动触发
    → 查 SpatialIndex: "新位置附近有 predator 吗？"
    → 有 → 狼的 HuntSystem 被唤醒 → 评估猎物
    → 无 → 不触发任何事
  
  狼猎杀羊 → Trigger<OnKill> → Observer 自动触发
    → 羊移除
    → 狼标记饱腹
    → 生成尸体
    → 尸体吸引 landBug（Trigger<OnCorpseSpawn>）
```

**每帧不再遍历卡。** 只响应状态变化。绝大多数卡在绝大多数帧里什么都不做——Observer 架构下它们零开销。

---

## 三、设计映射：标签 → Bevy Component

当前 M1/M2 用 `Vec<String>` 存标签。Observer 版本改为编译期 Component：

```rust
// 当前（M1/M2）
card_def.tags.contains("predator")  // 运行时字符串比较

// Observer 版
#[derive(Component)]
struct Predator;

// 狼实体 = 带 Predator + PackHunter + Being 等 Component
// 羊实体 = 带 Herbivore + Flocking + Being 等 Component
```

**代价**：`card_defs.ron` 的 `tags: ["predator", "pack_hunter"]` 不能直接用了——标签变成了 Rust 类型。每张卡定义需要映射逻辑。

**收益**：`Query<&Predator>` 是编译器生成的——只遍历有 Predator 的实体，零运行时开销。

### 变通方案：不全部改 Component

保持 `card_defs.ron` 的字符串标签，**唯独 Observer 调度用 Component**。新增一层编译期映射：

```rust
// 卡加载时，按标签自动插入对应 Component
fn spawn_card(world: &mut World, card_def: &CardDef) {
    let mut entity = world.spawn_empty();
    for tag in &card_def.tags {
        match tag.as_str() {
            "predator" => entity.insert(Predator),
            "herbivore" => entity.insert(Herbivore),
            "flocking" => entity.insert(Flocking),
            // ...
            _ => {}  // 非调度相关标签不变
        }
    }
}
```

`card_defs.ron` 不改。设计工作流不变。新增调度标签只需加一行 match。

---

## 四、事件定义

### 4.1 核心 Trigger

| Trigger | 触发时机 | 唤醒的系统 |
|---------|---------|----------|
| `OnMove(entity, old_pos, new_pos)` | 卡移动完成 | HuntSystem, FleeSystem, ForageSystem |
| `OnSpawn(entity, pos)` | 新卡生成 | SpatialIndex 注册, 繁殖链 |
| `OnDespawn(entity)` | 卡移除 | SpatialIndex 注销, 食物网通知 |
| `OnKill(predator, prey)` | 捕猎成功 | 尸体生成, 饱腹标记 |
| `OnEat(entity, food)` | 进食完成 | 饱腹标记, 生长, HP 恢复 |
| `OnCorpseSpawn(entity, pos)` | 尸体出现 | LandBug 吸引 |
| `OnStarve(entity)` | 饥饿死亡 | 尸体生成, 种群统计 |
| `OnReproduce(parent1, parent2, child)` | 繁殖成功 | 幼体照料链 |

### 4.2 定时 tick（Observer 不覆盖的）

以下仍需 `FixedTimestep` 系统，因为它们不是事件驱动的：

| 系统 | 原因 |
|------|------|
| 草皮再生 | 环境计时，无触发源 |
| 腐坏倒计时 | 纯时间驱动 |
| 腐解计时 | 纯时间驱动 |
| 岩壁风化 | 纯时间驱动 |
| 繁殖冷却 | 周期性检查 |

---

## 五、系统架构

```
                    ┌─────────────────┐
                    │   FixedTimestep  │
                    │  (环境 tick)     │
                    │  草/腐/风化     │
                    └────────┬────────┘
                             │ 产出卡/状态变化
                             ▼
┌──────────┐   Trigger   ┌─────────────────┐
│ Bevy     │◄───────────│  WorldState      │
│ Observer │            │  修改 (spawn/    │
│ System   │            │  move/kill/...)  │
└────┬─────┘            └─────────────────┘
     │ 自动触发
     ▼
┌──────────────────────────────────────────┐
│          行为 Observer 链                 │
│                                          │
│  OnMove → HuntObserver                   │
│         → FleeObserver                   │
│         → ForageObserver                 │
│                                          │
│  OnKill → CorpseObserver                 │
│         → HungerObserver                 │
│                                          │
│  OnCorpseSpawn → LandBugObserver         │
│                                          │
│  OnEat → GrowthObserver                  │
│        → ReproductionObserver            │
└──────────────────────────────────────────┘
```

**关键规则**：每个 Observer 只做一件事。多个 Observer 可以监听同一 Trigger。Observer 之间不直接互相调用——通过触发新的 Trigger 传递。

---

## 六、与当前 M2 的兼容

M2 落地后不删。Observer 版本可以**渐进替换**：

| 步骤 | 内容 |
|------|------|
| 1 | 加 `OnMove` Trigger + HuntObserver（替换 `tick_predator` 中移动触发部分） |
| 2 | 验证：狼行为不变，断言不降 |
| 3 | 逐步将其他行为迁移到 Observer |
| 4 | 最后移除 bucket tick 主循环 |

每步可独立验收、回退。

---

## 七、性能预期

| 场景 | M2 轮询 | Observer |
|------|--------|---------|
| 20 只动物 | ~0.002ms | ~0.001ms |
| 200 只动物 | ~0.02ms | ~0.005ms |
| 2000 只动物 | ~0.2ms | ~0.01ms（只有移动的才触发） |
| 2000 只 + 商业规则 | ~0.5ms | ~0.05ms（规则只在触发时跑） |

---

## 八、何时做

**M2 落地后。** Observer 不改行为逻辑——M2 的行为实现是 Observer 的"被调度内容"。先让 M2 完整跑通，再用 Observer 换调度层。

---

## 九、约束

- `card_defs.ron` 格式不改
- 行为逻辑不改（每个 Observer 调 M2 已有函数）
- 断言数不降
- Bevy 版本锁 0.14
