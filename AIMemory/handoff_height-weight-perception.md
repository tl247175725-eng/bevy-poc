# 高度/重量/感知域 — 三维标签系统

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（公理层输入参数完整化）

## 架构计划
三个新维度全部标签驱动，修改 compose/perceive/smash 的输入参数，不引入新系统。

## 架构反馈
公理层的 compose（高度）、perceive（感知）、smash（重量）从硬编码参数改为读取标签。这是架构完善，不是打补丁。

## 智能验收
- `height:flat` 卡不挡路
- `weight:feather` 砸不死动物
- `perception:dull` 反应范围缩小

---

## P0：高度标签

| 标签 | 值 | 阻挡 | 可挤开 | 例 |
|---|---|---|---|---|
| `height:flat` | 0 | 不阻 | — | 草、水 |
| `height:low` | 1 | 不阻 | — | 灌木、石头 |
| `height:medium` | 2 | 阻 | 生物可挤 | 羊、狼、人 |
| `height:high` | 3 | 阻 | 不可挤 | 山、大树 |

### compose 修改
`height:flat` 和 `height:low` 的卡不增加 `living_count` → 不参与一格一卡限制。只有 `medium+` 才算占用。

### movement 修改
`try_shove` / `try_yield` 检查 `height:high` → 拒绝挤开。

---

## P0：重量标签

| 标签 | 值 | 砸伤害 | 可搬运 | 例 |
|---|---|---|---|---|
| `weight:feather` | 0 | 0 | ✓ | 草、种 |
| `weight:light` | 1 | 1 | ✓ | 小石、枝 |
| `weight:medium` | 2 | 2 | ✓ | 工具、肉 |
| `weight:heavy` | 3 | 3 | ✗ | 大石 |
| `weight:immovable` | 4 | — | ✗ | 山 |

### smash 修改
`apply_smash_hit` 读取 `weight` 计算伤害。`feather(0)` 砸生物 → 伤害 0。

---

## P0：感知域标签

| 标签 | 范围修正 | 反应 | 例 |
|---|---|---|---|
| `perception:keen` | ×1.5 | 快 | 狐、鹰 |
| `perception:normal` | ×1.0 | 中 | 羊、狼 |
| `perception:dull` | ×0.5 | 慢 | 牛、龟 |
| `perception:nocturnal` | 夜间×2 | 夜行 | 狐、鼠 |

### perceive 修改
已有 `parse_channels`。在已有 range 上乘修正系数。`dull` 动物 dodge 频率降低（每 2 tick 才尝试一次 substitute move）。

---

## 卡牌标签分配

所有卡牌按上述表加 `height:N` 和 `weight:N` 标签。生物默认 `height:medium` + `weight:medium`。草 `height:flat, weight:feather`。山 `height:high, weight:immovable`。

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/axioms/laws.rs` | compose 按 height 区分占格 |
| `src/axioms/profile.rs` | parse_height / parse_weight / parse_perception |
| `src/interaction/smash.rs` | 伤害读 weight |
| `src/systems/movement.rs` | height:high 不挤 |
| `src/systems/tick_reactive.rs` | dull 降低 dodge 频率 |
| `assets/card_defs.ron` | 全卡加 height + weight 标签 |
| `AIMemory/design_ecology-chase-sprint_deepseek-v4.md` | 更新 |
