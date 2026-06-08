# 狐狸行为重写：复制狼模式

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: HIGH

## 策略

四轮修复都没让狐狸动起来。不再逐行追 bug。把狼的行为模式复制给狐狸，只加三个狐狸专属差异。

## 改动

### 1. FoxCard.from_card → 完全同 WolfCard.from_card

```gdscript
static func from_card(card: CardBase):
    if card == null or card.card_type not in ["fox", "foxCub"]:
        return null
    return card
```

### 2. fox_behavior.gd 的 tick → 结构完全同 wolf_behavior.gd

狼 tick 的结构：
```
进 tick → 眩晕检查 → 幼体分流 → 类型守卫(from_card) → 窝内休整 → 
躲人 → 惧火 → 躲狼 → 叼肉回窝 → 筑巢判定 → 消化冷却 → 
移动 tick → 找猎物 → 攻击/靠近 → 随机步
```

狐狸 tick 应完全复制这个结构，差异仅在：
- **筑窝**：不是叼草到空地，而是占灌木转化（已有 `update_fox_build_den`）
- **猎物**：田鼠 > 兔 > 尸体（已有的 `best_hunt_target`）
- **清道夫**：偷尸体肉（已有 `try_fox_scavenge`）

### 3. 关键：from_card 失败时必须回退

如果 `_FoxCard.from_card(actor)` 返回 null 但 card_type 是 fox/foxCub，用原始 actor 继续 tick——不 return。

## 验收

- 报告 §7 应有诊断记录
- 狐狸应移动、捕猎或筑窝

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

- `fox_behavior.gd` 按 `wolf_behavior` 重排：幼体→from_card 回退→眩晕→窝内消化→consume_move_tick→叼肉→躲人/惧火/躲狼→筑灌丛窝→冷却→清腐→捕猎。
- `FoxCard.from_card` 同 WolfCard。差异仅：update_fox_build_den、best_hunt_target、try_fox_scavenge。
