# 提纯 Phase 3：RuleIndex 行为匹配纯标签化 + 删除 legacy

**Priority**: P0 — Phase 2 激活了 RuleIndex，Phase 3 清理它内部的 type_name 硬编码

## 架构计划

Phase 2 让 RuleIndex 的判决生效了。但 RuleIndex 内部还有 type_name 硬编码——不清理掉，下次加新卡还是绕不过去。Phase 3 做三件事：

### 1. `behavior_rule_matches` 全部改成纯标签

**文件**: `src/rule_index.rs` lines 425-447

当前有两个 type_name 硬编码：

| 行为键 | 当前（type_name） | 改为（纯标签） |
|---|---|---|
| `BEHAVIOR_PREDATOR_DEN` | `matches!("wolf"/"wolfCub"/"fox"/"foxCub")` | `is_predator(def) && card_has_capability(def, "capability.hunt")` |
| `BEHAVIOR_COVER_FORAGER` | `matches!("fieldMouse"/"fieldMousePup"/"bambooRat")` | `card_has_tag(def, "cover_user") \|\| card_has_tag(def, "burrower")` |

同时确保 card_defs.ron 中那些依赖 type_name 匹配的卡有正确的标签：
- `wolf`/`fox` 已有 `predator` + `capability.hunt` → 不需要改卡
- `fieldMouse`/`bambooRat` 已有 `cover_user` 或 `burrower` 标签 → 不需要改卡

### 2. 删除 `def_tags` 中的 type_name 注入

**文件**: `src/rule_index.rs` line 360-364

```rust
// ❌ 当前
fn def_tags(def: &CardDef) -> Vec<String> {
    let mut tags: Vec<String> = def.tags.clone();
    tags.push(def.type_name.clone());  // ← 删除这行
    tags
}

// ✅ 改为
fn def_tags(def: &CardDef) -> &[String] {
    &def.tags
}
```

type_name 不应该被当作隐式 tag。标签驱动 = 只有显式标签才影响行为。如果某张卡需要某行为但缺标签，应该给卡加标签——而不是让 type_name 充当标签的替代品。

### 3. 删除所有 legacy 和 dual_track 函数

**文件**: `src/rule_index.rs`

Phase 2 之后以下全部是死代码，安全删除：

**删除的函数**（rule_index.rs）:
- `legacy_should_hunt`（lines 451-463）
- `legacy_should_stalk`（lines 465+）
- `legacy_should_flee_if_alone` 
- `legacy_should_graze`
- `legacy_should_eat`
- `dual_track_action`（lines 512-525）
- `dual_track_hunt`（lines 527+）
- `dual_track_stalk`
- `dual_track_flee_if_alone`
- `dual_track_graze`
- `dual_track_eat`

**删除调用点**（tick_reactive.rs）:
- 所有 `dual_track_hunt(...)`, `dual_track_graze(...)` 等调用
- 替换为直接调用 `rule_index.evaluate_action(world, actor_id, action)`

**删除的辅助**（world_rules.rs）:
- `ecosystem_behavior_key_legacy` 函数（30行 type_name match 树）
- `mesopredator_diet_key`
- `herbivore_grazer_profile`（如果仅被 legacy 使用）
- `can_hunt_def` 中可能仅服务于 legacy 的分支

## 架构反馈

1. `def_tags` 注入 type_name 是最隐蔽的 type_name 硬编码——它让行为系统"看起来"是走标签，但 type_name 以标签身份混入了。删掉后，如果任何卡因为缺少标签而行为异常，那说明卡的定义不完整——应该补标签，不是补 type_name。

2. 删除 legacy 函数的过程中可能发现 `tick_reactive.rs` 中仍有调用。这些调用在 Phase 2 后已经没有实际效果（legacy 参数不被使用），但为了代码整洁必须清理。

3. 本次改动风险中等——RuleIndex 的标签匹配逻辑如果不够精确，可能导致某些卡的行为消失或改变。但这是必须承担的代价：不暴露问题就没法修复。

## 智能验收

### 验收 A: behavior_rule_matches 无 type_name
```
grep -n "type_name" src/rule_index.rs
断言: behavior_rule_matches 中无 type_name 字符串
断言: def_tags 中无 type_name 注入
```

### 验收 B: legacy 全部删除
```
grep -n "legacy\|dual_track" src/rule_index.rs src/systems/tick_reactive.rs
断言: 无结果（所有 legacy 和 dual_track 已删除）
```

### 验收 C: 所有调用点改为直接 RuleIndex 查询
```
grep -n "dual_track\|legacy_should" src/systems/tick_reactive.rs src/
断言: 无结果
grep -n "ecosystem_behavior_key_legacy" src/
断言: 无结果
```

### 验收 D: cargo test + smoke test
```
cargo test -- --nocapture 全部 PASS
cargo run -- --smoke-test 烟雾测试 PASS（0 移动或 0 狩猎视为失败）
```

### 验收 E: 卡定义无遗漏
```
检查 wolf/fox/wolfCub/foxCub 有 predator + capability.hunt 标签 → 断言通过
检查 fieldMouse/fieldMousePup/bambooRat 有 cover_user 或 burrower 标签 → 断言通过
```

## 涉改文件

| 文件 | 改动 |
|---|---|
| `src/rule_index.rs` | 1. behavior_rule_matches 纯标签化 2. 删 def_tags type_name 注入 3. 删 legacy + dual_track |
| `src/systems/tick_reactive.rs` | 替换 dual_track 调用为直接 RuleIndex 查询 |
| `src/world_rules.rs` | 删除 ecosystem_behavior_key_legacy 等仅服务于 legacy 的函数 |

## 设计文档引用

- `design-philosophy-v5.md` §5.1 "禁止 type_name 硬编码"
- `design_machine-readable_v4.md` §3 "无硬编码"
- `design-philosophy-v5.md` §2.3 "公理不判断意图"
