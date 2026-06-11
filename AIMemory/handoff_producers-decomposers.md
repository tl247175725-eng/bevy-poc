# 生产者分层 + 分解者系统

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P1

**参考设计**: `AIMemory/design_human-readable_v4.md` 9.3-9.4

## 架构计划
标签驱动。生产者按土壤标签在对应地块自动生成。分解者用现有 in_ground 容纳体系。动物排便生成粪便卡。全走公理层。

## 架构反馈
复用现有 spawn 逻辑 + 地块标签（soil:*）。

## 智能验收
- 湿地草在 soil:wet 格生成
- 旱地草在 soil:dry 格生成
- 动物排便 → 粪便卡 → 粪虫分解 → 土壤肥力+1
- smoke test PASS

---

## P0-1：生产者分层

### 生成规则

在 `tick_environment` 或新建 `tick_producer_spawn` 中：

```
条件：格为空 + 符合土壤标签 + 随机概率

soil:rich, fertility:high  → 湿地草（grass, 70%概率）、菌菇（mushroom, 10%概率）
soil:wet, fertility:high   → 湿地草（60%概率）
soil:loose, shaded         → 菌菇（30%概率）、块茎（tuber, 10%概率）
soil:dry, fertility:low    → 旱地草（dry_grass, 30%概率）、蕨类（fern, 10%概率）
soil:deep, fertility:high  → 山药（wildYam, 20%概率，已有）、根茎（root, 10%）
soil:rocky                 → 不生成
```

### 新增植物卡

| 类型 | 标签 | HP | 食物值 |
|---|---|---|---|
| 旱地草 | `foodSource, arid` | 2 | 1 |
| 菌菇 | `foodSource, fungi` | 3 | 2 |
| 块茎 | `foodSource, tuber, underground` | 3 | 2 |
| 蕨类 | `foodSource, fern` | 1 | 1 |

### 产能

```
湿地草：高产（每 10 tick +1 HP，上限 4）
旱地草：中产（每 20 tick +1 HP，上限 2）
菌菇：中产（每 15 tick +1 HP，上限 3）
蕨类：低产（每 30 tick +1 HP，上限 1）
```

---

## P0-2：分解者 → 粪便循环

### 动物排便

```
每只动物 fed_today == true 时 → 在该格生成粪便卡（dung）
粪便卡：HP=2，标签 perishable, fertilizer
```

### 分解者

```
粪虫（dung_beetle）：in_ground，在粪便所在格自动生成
蚯蚓（earthworm）：in_ground，在 soil:rich 格自动生成
```

分解者从粪便/腐物中获取养分 → 提升土壤肥力：

```
粪便被分解者处理 → 粪便 HP-1 → HP=0 移除
粪便消失时 → 所在格 fertility 标签从 low→normal 或 normal→high（有上限）
```

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_producer_spawn.rs` | 新建：根据土壤标签生成植物 |
| `src/systems/tick_environment.rs` | 动物排便 + 分解者消费粪便 |
| `assets/card_defs.ron` | 新增旱地草/菌菇/块茎/蕨类/粪便/粪虫/蚯蚓 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS + smoke PASS
- 不同地块生不同植物
- 粪便→分解→肥力提升
