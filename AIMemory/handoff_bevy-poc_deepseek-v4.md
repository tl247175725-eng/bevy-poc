# Rust + Bevy 技术 POC

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Type**: POC 执行单，非正式迁移
**Godot 基准**: GDScript 642ms → C# Mono 295ms。F5 实感 PPT。判定 Godot 走不通。

---

## POC 目标

验证 Rust + Bevy 是否能在同等生态逻辑下跑出 10×+ 于 Godot C# 的 tick 性能。同时验证 Cursor + deepseek-v4 的协作流能否在 Rust 生态中正常工作。

---

## 一、POC 范围

```
bevy 0.14（锁定版本）
├── 36×24 格网渲染（色块，无精灵图）
├── CardDef 从 .ron 文件加载
├── SpatialIndex（tag + cell 双索引）
├── 生态逻辑：羊找草→吃 / 狼找羊→猎
├── 20 条精选断言（cargo test）
└── headless 跑 30 秒，输出 avg_tick_ms
```

### 不做的

- UI（侧栏、拖卡、信息面板）
- 玩家卡 / 玩家大脑
- 商业系统
- 精灵图 / 动画
- Godot 代码删除

---

## 二、项目结构

```
bevy-poc/
├── Cargo.toml                 # bevy = "0.14", ron = "..."
├── assets/
│   └── card_defs.ron           # 草、羊、狼 3 种卡定义
├── src/
│   ├── main.rs                 # 入口，窗口+格网渲染
│   ├── card_def.rs             # CardDef 结构体，从 .ron 加载
│   ├── spatial_index.rs        # by_tag + by_cell 双索引
│   ├── world_rules.rs          # 标签判定（flocking, predator 等）
│   ├── systems/
│   │   ├── tick_herbivore.rs   # 羊 tick：找草→移动→吃
│   │   ├── tick_predator.rs    # 狼 tick：找羊→移动→猎
│   │   └── grass_regen.rs      # 草皮再生
│   ├── grid_render.rs          # 格网渲染（色块）
│   └── bench.rs                # headless tick 基准测试
└── tests/
    └── assertions.rs           # 20 条核心断言
```

---

## 三、CardDef .ron 格式

```ron
[
    CardDef(
        type_name: "grass",
        display_name: "草皮",
        tags: ["cover", "food_source"],
        hp: 0,
    ),
    CardDef(
        type_name: "sheep",
        display_name: "羊",
        tags: ["being", "animal", "herbivore", "flocking"],
        hp: 2,
    ),
    CardDef(
        type_name: "wolf",
        display_name: "狼",
        tags: ["being", "animal", "predator", "pack_hunter"],
        hp: 4,
    ),
]
```

**原则**：加新卡 = 加一条 RON。不加 Rust 代码。

---

## 四、20 条核心断言

从 Godot 993 条 L0 中精选，覆盖生态闭环的核心行为：

| # | 断言 | 对应 Godot L0 |
|----|------|-------------|
| 1 | 草皮在非河岸格不自动再生 | grass 再生逻辑 |
| 2 | 羊有 flocking 标签时，成羊 < 3 不繁殖 | flocking_blocks_reproduction |
| 3 | 狼有 pack_hunter 标签时，单狼只猎 small_prey | _pack_hunter_under_strength |
| 4 | 草被羊吃后消失 | sheep 吃草 |
| 5 | 羊饱腹后不再吃草 | mark_ecology_fed |
| 6 | 狼在 hunt_range 内找到羊时发起捕猎 | best_hunt_target |
| 7 | 狼成功猎羊后羊变成尸体 | hunt 产出 |
| 8 | 羊看到狼时（fear_range 内）逃跑 | _flee_from_threat |
| 9 | 草在河岸格按 regen_interval 再生 | grass regen |
| 10 | SpatialIndex query_tag("grass") 返回正确数量 | 索引正确性 |
| 11 | SpatialIndex query_near(x,y,"sheep",3) 返回范围内羊 | 空间索引 |
| 12 | 羊移动后 SpatialIndex 更新位置 | 索引增量更新 |
| 13 | card_has_tag 对 flocking 羊返回 true | 标签判定 |
| 14 | 狼不在 hunt_range 内时不发起捕猎 | 距离判定 |
| 15 | 草的 hp 为 0 时被移除 | 资源消耗 |
| 16 | 羊在无草可吃时进入 idle | 饥饿状态 |
| 17 | 狼在无猎物时进入巡逻 | 空闲行为 |
| 18 | 草不会被重复吃 | 互斥消费 |
| 19 | 同一格可以有草 + 羊共存（不同层） | 格规则 |
| 20 | tick 1000 次后生态系统不自毁 | 长期稳定性 |

---

## 五、验收门

| 指标 | 通过标准 | 测量方式 |
|------|---------|---------|
| tick 性能 | headless 30s，平均 **< 30ms/tick** | `cargo run --release -- --bench` |
| 断言 | 20 条全 PASS | `cargo test` |
| 协作流 | Cursor 收到 handoff 后 1 次会话内改 1 条规则并编译通过 | 实际测试 |
| 可运行 | `cargo run --release` 弹出窗口，显示格网 + 色块卡移动 | 人工确认 |

**10× 目标说明**：Godot C# headless 295ms。Bevy POC 目标 < 30ms（10×）。若 < 16ms（60fps tick）则属于超额达标。

若 POC 达标 **< 30ms** → 分期迁移：生态核 → 玩家 → UI。
若 POC 未达标 > 30ms → 排查原因，决定是否继续。

---

## 六、Godot 线处理

- Godot 项目**不删、不改**——作为设计基准和 993 断言的行为说明书
- POC 期间 Godot 线冻结，不开发新功能
- 新设计继续在 Godot 设计文档中描述，handoff 发到 Bevy 线执行

---

## 七、风险提醒

- Bevy 0.14 API 在社区仍迭代快，锁定版本至关重要
- Rust toolchain 安装是第一步（`rustup`），确认 Cursor 环境能跑 `cargo build`
- headless bench 和带窗口的渲染是两套编译——POC 先冲 headless 数据，渲染后验
