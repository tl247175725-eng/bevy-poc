# Handoff 025 — 植物专属标签 + 植物卡修正

## 架构计划

**改什么：** `src/tags.rs` + `assets/card_defs.ron`（2 文件）
**依据：** design-philosophy-v5.md §13 抽象深度统一规则

### 1. tags.rs — 新增 30 个植物专属标签常量

在 `tag` 模块新增（bit 范围 231-260，远离此前范围）：

```
nutrition: autotroph, hemiparasitic, holoparasitic, carnivorous, detritivorous
fire_response: killed, resprouting, fire_dependent
drought_tolerance: low, medium, high
flooding_tolerance: low, medium, high
shade_tolerance: low, medium, high
dispersal: wind, animal, water, explosive, gravity
growth_form: tree, shrub, grass, vine, aquatic, epiphyte
woodiness: woody, herbaceous
lifespan: annual, perennial, long_lived
```

每个标签一行 TagInfo 常量。在 default_registry() 和 TAG_CONSTANTS 中注册。

### 2. card_defs.ron — 修正所有植物卡

**错误：** 植物卡错误使用了 `diet:herbivore`
**正确：** 植物用 `nutrition:autotroph`

每张植物卡对照抽象深度模板（§13.3），确保以下维度齐全：

```
植物-树木: habitat, nutrition:autotroph, growth_form, woodiness, lifespan,
           growth, repro, metab, defense, fire_response,
           drought_tolerance, flooding_tolerance, shade_tolerance,
           dispersal, body_plan:plant, body_size
植物-草本: 同上，growth:fast, lifespan:perennial/annual
地形卡: habitat, state, body_plan:amorphous（无植物标签）
```

### 具体修正

- lotus: 加 `nutrition:autotroph`, `drought_tolerance:low`, `flooding_tolerance:high`
- waterweed: 同上
- reed/cattail: 加 `nutrition:autotroph`, `fire_response:resprouting`, `flooding_tolerance:high`
- miscanthus: 加 `nutrition:autotroph`, `fire_response:resprouting`, `drought_tolerance:high`
- nanmu_tree: 加 `nutrition:autotroph`, `growth_form:tree`, `woodiness:woody`, `lifespan:long_lived`, `fire_response:killed`, `shade_tolerance:low`
- camphor_tree: 同 nanmu 模式
- bamboo: 加 `nutrition:autotroph`, `growth_form:grass`, `woodiness:woody`, `fire_response:resprouting`
- pine_forest: 加 `nutrition:autotroph`, `growth_form:tree`, `woodiness:woody`, `fire_response:killed`, `drought_tolerance:high`
- azalea: 加 `nutrition:autotroph`, `growth_form:shrub`, `woodiness:woody`, `fire_response:resprouting`, `shade_tolerance:high`
- lichen: 加 `nutrition:autotroph`, `drought_tolerance:high`

**所有植物卡移除 `diet:herbivore`。**

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- card_audit 不报错
