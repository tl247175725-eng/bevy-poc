# 生态地基全量落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: HIGH
**设计依据**: `docs/design/card-reality-audit.md`、`docs/design/human-card-intelligence.md`

---

## 铁律

每行决策从标签推导，零魔法数字。新增可供性 = 加注册表条目，不改函数。

---

## 一、草上限

`game_state.gd`: `LIVING_GRASS_CAP` 12 → **30**

88 格河岸，恢复速度不是瓶颈，卡在总草量。30 格草 = 峰值 10 张嘴同时吃还剩 20 格缓冲。

---

## 二、初始种群调整

`world_manager.gd` `_spawn_initial_cards`:

| 物种 | 改前 | 改后 |
|------|------|------|
| 羊 | 公母各 1 | 公 2 母 3 |
| 鹿 | 公母各 1 | 公 2 母 3 |
| 水牛 | 公母各 1 | 公 1 母 2 |
| 兔 | 3 只 | 6 只 |
| 田鼠 | 4 只+幼鼠若干 | 8 只+幼鼠 4 |
| 狐 | 公母各 1 | 不变 |
| 狼 | 公母各 1 | 不变 |

---

## 三、新标签注册

`card_db.gd` 在对应卡上追加：

| 卡 | 追加标签 |
|----|---------|
| sheep | `flocking` |
| wolf | `pack_hunter` |
| rabbit | `prolific` |
| fieldMouse | `burrower` |
| sheepMeat/rabbitMeat/deerMeat/wolfMeat/humanMeat | `perishable` |

---

## 四、标签行为落地

### 4.1 flocking（羊）
同类成年羊 < 3 → 繁殖条件不满足。检查位置：`population_manager` 羊繁殖判定。

### 4.2 pack_hunter（狼）
同类成年狼 < 2 → `best_hunt_target` 排除 `largePrey`（鹿/羊/水牛幼崽），仅保留 `smallPrey` + `smallHerbivore`。检查位置：`world_rules.gd` `is_hunt_target_for` 的 `pack_predator` 分支。

### 4.3 prolific（兔）
公母配对 → 繁殖间隔 = 羊的 1/4，每胎 3 只。检查位置：`population_manager` 新增兔繁殖逻辑。兔本身不迁入。

### 4.4 burrower（田鼠）
威胁临近 → 进入 underground 状态（不可猎、不可见）。检查位置：`field_mouse_behavior.gd` 威胁判定后新增 `enter_burrow` 路径。

### 4.5 perishable（生肉）
生肉离体后 N tick 自毁。检查位置：`game_state.gd` 新增 `PERISHABLE_TICKS` 常量，`environment_manager` 或专用 tick 倒计时到零后 remove。

---

## 五、水生态

### 5.1 水潭地形
暗河源头改为**暗河源头水潭**。水潭格 `base_cell_type = "pool"`，`tags = ["water", "still_water"]`。位于地图西侧高点，面积约 10-15 格。河流保持现有 bank/river/ford 分类。

### 5.2 水生卡
`card_db.gd` 新增：

| type_name | 标签 | 说明 |
|-----------|------|------|
| algae | `aquatic`, `primary_producer`, `organize.locked` | 附着水格，自然恢复。被虫和贝吃 |
| waterBug | `tiny`, `aquatic`, `grazer`, `volant` | 吃藻。被鱼吃。可飞行迁入 |
| fish | `aquatic`, `small`, `migratory` | 吃虫。被鸟和人捕。经暗河迁入 |
| shellfish | `aquatic`, `filter_feeder`, `sessile` | 滤水吃藻。被鸟和人采。不动 |

### 5.3 水生态行为
- algae：每个水格自动恢复，类似草皮。被消耗后 N 秒再生。
- waterBug：有藻则出现。藻缺则减少。可迁入。
- fish：有虫则出现。虫少则鱼少。经暗河入口迁入。
- shellfish：有藻则附着。滤水稳定藻的恢复速度。
- 人类捕鱼：走到水格→消耗一次捕猎机会（替代猎兔），产出鱼肉（等同兔肉的热量）。

`CARD_CAPABILITIES` 同步补全。

---

## 六、涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/core/game_state.gd` | LIVING_GRASS_CAP 30, PERISHABLE_TICKS |
| `scripts/world/world_manager.gd` | 初始种群数量 + 水生态 spawn |
| `scripts/cards/card_db.gd` | 新标签 + 水生卡注册 |
| `scripts/core/world_rules.gd` | CARD_CAPABILITIES 补全, pack_hunter 逻辑 |
| `scripts/world/population_manager.gd` | flocking/prolific 繁殖逻辑, 兔繁殖 |
| `scripts/world/behaviors/field_mouse_behavior.gd` | burrower 掘穴 |
| `scripts/world/behaviors/herbivore_grazer_behavior.gd` | 逃跑判定不豁免叼肉狼 |
| `scripts/world/environment_manager.gd` | perishable 倒计时 |
| `scripts/core/world_rules/world_rules_terrain.gd` | 水潭地形 + algae 恢复 |

## 约束

- L0 断言数不降
- 不碰管线/封闭模式/ecosystem_behavior_key
- 记 fix-log
