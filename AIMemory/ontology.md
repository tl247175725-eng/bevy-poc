# 桃花源抽象组件本体

> **单一真相源。** 所有标签维度、元数值、元动作在此定义。
> **源码级。** 和代码一起版本控制。改代码前先改本体。
> **机器可读。** 结构化 YAML，AI 和脚本均可解析。

---

## 标签维度（17 维，~184 标签）

```yaml
tag_dimensions:
  # ═══ 生态位维度 ═══
  habitat:           # 栖息地 | PanTHERIA | 种级 | 7标签
    labels: [aquatic, wetland, grassland, forest, mountain, subterranean, aerial]
    affects: [spawn_animals, Move, perception]

  diet:              # 食性 | EltonTraits | 种级 | 13标签
    labels: [carnivore, herbivore, omnivore, piscivore, insectivore, frugivore,
             granivore, detritivore, scavenger, nectar_feeder, filter_feeder,
             wood_eater, sanguivore]
    derives: [can_digest]
    affects: [Consume]

  foraging:          # 觅食策略 | EltonTraits | 种级 | 9标签
    labels: [ambush, pursuit, graze, browse, scavenge, filter, drill, trap, cooperative_hunt]
    affects: [Strike, Move]

  foraging_stratum:  # 觅食层位 | EltonTraits | 种级 | 7标签
    labels: [ground, understory, canopy, aerial, aquatic_surface, aquatic_submerged, bark]

  # ═══ 生理维度 ═══
  defense:           # 防御 | Winemiller 2015 | 器官级 | 9标签
    labels: [flee, hide, fight, armor, venom, camoflage, mimicry, chemical_spray, autotomy]
    affects: [Strike]

  thermo:            # 体温调节 | PanTHERIA | 器官级 | 2标签
    labels: [endotherm, ectotherm]

  metab:             # 代谢率 | PanTHERIA | 器官级 | 4标签
    labels: [high, medium, low, torpor]
    derives: [metab_rate, decay_rate]

  body_size:         # 体型 | PanTHERIA Body Mass | 种级 | 5标签
    labels: [tiny, small, medium, large, huge]
    derives: [estimate_mass_from_tags]
    affects: [Strike, Move, Consume]

  body_plan:         # 身体结构 | 形态学 | 器官级 | 8标签
    labels: [biped, quadruped, serpentine, avian, fish, insectoid, plant, amorphous]
    affects: [Move, Strike]

  # ═══ 行为维度 ═══
  capability:        # 能力 | 功能性状效应 | 器官级 | 15标签
    labels: [fly, swim, climb, burrow, dig, run, jump, glide, dive,
             grasp, bite, constrict, echolocate, regenerate, tool_use]
    derives: [impact_force参数]
    affects: [Strike, Move]

  cognition:         # 认知 | 游戏专属 | 种级 | 5标签
    labels: [instinct_only, basic_learning, tool_use, complex_reasoning, cultural_transmission]
    affects: [Decide, combat_proficiency上限]

  sense:             # 感知 | 感官生物学 | 器官级 | 8标签
    labels: [vision, hearing, smell, touch, echolocation, infrared, electrosense, magnetosense]
    affects: [Perceive]

  social:            # 社会结构 | PanTHERIA | 种级 | 8标签
    labels: [solitary, pair, pack, herd, colony, territorial, hierarchical, eusocial]
    derives: [sexual_dimorphism]
    affects: [Social need, Reproduce]

  activity:          # 活动节律 | PanTHERIA | 种级 | 5标签
    labels: [diurnal, nocturnal, crepuscular, cathemeral, arrhythmic]

  movement:          # 移动模式 | PanTHERIA | 种级 | 4标签
    labels: [sedentary, nomadic, migratory, territorial_patrol]
    affects: [Move]

  # ═══ 生活史维度 ═══
  repro:             # 繁殖 | PanTHERIA | 种级 | 8标签
    labels: [few_offspring, many_offspring, parental_care, no_parental_care,
             egg_layer, live_birth, semelparous, iteroparous]
    affects: [Reproduce]

  growth:            # 生长 | PanTHERIA | 种级 | 4标签
    labels: [fast, medium, slow, metamorphosis]
    derives: [age_curve]

  habitat_range:     # 栖息地专化度 | COMBINE | 种级 | 2标签
    labels: [specialist, generalist]

  # ═══ 植物专属维度 ═══
  nutrition:         # 营养方式 | Díaz et al. | 种级 | 5标签
    labels: [autotroph, hemiparasitic, holoparasitic, carnivorous, detritivorous]
    affects: [Consume]

  fire_response:     # 火烧响应 | Pérez-Harguindeguy 2013 | 种级 | 3标签
    labels: [killed, resprouting, fire_dependent]

  drought_tolerance: # 耐旱 | Pérez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  flooding_tolerance:# 耐淹 | Pérez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  shade_tolerance:   # 耐阴 | Pérez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  dispersal:         # 种子传播 | Weiher et al. 1999 | 种级 | 5标签
    labels: [wind, animal, water, explosive, gravity]

  growth_form:       # 生长型 | Díaz et al. 2016 | 种级 | 6标签
    labels: [tree, shrub, grass, vine, aquatic, epiphyte]

  woodiness:         # 木质化 | 形态学 | 种级 | 2标签
    labels: [woody, herbaceous]

  lifespan:          # 寿命 | 形态学 | 种级 | 3标签
    labels: [annual, perennial, long_lived]
    derives: [max_age]

  # ═══ 运行时维度 ═══
  state:             # 状态 | 运行时 | 运行时 | 9标签
    labels: [healthy, injured, sick, starving, exhausted, pregnant, growing, dying, dead]

  injury:            # 损伤 | 运行时 | 运行时 | 7标签
    labels: [bruised, fractured, severed, bleeding, infected, scarred, missing]

  personality:       # 人格 | 游戏专属 | 个体级 | 5标签
    labels: [bold, cautious, curious, aggressive, social]
    affects: [Decide过滤]

  # ═══ 基础分类（自动注册） ═══
  base:              # 基础分类 | 游戏内 | - | 6标签
    labels: [terrain, tree, plant, animal, fish, role:player]
    note: "card_defs.ron 中使用的高频元标签，注册在 TagRegistry bit 264-269"
```

---

## 元数值

```yaml
meta_values:
  A层_必须:
    time:
      - TICK_SECONDS: { value: 0.5, desc: "渲染帧步长(非逻辑时间)" }
      - TICKS_PER_DAY: { value: 420, desc: "1天=tick数", invariant: "铁律，改此值=改时间尺度" }
      - TICKS_PER_PHASE: { value: 60 }
      - PHASES_PER_DAY: { value: 7 }
    space:
      - GRID_CELL_SIZE: { value: 1.0 }
    materials:
      - DENSITY_FLESH: { value: 1050, desc: "kg/m³" }
      - DENSITY_WOOD: { value: 700 }
      - DENSITY_STONE: { value: 2700 }
      - DENSITY_STEEL: { value: 7850 }
      - HARDNESS_WOOD: { value: 1.0, desc: "摩斯" }
      - HARDNESS_STEEL: { value: 6.5 }
      - YIELD_STEEL: { value: 800, desc: "MPa" }
    thermal:
      - TEMP_BODY_MAMMAL: { value: 37.0 }
    senses:
      - VISION_RANGE_DEFAULT: { value: 6 }
      - HEARING_RANGE_DEFAULT: { value: 8 }
      - SMELL_RANGE_DEFAULT: { value: 4 }

  B层_强推荐:
    life:
      - HP_BASELINE: { value: 1, note: "妥协值，后续被Strike公式替代" }
      - METABOLISM_BASELINE: { value: 1.0 }
    mind:
      - DECISION_THRESHOLD_DEFAULT: { value: 1.2 }
    social:
      - NORM_STRENGTH_DEFAULT: { value: 0.5 }
    ecology:
      - NUTRITION_DECAY_HIGH: { value: 0.7, derives_from: "metab:high" }
      - NUTRITION_DECAY_MEDIUM: { value: 0.4, derives_from: "metab:medium" }
      - NUTRITION_DECAY_LOW: { value: 0.2, derives_from: "metab:low" }
      - SOCIAL_DECAY: { value: 0.1 }
      - CURIOSITY_DECAY: { value: 0.05 }
      - NUTRITION_BASELINE: { value: 0.3 }
      - SAFETY_BASELINE: { value: 1.0 }
      - SOCIAL_BASELINE: { value: 0.5 }
      - CURIOSITY_BASELINE: { value: 0.2 }
      - MAX_SENSE_RANGE: { value: 20 }
      - URGENCY_ACTIVATION_THRESHOLD: { value: 0.3 }
      - SAFETY_BLOCK_THRESHOLD: { value: 0.7 }
      - DIGESTION_EFFICIENCY: { value: 0.5 }
      - STRIKE_BASE_DAMAGE: { value: 1, note: "待Strike公理替代" }

  派生函数:
    - baseline_energy: { inputs: [mass, metab_rate], output: "f32", used_by: [Consume] }
    - impact_force: { inputs: [mass, velocity, contact_area, hardness_ratio], output: "f32", used_by: [Strike] }
    - estimate_mass_from_tags: { inputs: [body_size TagBits], output: "f32(kg)", used_by: [Consume, Strike, Move] }
    - speed_from_mass: { inputs: [mass], output: "f32" }
    - weight_from_mass_density: { inputs: [mass, density], output: "f32" }
    - age_strength_factor: { inputs: [age, maturity, max_age], output: "f32", note: "待实现" }
```

---

## 元动作（25变体7层）

```yaml
meta_actions:
  物理层_11:
    - Move:    { axiom: "traverse(待接)", params: [dx, dy] }
    - Strike:  { axiom: "impact_force()", params: [target] }
    - Consume: { axiom: "can_digest()", params: [target] }
    - Combine: { params: [ingredient] }
    - Separate:{ params: [target] }
    - Constrain:{ params: [target] }
    - Release: { params: [x, y] }
    - Pause:   { params: [ticks] }
    - Hide:    { params: [cover_id] }
    - Emerge:  {}
    - Alter:   { params: [target] }

  信息层_4:
    - Signal:  { params: [content, range] }
    - Receive: { params: [source] }
    - Process: { params: [input] }
    - Store:   { params: [content, target] }

  生物层_3:
    - Reproduce: { params: [partner] }
    - Inherit:   { params: [parent] }
    - Transform: { params: [target] }

  心智层_3:
    - Decide:  { params: [intention] }
    - Teach:   { params: [target, skill] }
    - Bond:    { params: [target, bond_type] }

  社会层_2:
    - Assign:  { params: [target, role] }
    - Commit:  { params: [target, obligation] }

  超自然层_2:
    - Create:  { params: [template, x, y] }
    - Destroy: { params: [target] }
```

---

## 公理

```yaml
axioms:
  - id: can_digest
    file: src/axioms/consume.rs
    inputs: [actor_tags, target_tags, target_is_corpse]
    output: bool
    validates: [Consume]
    status: ✅ 已贯通

  - id: traverse
    file: src/axioms/laws.rs
    validates: [Move]
    status: ❌ 已定义但未在apply_meta_action调用

  - id: impact_force / strike_force
    file: src/axioms/strike.rs
    validates: [Strike]
    status: ✅ 已贯通（035）——body_plan×body_size×capability→两条物理公式

  - id: compose
    file: src/axioms/laws.rs
    validates: [spawn, Move]
    status: ⚠️ 部分使用

  - id: perceive
    file: src/axioms/laws.rs
    validates: [Perceive]
    status: ⚠️ 使用旧EntityProfile系统
```

---

## 关键关联映射

```yaml
cross_references:
  # 标签 → 元数值
  tag_to_value:
    body_size → estimate_mass_from_tags
    metab → decay_rate, baseline_energy
    cognition → combat_proficiency上限
    lifespan → max_age, age_strength_factor

  # 标签 → 元动作
  tag_to_action:
    diet → Consume
    capability + body_size → Strike
    body_plan + capability + body_size → Move
    repro + social → Reproduce
    cognition → Decide

  # 标签 → 标签
  tag_to_tag:
    diet + foraging + body_size → 攻击策略
    social → sexual_dimorphism(派生)
    reproduction + growth + lifespan → 年龄曲线
```

---

## 抽象深度统一规则

```yaml
abstraction_rules:
  - "所有维度深度不超过器官级（DF级）"
  - "同维度内标签深度必须一致（如body_size内部全是种级）"
  - "动物标签和植物标签不混用维度"
  - "新增维度必须在来源文献中有依据"

  depth_levels:
    分子级: ["不用于游戏——除非涉及毒液/毒素的生化机制"]
    细胞级: ["不用于游戏"]
    组织级: ["injury 维度在此深度"]
    器官级: ["capability, defense, body_plan, sense 在此深度——最高上限"]
    个体级: ["personality 在此深度"]
    种级: ["diet, habitat, body_size, social, repro 在此深度——默认深度"]
    生态级: ["foraging 策略在此深度"]
```

---

## 变更流程

```yaml
change_process:
  1_design:
    - "设计提案中说明：新增什么、属于哪个维度、抽象深度级别、来源文献"
  2_ontology:
    - "在本体对应节加一条 YAML"
    - "检查抽象深度是否和同维度一致"
    - "更新 cross_references 关联"
  3_sync:
    - "tags.ron: 加标签定义"
    - "tags.rs: 加 TagInfo 常量 + TAG_CONSTANTS 引用 + 分配 bit"
    - "meta_values.rs: 加常量（如需）"
    - "card_defs.ron: 给对应卡牌加标签"
  4_validate:
    - "bash scripts/check-iron-law.sh"
    - "cargo test"
  5_commit:
    - "ontology.md 和代码在同一个 commit"
```

---

## 不变量

```yaml
invariants:
  - "每个动物必须有 body_size 标签（恰好一个）"
  - "diet:* 标签只出现在 animal 实体上"
  - "nutrition:autotroph 只出现在 plant 实体上"
  - "所有裸数字必须追溯到 meta_values.rs"
  - "所有标签名必须在 TagRegistry 注册"
  - "ontology.md 和代码同步变更——同一 PR/commit"
  - "加新标签维度不能引入新的抽象深度级别"
```

---

> **维护规则**：改代码前先看本体。本体和代码不一致时，以本体为准——要么修正代码，要么修正本体并更新变更记录。
