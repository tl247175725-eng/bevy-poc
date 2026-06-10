# 生态/冲刺/隐藏 — 批量落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（设计已确认，直接执行）

## 架构计划
全部通过标签+现有引擎落地，不引入新系统。肉量=HP 标签、冲刺是已有标签扩展、路径逃跑复用 `find_path`、丛类卡隐藏复用现有 `drive:hide` 逻辑。

## 架构反馈
无。标签数据配置为主，代码改动仅限 `flee_from` → `flee_pathfind` 和隐藏视觉。

## 智能验收
- 野牛 HP=8 肉=8，狼 HP=4 肉=4
- 所有生物有 sprint 标签
- 被追猎时不再卡角落
- 灌木卡可见（已修）
- 草/灌木藏动物时显示紫色"藏"圆

## 设计文档
`AIMemory/design_ecology-chase-sprint_deepseek-v4.md`

---

## P0：HP 和肉量更新

`assets/card_defs.ron`，按表修改所有动物的 `hp` 字段 + 新增 `meat_yield:N` 标签：

```
sheep: hp=3
deer: hp=3  
wolf: hp=4
waterBuffalo: hp=8
fox: hp=2
pheasant: hp=2
rabbit: hp=1
fieldMouse: hp=1
fieldMousePup: hp=1
bambooRat: hp=1
fish: hp=1
waterBug: hp=1
landBug: hp=1
```

肉量标签 `meat_yield:N` 加在每种动物上（昆虫除外）：
```
sheep → meat_yield:3
wolf → meat_yield:4
waterBuffalo → meat_yield:8
...
```

`hunt_kill` 中读取 `meat_yield` 标签，生成对应数量的肉卡。若无标签则默认 1。

---

## P0：冲刺标签

所有动物加 sprint 标签：

```
sheep, deer, waterBuffalo: sprint:slow
wolf: sprint:normal
fox, pheasant, fieldMouse, rabbit, bamboRat: sprint:fast
rabbit: sprint:burst (覆盖 sprint:fast — 兔是爆发型)
玩家 (player): sprint:endurance
```

冲刺特效：**残影拖尾 + 尘土粒子**

### 残影实现
`sprint_visual` 系统：检测移动速度 > base 时在实体位置孵化 2-3 个半透明 sprite 副本，每个副本 alpha 从 0.5 渐隐到 0，持续 0.2s 后 despawn。

### 尘土实现
同样的系统，在起步时（第一帧进入 sprint 状态）在脚下 spawn 一个小圆圈 sprite，scale 从 0.5 扩到 1.5，alpha 从 0.6 渐隐到 0，持续 0.3s 后 despawn。

新增 `SprintTrail` 和 `SprintDust` 组件标记。

---

## P0：路径逃跑

`flee_from` 改为 `flee_pathfind`：被追猎锁定时（predator 在相邻格且有 hunting 状态），不用单步反方向，调用 `find_path` 找远离 predator 的最远可到达格，沿路径走第一步。

```rust
// 在 tick_reactive 的 Flee 驱动中
if is_hunter_nearby(world, id, predator_id) {
    // 路径逃跑
    let safe_pos = find_path_away_from(world, x, y, predator_x, predator_y, 5);
    if let Some((nx, ny)) = safe_pos.first() {
        move_toward(world, id, x, y, nx, ny);
    }
} else {
    // 普通警觉
    flee_from(world, id, x, y, tx, ty);
}
```

---

## P0：腐殖土长草

`humus_age` 超时 → 在该格 spawn "grass"。清除此格 humus 记录。

```rust
// tick_environment 中
if age > HUMUS_GRASS_SECONDS {
    world.spawn("grass", x, y);
    world.humus_layers.remove(&key);
    world.humus_age.remove(&key);
}
```

---

## P0：草内虫类容纳

草卡新增 `fauna_worm: 0` 和 `fauna_beetle: 0` 字段（或 label-based 计数）。草生成时初始随机 0-3 条虫。

鼠/鸟的 `drive:seek(target=grass_fauna)` 标签：seek 到达草的相邻格 → 消耗草内容纳计数 → mark fed。

点击草 → 面板显示"蚯蚓×N 甲虫×M"。

---

## P1：丛类卡隐藏视觉

草和灌木有 `cover.small` 标签。当 `drive:hide` 触发后，卡牌右上角渲染**紫色实心圆 + 白"藏"字**，圆直径 = CARD_SIZE / 2。

新增 `HideBadge` 组件，在 `sync_card_visuals` 或 `hide_visuals` 系统中检测 `hidden_in_grass || in_burrow` 状态显示紫色圆形 badge。

## 涉及文件

| 文件 | 改动 |
|---|---|
| `assets/card_defs.ron` | HP + meat_yield + sprint 标签 |
| `src/systems/tick_reactive.rs` | flee → find_path 路径逃跑 |
| `src/systems/tick_environment.rs` | humus → grass |
| `src/card_visual.rs` | 残影/尘土/藏 badge 视觉 |
| `src/world_state.rs` | 草卡 fauna 计数（或直接复用 bush_microfauna） |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS + `-- --smoke-test` PASS
- 灌木可见
- 被追不再卡死角落
- 冲刺有残影拖尾
- 草中藏动物有紫色"藏"圆
