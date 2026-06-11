# 腐败 + 疾病系统

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P1

**参考设计**: `AIMemory/design_human-readable_v4.md` 9.5-9.6

## 架构计划
腐败用标签驱动（fresh→early→advanced→severe→humus）。疾病用标签驱动（因果触发）。复用现有 tick_environment 和 tick_reactive。

## 架构反馈
Entity 已有 decay_timer。疾病在 Entity 上加 disease 标签。

## 智能验收
- 尸体从 fresh 推进到 severe → 变腐殖土
- 吃腐肉→中毒，受寒→发烧
- smoke test PASS

---

## P0-1：腐败四阶段

### 标签映射

```
decay:fresh    → 新鲜，正常状态
decay:early    → 变色，decay_timer > 阈值1
decay:advanced → 软化，decay_timer > 阈值2
decay:severe   → 濒崩，decay_timer > 阈值3 → 下次 tick 变腐殖土
```

### 腐败速度

```
基础速度：每 tick decay_timer += 1.0
湿地：×2.0（潮湿加速）
干燥：×0.5（旱地减速）
寒冷（冬季温度<0）：×0.0（冻结不腐败）
```

### 不同材料腐败时间

```
肉：新鲜→初期 30 tick，初期→中期 30 tick，中期→严重 30 tick（共 90 tick）
木：新鲜→初期 300 tick，→中期 300 tick，→严重 300 tick（共 900 tick）
骨：不腐败（除非有 special tag）
```

### 视觉

```
早期：卡牌颜色微变（RGB 略微偏移）
中期：颜色明显暗沉
严重：出现裂纹/破碎效果（sprite 闪烁或变灰）
```

---

## P0-2：疾病

### 触发因果

```
吃腐肉（decay:advanced+ 的肉被吃）→ eater 获得 food_poisoned
冬季未保暖（temperature < 0 且无 shelter）→ fever
被咬伤（predator 攻击且 prey 未即死）→ infected
```

### 疾病效果

```
food_poisoned：移动速度 ×0.5，每 tick 有 5% 概率跳过行动，持续 50 tick 后自愈
fever：需要更多休息（need:rest 衰减加速 ×3），寒冷耐受力下降
infected：HP 每日 -1，需要草药/自愈概率低
```

### 标签格式

```
food_poisoned    → flag，临时标签，50 tick 后移除
fever            → flag，临时标签，有 shelter + 温暖则 30 tick 后移除
infected         → flag，临时标签，5% 自愈概率/tick，否则 HP-1/day
```

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_environment.rs` | 腐败推进 + 疾病推进/自愈 |
| `src/card_visual.rs` | 腐败视觉（颜色偏移） |
| `src/systems/tick_reactive.rs` | 疾病影响行为 |
| `src/world_state.rs` | Entity 加 disease 标签临时字段 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS + smoke PASS
- 尸体逐渐变色→腐殖土
- 吃腐肉→减速+跳帧
