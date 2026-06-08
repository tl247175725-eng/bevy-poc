# 增量迭代前清理：card_type 残余 + emits + 空壳删除

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-02

---

## 一、清 card_type 残余

以下 `is_*` 函数改为 tag/capability 检查。每个给出精确改法。

### 1. `world_rules.gd` — `is_bush`
**现状**：`card.card_type == "bush"`
**改为**：`card_has_tag(card, "bush")`

### 2. `world_rules.gd` — `is_field_mouse`
**现状**：`card.card_type in ["fieldMouse", "fieldMousePup"]`
**改为**：`card_has_tag(card, "smallPrey") and card_has_tag(card, "cover_user")`

### 3. `world_rules.gd` — `is_field_mouse_adult`
**现状**：`card.card_type == "fieldMouse"`
**改为**：`is_field_mouse(card) and can_reproduce(card)`

### 4. `world_rules.gd` — `is_field_mouse_pup`
**现状**：`card.card_type == "fieldMousePup"`
**改为**：`is_field_mouse(card) and can_grow(card)`

### 5. `world_rules.gd` — `is_adult_fox`
**现状**：`card.card_type == "fox"`
**改为**：`card_has_tag(card, "mesopredator") and can_reproduce(card)`

### 6. `world_rules.gd` — `is_fox_cub`
**现状**：`card.card_type == "foxCub"`
**改为**：`card_has_tag(card, "mesopredator") and can_grow(card)`

### 7. `world_rules.gd` — `is_fox_den`
**现状**：`card.card_type == "foxDen"`
**改为**：`card_has_tag(card, "den.candidate.fox")`

### 8. `world_rules.gd` — `fox_dens`
**现状**：遍历 cards 检查 `card_type == "foxDen"`
**改为**：遍历 cards 检查 `card_has_tag(card, "den.candidate.fox")`

### 9. `world_rules.gd` — `is_sheep_lamb`
**现状**：`card.card_type == "lamb"`
**改为**：`card_has_tag(card, "juvenile") and card_has_capability(card, "capability.be_cared_for") and not card_has_capability(card, "capability.reproduce")`

### 10. `world_rules.gd` — `is_deer_fawn`
**现状**：`card.card_type == "deerFawn"`
**改为**：`card_has_tag(card, "juvenile") and card_has_tag(card, "herbivore") and card_has_capability(card, "capability.be_cared_for")`

### 11. `world_rules.gd` — `is_adult_deer`
**现状**：`card.card_type == "deer"`
**改为**：`card_has_tag(card, "herbivore") and can_reproduce(card) and can_forage(card)`

### 12. `world_rules_camp.gd` L36 — `type_name == "wolfDen"`
**现状**：字符串硬编码 wolfDen
**改为**：`card_has_tag(type_name, "den") and card_has_tag(type_name, "animalHome")`

### 13. `card_base.gd` L44 — `card_type in ["wolf", "wolfCub", "fox", "foxCub"]`
**现状**：视觉可见性用 card_type 列表
**改为**：`card_has_capability(card, "capability.return_home")`

---

## 二、补 emits 事件链

在 `world_rules.gd` 顶部常量区加一个字典，列出所有跨文件副作用事件：

```gdscript
const RULE_EMITS = {
    "try_hunt_attack": {
        "emits": ["card_damaged", "card_died"],
        "crosses": ["interaction_manager.gd"]
    },
    "_on_card_removed": {
        "emits": ["corpse_spawned"],
        "crosses": ["interaction_manager.gd", "ecosystem_manager.gd"]
    },
    "corpse_decay": {
        "emits": ["humus_formed"],
        "crosses": ["ecosystem_manager.gd"]
    },
    "humus_formed": {
        "emits": ["grass_regen_accelerated"],
        "crosses": ["environment_manager.gd"]
    },
    "mark_ecology_fed": {
        "emits": ["predator_fed"],
        "crosses": ["population_manager.gd"]
    },
    "_on_ecology_feed_new_day": {
        "emits": ["starvation_check", "card_died"],
        "crosses": ["population_manager.gd"]
    },
    "fox_den_built": {
        "emits": ["bush_removed", "den_created"],
        "crosses": ["ecosystem_manager.gd"]
    },
    "wolf_den_built": {
        "emits": ["den_created"],
        "crosses": ["ecosystem_manager.gd"]
    },
    "grass_consumed": {
        "emits": ["grass_regen_triggered"],
        "crosses": ["environment_manager.gd"]
    },
    "configure_cell_overlay": {
        "emits": ["cell_state_changed"],
        "crosses": ["world_manager.gd"]
    },
}
```

不参与运行时逻辑，仅供静态查询。有遗漏的事件你自己补。

---

## 三、删除旧 behavior 空壳

删除以下五个文件：
- `scripts/world/behaviors/wolf_behavior.gd`
- `scripts/world/behaviors/fox_behavior.gd`
- `scripts/world/behaviors/rabbit_behavior.gd`
- `scripts/world/behaviors/deer_behavior.gd`
- `scripts/world/behaviors/sheep_behavior.gd`

然后修改所有引用方，直接引用目标文件：
- `WolfBehavior` → `PredatorDenBehavior`
- `FoxBehavior` → `PredatorDenBehavior`
- `RabbitBehavior` → `HerbivoreGrazerBehavior`
- `DeerBehavior` → `HerbivoreGrazerBehavior`
- `SheepBehavior` → `HerbivoreGrazerBehavior`

搜索 `scripts/` 下所有 `preload("res://scripts/world/behaviors/wolf_behavior.gd")` 等引用，替换为目标 preload。

---

## 约束

- L0 断言数不降
- 不碰管线/签名/封闭模式
- 记 fix-log
