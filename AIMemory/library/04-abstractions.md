# 四、抽象体系

### 标签维度（17 维，~184 标签）

```yaml
tag_dimensions:
  # === 生态位维度 ===
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

  # === 生理维度 ===
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

  # === 行为维度 ===
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

  # === 生活史维度 ===
  repro:             # 繁殖 | PanTHERIA | 种级 | 8标签
    labels: [few_offspring, many_offspring, parental_care, no_parental_care,
             egg_layer, live_birth, semelparous, iteroparous]
    affects: [Reproduce]

  growth:            # 生长 | PanTHERIA | 种级 | 4标签
    labels: [fast, medium, slow, metamorphosis]
    derives: [age_curve]

  habitat_range:     # 栖息地专化度 | COMBINE | 种级 | 2标签
    labels: [specialist, generalist]

  # === 植物专属维度 ===
  nutrition:         # 营养方式 | Diaz et al. | 种级 | 5标签
    labels: [autotroph, hemiparasitic, holoparasitic, carnivorous, detritivorous]
    affects: [Consume]

  fire_response:     # 火烧响应 | Perez-Harguindeguy 2013 | 种级 | 3标签
    labels: [killed, resprouting, fire_dependent]

  drought_tolerance: # 耐旱 | Perez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  flooding_tolerance:# 耐淹 | Perez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  shade_tolerance:   # 耐阴 | Perez-Harguindeguy 2013 | 种级 | 3标签
    labels: [low, medium, high]

  dispersal:         # 种子传播 | Weiher et al. 1999 | 种级 | 5标签
    labels: [wind, animal, water, explosive, gravity]

  growth_form:       # 生长型 | Diaz et al. 2016 | 种级 | 6标签
    labels: [tree, shrub, grass, vine, aquatic, epiphyte]

  woodiness:         # 木质化 | 形态学 | 种级 | 2标签
    labels: [woody, herbaceous]

  lifespan:          # 寿命 | 形态学 | 种级 | 3标签
    labels: [annual, perennial, long_lived]
    derives: [max_age]

  # === 运行时维度 ===
  state:             # 状态 | 运行时 | 运行时 | 9标签
    labels: [healthy, injured, sick, starving, exhausted, pregnant, growing, dying, dead]

  injury:            # 损伤 | 运行时 | 运行时 | 7标签
    labels: [bruised, fractured, severed, bleeding, infected, scarred, missing]

  personality:       # 人格 | 游戏专属 | 个体级 | 5标签
    labels: [bold, cautious, curious, aggressive, social]
    affects: [Decide过滤]

  # === 基础分类（自动注册） ===
  base:              # 基础分类 | 游戏内 | - | 6标签
    labels: [terrain, tree, plant, animal, fish, role:player]
    note: "card_defs.ron 中使用的高频元标签，注册在 TagRegistry bit 264-269"
```

---

### 元数值

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
      - DENSITY_FLESH: { value: 1050, desc: "kg/m3" }
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

  元数值分层原则:
    A_必须: "世界无法闭环，物理引擎刚需（tick/cell/mass/temperature/pH）"
    B_强推荐: "涌现依赖，高层行为的前提（hp/trust/reputation/scarcity）"
    C_可选: "看后期，不影响当前开发（电磁/超自然参数）"
    D_派生: "不进入元数值表（speed/damage/price/morale）"

  元数值判断标准:
    - "1. 不能由已有元数值稳定推出"
    - "2. 是大量规则、状态或结果的共同输入"
    - "3. 可以被元动作/Elapse/Chance 持续读取和改变"
    - "4. 派生值不进入元数值表"
    - "5. 所有游戏中的数字都应能从元数值派生"

  派生规则示例:
    - "speed = distance / duration"
    - "weight = mass x gravity（元数值是 mass，不是 weight）"
    - "hunger = nutrition_need - nutrition"
    - "damage = force x edge x material_response x angle"
    - "price = demand x scarcity x exchange_value x trust_modifier"
```

---

### 元动作（25 变体 7 层）

```yaml
meta_actions:
  物理层_11:
    - Move:    { axiom: "traverse(待接)", params: [dx, dy] }
    - Strike:  { axiom: "impact_force()", params: [target] }
    - Consume: { axiom: "can_digest()", params: [target] }
    - Combine: { params: [ingredient], note: "无配方表，属性代数合并" }
    - Separate:{ params: [target] }
    - Constrain:{ params: [target] }
    - Release: { params: [x, y] }
    - Pause:   { params: [ticks], note: "标签分化: Rest/Think/Observe/Appreciate/Examine" }
    - Hide:    { params: [cover_id] }
    - Emerge:  {}
    - Alter:   { params: [target], note: "烹饪/燃烧/冶炼/发酵——改属性值不改身份" }

  信息层_4:
    - Signal:  { params: [content, range] }
    - Receive: { params: [source] }
    - Process: { params: [input], note: "主动心智工作，不是Pause的副产品" }
    - Store:   { params: [content, target], note: "主动编码记忆" }

  生物层_3:
    - Reproduce: { params: [partner] }
    - Inherit:   { params: [parent], note: "Reproduce的子过程，标签继承非随机" }
    - Transform: { params: [target], note: "成长/变态/愈合/衰老，不同于Alter" }

  心智层_3:
    - Decide:  { params: [intention], note: "需求匹配引擎输出激活Decide" }
    - Teach:   { params: [target, skill], note: "永久转移标签，不是Signal" }
    - Bond:    { params: [target, bond_type], note: "创建关系链接，不是Combine" }

  社会层_2:
    - Assign:  { params: [target, role], note: "改变社会现实（命名/定价/所有权）" }
    - Commit:  { params: [target, obligation], note: "规范性约束（契约/债务/禁令）" }

  超自然层_2:
    - Create:  { params: [template, x, y], note: "无前因产出新实体" }
    - Destroy: { params: [target], note: "从因果链彻底移除" }

  底层机制_2:
    - Elapse: "时间流逝——世界推进。所有 duration/cooldown/decay 的驱动源"
    - Chance: "概率结算——多结果随机分配。所有 mutation/critical/drop 的随机源"

  无配方原则: |
    Combine 不做 a+b=c。不做配方表。
    产物由输入的材质属性代数合并决定。
    引擎不认识"刀"——只认识 {hard>=4, shape:blade, edge:present}。
    任何"带把的硬片"都可以当刀。好不好用由物理算，不由配方判。

  元动作判断标准:
    不可分解性: "能分解为已有元动作的序列？ 是->不是元动作"
    产物新颖性: "创造了已有元动作创造不了的东西？ 是->是元动作"
    跨层效应: "跨越物理/信息/生物/心智/社会层？ 是->是元动作"
    因果独立性: "能用已有元动作的因果链完整描述？ 是->不是元动作"
    标签不可替代性: "用已有元动作+标签能达到同样效果？ 是->不是元动作"
```

---

### 公理

```yaml
axioms:
  - id: can_digest
    file: src/axioms/consume.rs
    inputs: [actor_tags, target_tags, target_is_corpse]
    output: bool
    validates: [Consume]
    status: "已贯通"

  - id: traverse
    file: src/axioms/laws.rs
    validates: [Move]
    status: "已定义但未在apply_meta_action调用"

  - id: impact_force / strike_force
    file: src/axioms/strike.rs
    validates: [Strike]
    status: "已贯通（035）——body_plan x body_size x capability -> 两条物理公式"

  - id: compose
    file: src/axioms/laws.rs
    validates: [spawn, Move]
    status: "部分使用"

  - id: perceive
    file: src/axioms/laws.rs
    validates: [Perceive]
    status: "使用旧EntityProfile系统"
```

---

### 标签纯度规则

> 基于功能性状生态学 (Winemiller 2015) + BFO 本体论工程学。

**卡的定义：** 一张卡 = 在所选粒度层级上，能够同时满足以下三个条件的实体：
1. **独立承载标签**（不是继承自父实体的）
2. **独立参与元动作**（可以被 Move/Strike/Consume/Store）
3. **独立存在于格子上**（有自己的位置状态）

不满足则不是卡，是标签或元数值。同类别深度一致：哺乳动物全模拟器官，鸟全模拟翅膀，虫不模拟器官。不跨类别不一致。

**纯度验证五条（每次新增标签必须通过）：**

| # | 标准 | 测试 |
|---|---|---|
| 1 | **不可含物种名** | `predator` 不是 `wolf`。标签是描述属性，不是分类物种 |
| 2 | **可传递** | 任何带 `predator` 的实体触发相同的 fear 反应——不查物种 |
| 3 | **可组合** | `predator + nocturnal + pack_hunter` 自动涌现行为，不需要额外规则 |
| 4 | **趋同验证** | 该性状在三个以上不相关谱系中独立进化过 -> 高纯度。只有一种动物有 -> 不是标签 |
| 5 | **行为闭包** | 有此标签 -> 引擎能产出完整行为后果——不需要额外查物种 |

**标签五维度（功能性状生态学）：**

| 维度 | 含义 | 标签前缀 |
|---|---|---|
| 栖息地 | 在哪里生活 | `habitat:` |
| 生活史 | 怎么生长繁殖 | `repro:` `growth:` |
| 营养 | 吃什么、怎么获取 | `diet:` `foraging:` |
| 防御 | 怎么不被吃 | `defense:` |
| 代谢 | 怎么处理能量 | `metab:` `thermo:` |

加上：社会（`social:`）、身体（`body:`）、能力（`capability:`）。

**标签技术架构：**
- **集中注册表**（TagRegistry）——"生态位周期表"，所有标签的 bit 位在此定义
- **每实体 bitmask**（TagBits）——实体在性状空间中的位置
- **查询** = bitmask AND，O(1)

---

### 抽象深度规则

> 基于 BFO 四原则 + 功能性状生态学。

**核心原则：同构——让万物按本来的本质存在。** 标签不是我们赋予实体的行为。标签是实体本来的样子。引擎读标签，行为自然流出。

**BFO 抽象深度四原则：**

| 原则 | 含义 | 我们的规则 |
|---|---|---|
| **Adequatism** | 建模到目的所需即可 | 能被砸/叠/拿/抽操作 -> 必须建模。不能被操作 -> 不建模 |
| **Perspectivalism** | 不同域可不同粒度，但同域必须一致 | 不同类别可不同维度，但同类内深度完全相同 |
| **Realism** | 只描述真实存在的东西 | 标签从性状提取，不发明。趋同验证（三谱系以上独立进化） |
| **Fallibilism** | 可修订 | 发现缺标签 -> 补。但补了之后同类全补，保持深度一致 |

**抽象深度统一规则：**
1. **同类绝对一致**：同类别内所有成员共用同一套标签模板。模板深度由该类别中最复杂的成员决定。
2. **跨类各自独立**：不同类别可用不同维度。动物用 `diet`，植物用 `nutrition`——不是"植物更浅"，是"植物有不同的本质"。
3. **目的测试**：该属性是否影响游戏操作（砸/叠/拿/抽）或引擎计算（Need/Perceive/Decide）？是 -> 必须建模。否 -> 不建模。
4. **粒度上下限**：上限=器官级别（肝/心/肺可被砸伤）。下限=不模拟细胞/分子/原子。

**各类别标签深度模板：**

| 类别 | 必修标签维度 | 禁入标签维度 |
|---|---|---|
| 动物-哺乳类 | diet/foraging/foraging_stratum/defense/thermo/metab/repro/growth/social/activity/movement/cognition/sense/body_size/body_plan/capability/habitat/habitat_range + 器官标签 | -- |
| 动物-鸟类 | 同上 + body_plan:avian + capability:fly | -- |
| 动物-爬行类 | 同上（thermo:ectotherm），cognition 上限 basic_learning | cultural_transmission |
| 动物-鱼类 | 同上，body_plan:fish | cognition |
| 植物-树木 | habitat/growth_form/woodiness/lifespan/nutrition/growth/repro/metab/defense/fire_response/drought_tolerance/flooding_tolerance/shade_tolerance/dispersal/body_plan:plant/body_size | cognition/activity/social/movement/foraging |
| 植物-草本 | 同上 + growth:fast | 同上 |
| 地形 | habitat/state/body_plan | 全部生物标签 |

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
