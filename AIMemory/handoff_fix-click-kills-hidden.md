# 左键点击不砸藏匿实体 — 交互安全修复

**Priority**: P0（点草丛兔子立即死亡，草卡无法互动）

## 根因

两条攻击路径都会杀死藏匿实体：

### 路径 1：detect_drag_smash（拖拽中每帧检测）
- 左键按下 → `resolve_selection_card` 返回草 → `can_drag_entity` true → 拖拽开始
- 同一帧 `detect_drag_smash` 检查拖拽卡与所有其他 CardVisual 的距离
- 若兔子的 CardVisual 仍在（或兔子不在 cover 而是与草同格共存，compose 允许 Low over Flat）→ 距离 < SMASH_CONTACT_DIST → SMASH!
- `apply_smash_hit` 未检查 `target.in_cover` → 兔子 HP 1 → 杀死 → 产尸

### 路径 2：find_impact_target → try_impact（释放时）
- `find_impact_target` 在格内 `entities_at` 中找任意非 source 实体 → 不排除 `in_cover`
- 释放左键时 `try_impact` → `apply_smash_hit` → 同样不检查 `in_cover`

### 选择卡片也暴露了隐患
- `resolve_selection_card` 不过滤 `in_cover` 实体（仅排序排最后）
- 两个同格实体同 sort key 时返回哪个取决于 HashMap 迭代序 → 不确定
- 可能返回兔子 → 拖拽兔子 → 砸草 → 同样异常

## 修复

### 修复 1：apply_smash_hit 拒绝 in_cover 目标（defense in depth）

**文件**: `src/interaction/smash.rs` → `apply_smash_hit`

在获取 tgt 后立即检查：
```rust
let Some(tgt) = world.entities.get(&target).cloned() else {
    return SmashOutcome::NoEffect;
};
// NEW:
if tgt.in_cover {
    return SmashOutcome::NoEffect;
}
```

一条检查覆盖两条攻击路径。

### 修复 2：resolve_selection_card 排除 in_cover 实体

**文件**: `src/selection_info.rs` → `resolve_selection_card`

filter 中增加 `&& !e.in_cover`：
```rust
.filter(|e| {
    e.x == x && e.y == y
        && !e.in_den && !e.in_burrow
        && !e.in_tree && !e.in_pool && !e.in_ground
        && !e.in_cover  // NEW
})
```

### 修复 3：detect_drag_smash 跳过 in_cover 的 CardVisual（防御纵深）

**文件**: `src/ui_interaction.rs` → `detect_drag_smash`

在碰撞检测循环中跳过 `in_cover` 实体：
```rust
for (cv, transform) in &cards {
    if cv.entity_id == source_id.0 { continue; }
    // NEW
    if sim.0.entities.get(&EntityId(cv.entity_id))
        .is_some_and(|e| e.in_cover) { continue; }
    // ...
}
```

### 修复 4（可选）：不让低矮地形卡被拖拽

`can_drag_entity` 中检查 `cell.overlay` 标签（草、灌木等），阻止拖拽：
```rust
if let Some(def) = world.card_defs.get(&entity.type_name) {
    if def.is_rooted { return false; }
    if card_has_tag(def, "cell.overlay") { return false; }  // NEW
}
```

## 智能验收

### 验收 A：点藏兔子的草丛，兔子不死亡
```
启动游戏 → 等兔子 hide 进草丛 → 左键点击草丛
断言: 兔子不死亡（HP 不变）
断言: 不出兔尸体
断言: 选中草卡，详情面板显示草
```

### 验收 B：藏匿实体不可选
```
左键点有藏兔的草丛 → 选中草卡（不选中兔子）
构造: 草丛上只有藏匿兔（无草）→ 点击应无选中或无操作
```

### 验收 C：拖拽不会误砸藏匿实体
```
拖一张卡经过藏兔的格子 → 不触发 smash 动画/伤害
```

### 验收 D：正常砸功能不受影响
```
拖一张卡砸可见的卡 → 正常造成 1 HP 伤害
狼打羊 → 正常狩猎（AI smash 不经过 UI，不受影响）
```

### 验收 E：编译+测试
```
cargo test -- --nocapture 全部 PASS
cargo build 成功
```

## 涉改文件

| 文件 | 修复 |
|---|---|
| `src/interaction/smash.rs` | 修复 1（apply_smash_hit 拒绝 in_cover） |
| `src/selection_info.rs` | 修复 2（resolve_selection_card 排除 in_cover） |
| `src/ui_interaction.rs` | 修复 3（detect_drag_smash 跳过 in_cover）+ 修复 4（cell.overlay 不可拖拽） |

## 设计文档引用

- `design_interaction-system_deepseek-v4.md` — 左键=选+砸，右键=叠+搬运
- `design_machine-readable_v4.md` §2.2 — 藏中实体距离>1不可感知（同理，藏中不可被砸）
