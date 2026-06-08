# Bevy 迁移 M1：全卡定义 + WorldRules 核心 + 颜色体系

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0
**基准**: Godot `card_db.gd` 全量 + `world_rules.gd` 核心判定 + `terrain_visual_palette.gd` + `card_base.gd` 颜色定义

---

## M1 目标

把 Godot 中所有卡牌定义、WorldRules 核心标签判定、格网颜色体系迁移到 Bevy 项目。不迁行为逻辑（M2 做），不迁 UI（M3 做）。

---

## 一、card_defs.ron — 全卡定义

`E:\桌面\bevy-poc\assets\card_defs.ron`，替换现有 3 卡版本。

**参照 Godot 文件**：`E:\桌面\方寸商国：桃花源记\scripts\cards\card_db.gd`

全部 50+ 种卡，每卡一条 RON 记录：

```ron
CardDef(
    type_name: "grass",
    display_name: "草皮",
    icon: "草",
    tags: ["cover", "food_source", "organize.locked"],
    color: (200, 242, 139, 255),  // RGBA
    hp: 0,
    is_rooted: true,
)
```

### 字段定义

| 字段 | 类型 | 来源 Godot |
|------|------|-----------|
| `type_name` | String | `_reg` 第一参数 |
| `display_name` | String | `_reg` 第三参数（中文） |
| `icon` | String | `_reg` 第二参数（单字） |
| `tags` | Vec<String> | `_reg` 第四参数 |
| `color` | (u8,u8,u8,u8) | `card_base._card_style` 配色 |
| `hp` | i32 | `_reg` 第五参数 base_hp |
| `is_rooted` | bool | `_reg` 第六参数 |

### 颜色来源

**参照 Godot 文件**：`E:\桌面\方寸商国：桃花源记\scripts\cards\card_base.gd` 中 `_card_style` 配色字典。

每张卡的颜色从 Godot hex 色值直接转 RGBA：
- 草皮：`#c9f28b` → `(201, 242, 139, 255)`
- 羊：`#f5e6ca` → `(245, 230, 202, 255)`
- 狼：`#8b4513` → `(139, 69, 19, 255)`
- 等

卡的颜色列表若有遗漏，以 `card_base.gd` 为唯一真相源。

### 完整卡表

以下列表对照 `game-design-overview.md` 第 3 节"当前已实现的完整卡表"：

**大型动物**：sheep, lamb, deer, deerFawn, wolf, wolfCub, fox, foxCub, waterBuffalo, waterBuffaloCalf

**小型动物**：rabbit, fieldMouse, fieldMousePup, pheasant, pheasantChick, bambooRat

**水生**：algae, waterBug, fish, shellfish

**树内容纳**：oak, pine, birdNest

**水潭容纳**：waterCaltrop, lotus

**地下容纳**：wildYam

**掉落物**：acorn, pineCone, caltropFruit, lotusSeed, wildYamRoot

**陆虫**：landBug

**建筑**：wolfDen, foxDen, fire, hut

**植物**：grass, dryGrass, bush

**资源**：stone, shard, tri, square, knife, spear, wood, twig, woodStruct, axe, hoe, hammer, bucket, waterbucket, halfbucket, berry, mushroom, wetWood, mushroomWood, charcoal, grassRope

**肉类**：sheepMeat, rabbitMeat, deerMeat, wolfMeat, humanMeat, fishMeat, cookmeat

**尸体**：sheepCorpse, deerCorpse, wolfCorpse, playerCorpse

**人类**：player, taoyuanElder, taoyuanForager, taoyuanYouth

**冻结卡**（POC 阶段可跳过索引但保留定义）：traveler, mushroomFarmer, mushroomGreenhouse, coin, copperBlock, copperCraft, table

---

## 二、card_def.rs — 数据结构

扩展 POC 现有结构体，加颜色和 icon：

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CardDef {
    pub type_name: String,
    pub display_name: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub color: (u8, u8, u8, u8),
    pub hp: i32,
    pub is_rooted: bool,
}
```

新增 `CardDef::card_has_tag(&self, tag: &str) -> bool` —— 在 tags 向量中匹配，含前缀匹配（`tag.begins_with`）。

---

## 三、world_rules.rs — 核心标签判定

**参照 Godot**：`E:\桌面\方寸商国：桃花源记\scripts\core\world_rules.gd`

以下函数全部迁移。参数从 GDScript `CardBase` 改为 `&CardDef` + 运行时状态 struct。

### 3.1 标签判定基础

| 函数 | Godot 来源 | 说明 |
|------|-----------|------|
| `card_has_tag(def, tag)` | `card_has_tag` | 含前缀匹配 |
| `card_has_capability(def, cap)` | `card_has_capability` | 从 CARD_CAPABILITIES 查 |
| `is_being(def)` | 等效 `tag=="being"` | |
| `is_animal(def)` | 等效 `tag=="animal"` | |
| `is_predator(def)` | 等效 `tag=="predator"` | |
| `is_mesopredator(def)` | 等效 `tag=="mesopredator"` | |
| `is_herbivore(def)` | 等效 `tag=="herbivore"` | |
| `is_juvenile(def)` | 等效 `tag=="juvenile"` | |
| `is_small_prey(def)` | 等效 `tag=="smallPrey"` | |
| `is_large_prey(def)` | 等效 `tag=="largePrey"` | |

### 3.2 生态判定

| 函数 | 说明 |
|------|------|
| `flocking_blocks_reproduction(adult_defs)` | 同 `flocking` 判定，成体 < 3 → true |
| `pack_hunter_under_strength(hunter_def)` | 单狼 → 只猎 smallPrey |
| `is_grazer_flee_wolf_threat(actor_def)` | 狼是否对食草动物构成逃跑威胁 |
| `is_hunt_target_for(hunter_def, target_def)` | 猎物判定链，复用 Godot `HUNT_PROFILE` 标签 |
| `hunt_target_score(hunter_def, target_def)` | 猎物评分，复刻 Godot 评分逻辑 |
| `best_hunt_target(hunter_def, candidates)` | 从候选列表选最优猎物 |
| `is_feed_source(actor_def, source_def)` | 喂食源判定 |
| `best_feed_source_for(actor_def, candidates)` | 最优喂食源 |
| `mark_perishable(entity)` | 生肉标记腐坏倒计时 |
| `is_aquatic_card(def)` | `aquatic` 标签 |
| `is_burrower(def)` | `burrower` 标签 |

### 3.3 繁殖与生命周期

| 函数 | 说明 |
|------|------|
| `can_reproduce(male_def, female_def)` | 同种+公母 |
| `prolific_litter_size(def)` | `prolific` → 3，默认 1 |
| `prolific_repro_cycle(def)` | `prolific` → 3s，默认 `POPULATION_REPRO_CYCLE` |

---

## 四、CARD_CAPABILITIES 迁移

**参照 Godot**：`world_rules.gd` 中 `CARD_CAPABILITIES` 字典。

Rust 中用 `HashMap<String, Vec<String>>` 或编译期 `phf` 静态表。每条同 Godot。示例：

```rust
static CARD_CAPABILITIES: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "player" => &["capability.move", "capability.hunt", "capability.forage", ...],
    "sheep" => &["capability.move", "capability.forage", "capability.flee", ...],
    ...
};
```

Cursor 决定用 `phf`、`lazy_static`、还是 `once_cell::Lazy`。

---

## 五、格网与卡牌渲染颜色

### 5.1 地形颜色

**参照 Godot**：`E:\桌面\方寸商国：桃花源记\scripts\ui\terrain_visual_palette.gd`

```rust
pub fn terrain_color(terrain_type: &str) -> (u8, u8, u8, u8) {
    match terrain_type {
        "grassland" => (196, 224, 160, 255),
        "riverbank" => (180, 215, 140, 255),
        "bank" => (170, 200, 130, 255),
        "river" => (100, 160, 220, 255),
        "ford" => (130, 180, 210, 255),
        "pool" => (80, 140, 210, 255),
        "dark_river_pool" => (40, 80, 160, 255),
        "cliff" => (140, 130, 120, 255),
        "wetland" => (160, 190, 150, 255),
        _ => (180, 175, 165, 255), // default soil
    }
}
```

### 5.2 卡牌颜色

`CardDef.color` 字段已在 RON 中定义。渲染时直接从 `CardDef` 读取。

### 5.3 选中高亮

选中卡 → 绿色边框。`(50, 220, 80, 255)` 描边 2px。

---

## 六、项目文件结构

```
bevy-poc/
├── Cargo.toml
├── assets/
│   └── card_defs.ron              # 全卡定义（M1 重点）
├── src/
│   ├── main.rs                     # 入口
│   ├── card_def.rs                 # CardDef 结构体 + 加载（扩展）
│   ├── spatial_index.rs            # 已有，不动
│   ├── world_rules.rs              # 核心判定（M1 重点，大幅扩充）
│   ├── capabilities.rs             # CARD_CAPABILITIES 静态表（新建）
│   ├── terrain_colors.rs           # 格网配色（新建）
│   ├── world_state.rs              # 世界状态（已有，可能微调）
│   ├── grid_render.rs              # 渲染（M1 扩展颜色）
│   ├── bench.rs                    # 已有
│   └── systems/
│       ├── mod.rs
│       ├── tick_herbivore.rs       # 已有，M2 扩展
│       ├── tick_predator.rs        # 已有，M2 扩展
│       └── grass_regen.rs          # 已有
└── tests/
    └── assertions.rs               # M1 扩充至 50+ 条
```

---

## 七、断言（M1 新增）

在 POC 20 条基础上加 30 条，覆盖标签和判定：

| # | 断言 |
|----|------|
| 21 | card_defs.ron 加载成功，总数 ≥ 50 |
| 22 | sheep 有 `flocking` `herbivore` `largePrey` |
| 23 | wolf 有 `predator` `pack_hunter` |
| 24 | deer 有 `wildPrey` `largePrey` |
| 25 | rabbit 有 `prolific` `smallHerbivore` |
| 26 | fish 有 `aquatic` `small` |
| 27 | shellfish 有 `sessile` |
| 28 | oak 有 `nut_producer` `rooted` |
| 29 | waterCaltrop 有 `floating` |
| 30 | wildYam 有 `tuber` `underground` |
| 31 | landBug 有 `volant` `smallPrey` |
| 32 | 所有肉卡有 `perishable` |
| 33 | 所有尸体卡有 `corpse` |
| 34 | `is_predator(wolf_def)` → true |
| 35 | `is_predator(sheep_def)` → false |
| 36 | `is_herbivore(sheep_def)` → true |
| 37 | `is_herbivore(wolf_def)` → false |
| 38 | `is_aquatic_card(fish_def)` → true |
| 39 | `is_aquatic_card(sheep_def)` → false |
| 40 | `card_has_tag(oak_def, "nut_producer")` → true |
| 41 | `card_has_tag(fire_def, "camp.anchor")` → true |
| 42 | `card_has_tag(stone_def, "material.stone")` → true |
| 43 | `card_has_tag(bucket_def, "container.water")` → true |
| 44 | `flocking_blocks_reproduction([sheep, sheep])` → true (2 < 3) |
| 45 | `flocking_blocks_reproduction([sheep, sheep, sheep])` → false |
| 46 | `pack_hunter_under_strength([wolf])` → true (只有 1 只) |
| 47 | `pack_hunter_under_strength([wolf, wolf])` → false |
| 48 | `is_hunt_target_for(wolf_def, sheep_def)` → true |
| 49 | `is_hunt_target_for(wolf_def, rabbit_def)` → false (large prey only) |
| 50 | CARD_CAPABILITIES 中每张卡的能力列表非空 |

---

## 八、验收

- `cargo test`：50 条断言全部 PASS（POC 20 + M1 新增 30）
- `cargo run --release`：格网渲染正确，草绿色、羊白色、狼红色，河岸蓝色
- `cargo run --release -- --bench`：avg_tick_ms 保持在 < 5ms

## 约束

- 不迁行为逻辑（M2）
- 不迁 UI（M3）
- 不碰 Godot 文件
- `world_rules.rs` 只做判定函数，不做 tick 逻辑
- 记 fix-log（在 Bevy 项目根目录建 `FIX_LOG.md`）
