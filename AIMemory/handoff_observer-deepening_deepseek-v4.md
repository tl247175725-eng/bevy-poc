# Observer 深化：BehaviorRegistry 事件化

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-07
**Priority**: P0（优化地基）
**基线**: Observer B 两刀已落地。Light Rete RuleIndex 已落地。

---

## 目标

当前 `main_tick` 仍 bucket 遍历全部活性卡。Observer B 加了 OnMove 邻居通知但 bucket tick 没删。深化后 bucket tick 降级为安全网——绝大多数行为由事件链自动触发。

**原则**：安静卡不问，谁动问谁。

---

## 一、改动点

### 1.1 bucket tick 降级

`main_tick` 里 `for id in active { tick_entity(world, id, delta) }` ——不再逐卡调用 BehaviorRegistry。

改为：
```rust
// 只 tick 必须轮询的：捕食者 patrol、固定计时器
for id in world.entities_with_tag("predator") {
    tick_predator_patrol(world, id, delta);
}
// 环境/繁殖/腐坏 仍走 FixedTimestep（不变）
```

食草、食肉、覆盖觅食等行为全部由 Observer 事件驱动。

### 1.2 行为注册表改事件注册

`BehaviorRegistry::tick_card` 的大 switch 改为事件→函数的注册表：

```rust
struct EventRegistry {
    handlers: HashMap<SimEventType, Vec<EventHandler>>,
}

// 注册
registry.on(SimEventType::Move, |world, entity| {
    // 通知邻居 predator → 可能触发 hunt
    // 通知邻居 prey → 可能触发 flee
});
registry.on(SimEventType::Kill, |world, predator, prey| {
    // 生成尸体
    // 标记饱腹
    // 吸引陆虫
});
registry.on(SimEventType::Spawn, |world, entity| {
    // 空间索引注册
});
```

### 1.3 保留的 FixedTimestep tick

这些没有触发源，必须定时跑：

| 系统 | 原因 |
|------|------|
| 草皮再生 | 环境计时 |
| 腐坏倒计时 | 纯时间驱动 |
| 腐解计时 | 纯时间驱动 |
| 繁殖冷却 | 周期性检查 |
| 捕食者 patrol | 狼不动时也要能"闻"到附近的羊 |
| 岩壁风化 | 纯时间驱动 |
| 山药蔓延 | 纯时间驱动 |
| 水藻再生 | 环境计时 |

---

## 二、事件链示例

```
狼移动到新位置
  → SimEvent::Move(wolf, old_pos, new_pos)
  → EventRegistry.on_move → 查 SpatialIndex: "5 格内有 wildPrey？"
  → 有 → on_hunt_trigger(wolf, prey)
       → wolf tick → 判定 → 移动逼近 → 产生 OnMove
  → 狼抵达猎物邻格 → on_kill(wolf, prey)
       → 生成 corpse
       → on_corpse_spawn → 吸引 landBug
       → landBug OnMove → 附近 fieldMouse 触发 Forage
```

一条 OnMove 触发整条因果链，不需要 bucket tick 逐卡问。

---

## 三、与 Rete RuleIndex 配合

Rete 的 `tag_index` 已经能从"标签变了"跳到"哪些规则要看"。Observer 深化后：

```
卡标签变更 → RuleIndex.tag_index["predator"] → 相关规则
卡移动     → SimObserver.on_move → 通知邻居
卡死亡     → SimObserver.on_kill → 生成尸体 + 食物网通知
```

两者各司其职。Rete 管"哪些规则受标签影响"，Observer 管"哪些事件触发了行为链"。

---

## 四、涉及文件

| 文件 | 改动 |
|------|------|
| `src/main_tick.rs` | bucket tick 降级，只跑 patrol + 环境 |
| `src/sim_observer.rs` | 加 EventRegistry 注册表 |
| `src/systems/tick_predator.rs` | 拆出 `tick_predator_patrol` 独立函数 |
| `src/systems/tick_herbivore.rs` | graze 从 bucket 移到 OnMove 触发 |
| `src/systems/tick_cover_forager.rs` | forage 从 bucket 移到事件触发 |

---

## 五、验收

- `cargo test`：120+ 断言不降
- `cargo run --release`：生态行为一致，羊吃草狼猎鹿正常
- bench tick 降低（安静时近乎零）
- assert_20 1000 tick 稳定性不退化

## 约束

- 环境/繁殖/腐坏仍走 FixedTimestep
- 捕食者 patrol 仍每 tick
- 不碰 M1-M4 数据层和渲染层
- 记 FIX_LOG
