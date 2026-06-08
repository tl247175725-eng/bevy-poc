# 调度改造 Step 5 + 6：capability 签名分发 + 捕食者壳删除

**From**: cursor | **To**: cursor（自执行）  
**Date**: 2026-05-30 | **Parent**: proposition 6 步表 Step 5–6  
**前置**: Step4 宿主收敛 DONE（审查 PASS）

---

## 目标（一次 F5 验收）

1. **Step 5**：`ecosystem_behavior_key` 不再 `card_type` 查表，改为 capability 签名。
2. **Step 6**：`WolfBehavior` / `FoxBehavior` 空壳或删除，逻辑迁入 `PredatorDenBehavior`；兔/鹿/羊合并 `HerbivoreGrazerBehavior`。
3. **验收卡**：仅注册标签即可吸附——猞猁 `hunt+move_normal+use_cover`；野牛 `forage+move_slow+flee`。

---

## 自拆顺序

| 序 | 任务 | 文件 |
|----|------|------|
| 1 | 签名常量 + `ecosystem_behavior_key` + `herbivore_grazer_profile` | `world_rules.gd` |
| 2 | `PredatorDenBehavior`（幼体/窝内/狼迁出） | 新建 |
| 3 | `HerbivoreGrazerBehavior`（rabbit/deer/sheep/slow/juvenile） | 新建 |
| 4 | Registry 改 match；pipeline `applies_to` 改 key | `ecosystem_tick_registry.gd`, `capability_behavior_pipeline.gd` |
| 5 | Wolf/Fox/Rabbit/Deer/Sheep 空壳 forward | `*_behavior.gd` |
| 6 | card_db + CARD_CAPABILITIES：`lynx` / `bison` | `card_db.gd`, `card_rule_audit.gd` |
| 7 | 单测改 key + `_test_capability_signature_adsorption` | `unit_test_cases.gd` |
| 8 | [x] L0 → #回复 / CHANGELOG / fix-log / work.log |

---

## 签名表（匹配顺序）

| Key | Requires | Forbids | Registry |
|-----|----------|---------|----------|
| `traveler` / `mushroom_farmer` / `taoyuan` | 原社交判定 | — | 独立 behavior |
| `predator_den` | hunt, return_home, care_child | — | PredatorDen + Pipeline |
| `mesopredator_hunt` | hunt, use_cover | return_home | Pipeline only |
| `cover_forager` | forage, use_cover, escape_cover | — | FieldMouse |
| `herbivore_grazer` | forage+flee+be_hunted **或** grow+be_cared_for（幼体） | — | HerbivoreGrazer |

**Profile**（同 key 内）：`rabbit` | `deer` | `sheep` | `slow` | `juvenile`

---

## 不碰

- `game_ui` 大改
- 攻击 API `wolf_attack`/`fox_attack` 命名（可后续）
- 桃源/旅人/蘑菇农 tick 实现

---

## 验收

- L0 全绿
- `ecosystem_behavior_key(lynx)==mesopredator_hunt`，`bison==herbivore_grazer`（profile slow）
- `CapabilityBehaviorPipeline.applies_to(lynx)` 为真
- F5：狼狐兔鹿羊行为不退化；新卡仅加 card_db + CARD_CAPABILITIES

---

## #回复

**L0 PASS — 795 断言**（+16 vs Step4）

- `ecosystem_behavior_key` 全 capability 签名；Registry 5 类生态 key + 3 社交例外
- `PredatorDenBehavior` / `HerbivoreGrazerBehavior` 合入；Wolf/Fox/Rabbit/Deer/Sheep 空壳 forward
- 验收卡 `lynx`→`mesopredator_hunt`+管线；`bison`→`herbivore_grazer`（profile slow）
- 幼狼/狐仍 `predator_den`（域成员+grow/hunt 回退）

**F5 建议**：狼窝/狐筑窝/兔鹿羊吃草/猞猁游荡捕猎（若已放地图）一轮目视。
