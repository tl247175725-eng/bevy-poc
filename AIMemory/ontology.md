# 桃花源设计图书馆（唯一设计资料源）

> **单一真相源。** 打开此文件即可理解整个项目的设计。
> **源码级。** 和代码一起版本控制。改代码前先改本体。
> **机器可读。** 结构化 YAML，AI 和脚本均可解析。

---

## 一、项目定义

**方寸商国：桃花源记** — 一个标签驱动的卡牌生态模拟游戏。南宋商人逃入桃花源，从零开始生存、建造、交易，最终把一个小山谷发展成商业枢纽。Rust + Bevy 0.15 ECS。游戏的深度不在脚本，在**所有实体自己涌现行为**——像矮人要塞的猫集体醉酒一样不可预测。世界只需要三根柱子——标签、元数值、元动作。所有行为从这三者的交互中涌现。

**核心赌注**：标签是存在，公理是物理——行为是涌现。引擎不认识"刀"——只认识 {hard>=4, shape:blade, edge:present}。引擎不认识"篝火"——只认识 {flammability>0.5, spark_source=true}。

---

## 二、设计哲学

### 2.1 存在论：标签即存在

**标签是存在的唯一载体。** 一个东西"是什么"，不来自它的名字，不来自它的类继承，不来自代码中任何地方的类型判断。一个东西是什么，**仅来自它携带的标签**。

```
type_name: "wolf"        <- 只是显示名，不是身份
tags: [predator, pack_hunter, body.large, ...]  <- 这才是狼的"存在"
```

**标签只定义"是什么"，不定义"怎么做"。** 标签是布尔质性的声明。数值不直接写在标签上——数值从标签组合推导。

**叠加域：标签的组合产生新意义。** 单个标签定义自身。多个标签同时存在时，它们的逻辑域叠加。"叠加域"不是代码显式定义的——它从公理和需求匹配中涌现。

```
cold + predator_nearby + can_craft + fire_nearby
-> 四者叠加 -> 需求匹配指向篝火
-> 不是标签显式说了"造篝火优先"，是需求叠加的必然结果
```

### 2.2 公理层：世界的物理引擎

四条公理构成世界的"允许"边界：

| 公理 | 回答 | 输入 | 输出 |
|---|---|---|---|
| compose | 这张卡能进这个格吗 | 格状态 + 卡 profile | Allowed / Denied |
| traverse | 这张卡能跨介质吗 | 卡 profile + 从介质 + 到介质 | Allowed / Denied |
| perceive | A 能感知 B 吗 | A profile + B profile + 距离 + 介质 | Detected / Undetected |
| transform | A 作用于 B 的能量转化 | A profile + B profile + 动作类型 | 能量收支 |

公理只回答"能不能"，不回答"该不该"。该不该——那是需求-满足匹配层的事。

**公理是纯函数：** 不依赖任何全局状态（除传入参数）、不产生副作用、输入相同则输出相同、每条公理独立于其他三条。这是消除耦合的核心机制。

**公理不判断"意图"：** compose 只说"格里已有狼 -> Denied"。羊为什么不去——那是需求层的安全需求压制了觅食需求。

### 2.3 决策层：哲学僵尸的"看似智能"

**没有"决策"，只有"对应"。** 传统 AI：感知 -> 分析 -> 决策 -> 规划 -> 执行。我们的模型：

```
标签（是什么）
  x
公理（允许什么）
  x
需求（缺什么）
  x
环境（有什么）
  |
匹配（什么行动同时覆盖最多急迫需求）
  |
执行（元动作序列）
```

没有"权衡利弊"的步骤。没有"计算得分"的步骤。只有：**当前状态 -> 需求激活 -> 环境匹配 -> 执行**。

**需求-满足匹配替代效用评分：** 需求在实体身上（标签定义: need:eat）。环境提供满足物（草满足 eat，篝火满足 warmth+safety）。匹配逻辑：找出覆盖最多急迫需求的行动。不需要对象"广播广告"——实体主动寻找，因为需求是它自己的本质。

**从匹配结果中，满足"足够好"条件的选项随机选一。** 避免机器感，也给玩家留下干预空间。

### 2.4 质量定义 = 与现实世界的同构度

> 基于 Executable Ontologies (Boldachev 2025)、Dwarf Fortress Simulation Principles (Adams 2015)、多智能体动力系统 (arXiv 2024-2025)。

本体论工程学标准：给定一个本体 OL 和一个理想本体 OC，OL 的质量 = OL 与 OC 之间的同构程度。

**保结构映射**：不是搬运现实的细节，而是搬运现实的结构原理。
- 温度、降雨、海拔、排水的交互 -> 决定生物群系（不是直接定义生物群系）
- 速度、重量、能量的交互 -> 决定撞击伤害（不是直接定义伤害值）
- 安全需求、制造能力、环境威胁的叠加 -> 决定行为选择（不是直接定义行为优先级）

**Tarn Adams (Dwarf Fortress):** "不要直接定义生物群系。分别处理温度、降雨、海拔、排水。这些场的交互决定最终结果——更自然，内部更自洽。"

**Tarn Adams 谈继承：** "当你声明一个类是一种物品，它把你锁在那结构里。如果有一个物品行为像 A 又像 B，在类继承里几乎不可能。把不同组件开关——就简单得多。" 我们的标签平铺架构天然避免此问题。

### 2.5 可执行本体论 (EO) 范式

不是问"智能体该怎么行动？"——而是问"这个世界里什么是真的？当条件满足时，什么变得可能？"行为从语义结构中涌现，而非被显式编程。我们的标签 -> 公理 -> 需求匹配 -> 元动作架构正是 EO 范式的实现。

LUDOCORE 系统（Smith et al. 2010）：用事件演算替代条件判断。Fluent（随时间变化的谓词）+ Event（离散事件）。新规则是纯粹的新增声明，不需要修改已有规则。

### 2.6 振荡的数学本质

Poincare-Bendixson 定理：二维动力系统的极限集只能是不动点或极限环。三维以上可能出现混沌。

**关键发现**：限制系统行为的不是维度（智能体数量），是**交互图的拓扑结构**。即使有任意多智能体，如果交互图稀疏（每个智能体只和少数邻居交互），极限行为仍然是简单的——只有极限环，没有混沌。

我们的实体已经有稀疏交互图：每个实体只感知曼哈顿范围 4-8 格内的邻居。这在数学上天然抑制混沌。

**破坏性振荡 vs 良性振荡：**

| 破坏性振荡 | 良性振荡 |
|---|---|
| 每 tick 在两个同分驱动间切换 | 需求饱和后自然切换到下一个需求 |
| 碰撞-位移-再碰撞的死循环 | Near-tie 随机选 -> 看起来"犹豫"但合理 |
| fed_today 锁死 -> 无目的空转 | 偶尔的策略摇摆 -> 丰富行为模式 |

"不稳定性、振荡和部分收敛不应仅被视为学习失败——它们包含关于策略探索的信息。" (Instability as Insight, 2025)

**防振荡法则：**
1. **行为惯性（Hysteresis）**：当前正在执行的元动作序列不被新决策打断（除非安全急迫度触发）
2. **执行冷却（Cooldown）**：同一元动作组合执行后有最小间隔
3. **急迫度阈值（Activation Threshold）**：新需求的急迫度必须 > 当前需求的 1.2 倍才切换
4. **熵正则化（Entropy Regularization）**：需求慢衰减后的自然切换 = 天然的探索机制

### 2.7 元本质完备性声明

**元本质 = 25 个元动作 + 元数值 A/B 层 + 标签。** 不需要第四根柱子。

- **元关系（Owns/Contains/ParentOf等）**不是新基元——是对应元动作执行后留下的**标签状态**
- **元状态（alive/dead/solid/liquid等）**不是新基元——是元数值跨越**阈值**的结果
- **三柱职能边界（铁律）**：标签描述形态（是什么），元数值描述强度（有多强），元动作描述变化（在做什么）。三柱不越界，不重复

---

## 三、世界设定

```yaml
world_setting:
  environment: "亚热带火山环境——火山土壤极肥沃，支撑高生物多样性"
  area_basis: "50人狩猎采集部落的生存面积 → ~100 km²（10km×10km）"
  grid: "64×64 = 4096 格（待实现，当前代码仍为 32×32）"
  cell_size: "~2.5 公顷（158m×158m）"
  species_scope: "中国历史上存在过的所有物种（含已灭绝）共存"
  stability: "高度稳定生态——不会轻易打破，只有极端干预才能破坏平衡"
  current_status: "16种动物是未完成子集——物种名单待补全"
  population_note: "当前捕食者/猎物比例失衡是因为物种不全，不是数字错误"
  building: "模块化——一种建筑占一格，不是一个家具一格"
```

### Z 轴与多层棋盘体系

```yaml
z_axis:
  position: "Entity { x: u8, y: u8, z: i16 }  // 负=地下, 0=地表, 正=空中"
  spatial_index: "3D 桶，每层独立 2D 棋盘"

  layer_types:
    abstract_channel:
      z_range: "命名层之间"
      display: "进度条 + 产出表 + 随机发现事件"
      permission: "挖/取/运输，不可建基地"
    named_board:
      z_range: "地质年代/大气层分界"
      display: "完整棋盘（36x24）"
      permission: "全部权限"
    sky_platform:
      z_range: "任意高度"
      display: "完整棋盘"
      permission: "需悬浮能力+持续动力消耗"

  activation:
    - "命名层 -> 触及层边界 -> 自动激活该层棋盘"
    - "放置基地卡 -> 任意 Z 坐标手动激活棋盘"

  travel_rules:
    same_z: "曼哈顿移动"
    cross_z: "需要运输卡（竖梯/电梯/火箭等）-> Z 层间不自动连通"
    sky: "位置 (x,y,z) 不自动连通地面 (x,y,0)"

  underground_layers:  # 地质年代（深度为游戏压缩值）
    - { name: "更新世", depth: "100m", resource: "猛犸化石、冰期沉积" }
    - { name: "上新世", depth: "300m", resource: "铁矿、褐煤" }
    - { name: "中新世", depth: "500m", resource: "石油层" }
    - { name: "白垩纪", depth: "800m", resource: "白垩岩、恐龙化石" }
    - { name: "石炭纪", depth: "1200m", resource: "煤矿层" }
    - { name: "寒武纪", depth: "2000m", resource: "页岩、三叶虫化石" }
    - { name: "前寒武纪", depth: "3500m", resource: "铁矿石、花岗岩" }

  sky_layers:  # 基于 NOAA 2026 标准大气
    - { name: "对流层", height: "0-12km", env: "正常~补充氧" }
    - { name: "平流层", height: "12-50km", env: "需增压服" }
    - { name: "中间层", height: "50-85km", env: "太空级维生" }
    - { name: "热层", height: "85-600km", env: "航天器" }
    - { name: "外大气层", height: "600km+", env: "轨道空间" }

  underground_vs_sky:
    underground: { support: "下方稳定链->地表", energy: "无（静态结构）", movable: "不可", travel: "物理连通（楼梯/竖井）", hover: false }
    sky: { support: "悬浮动力持续消耗", energy: "持续消耗（燃料/能源）", movable: "可移动（漂移/重定位）", travel: "运输链路（电梯/火箭）", hover: true }

  inactive_layer_handling:
    current: "完整 tick 模拟"
    non_current: "概率积分估计（时间 x 能力 x 资源 -> 最可能结果）"
    non_activated: "不存在（不存数据，不消耗性能）"
```

---

## 四、抽象体系

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

---

## 五、身体模拟系统（齐平矮人要塞深度）

```yaml
body_simulation:
  principle: "标签驱动+数据定义，零硬编码。加新物种=选模板+填标签"
  depth: "组织层（皮肤/脂肪/肌肉/肌腱/骨骼/器官）。细胞以下不模拟"
  
  body_templates:  # 按 body_plan 标签自动选择模板
    quadruped: "四足（鹿虎象犀猪牛豺熊獭鳄）"
    biped: "双足（人类/虎人）"
    avian: "鸟形"
    serpentine: "蛇形（蟒蛇）"
    fish: "鱼形（鲤鱼/鲶鱼）"
    insectoid: "虫形（容纳型）"
    plant: "植物型"

  tissue_layers:  # 每个身体区域从外到内的层
    - skin: "第一道防线。defense:armor 修改此层强度"
    - fat: "能量储备+缓冲+保温。body_fat 池决定厚度"
    - muscle: "力量来源。损伤=该区域力量下降"
    - tendon: "连接肌肉和骨骼。断裂=肌肉无法作用于关节"
    - bone: "结构支撑。骨折=结构失效"
    - organ: "核心功能。穿透到此=器官损伤"

  body_systems:  # 每个器官属于一个系统，系统损伤有全局后果
    circulatory: "心脏+血管。心脏破=即死。出血=blood_volume下降"
    respiratory: "肺x2+气管+鳃(鱼)。窒息/溺水"
    nervous: "脑+脊髓+神经。瘫痪/昏迷/即死"
    digestive: "胃肠+肝+肾。穿透=感染。肝伤=代谢崩溃"
    reproductive: "睾丸x2/卵巢x2+子宫+乳腺。损伤=不育/不能哺乳"
    sensory: "眼x2+耳x2+鼻+口舌+侧线(鱼)。损伤=感知通道降级"
    musculoskeletal: "各部位骨骼+肌肉+肌腱。骨折/肌肉撕裂/肌腱断裂"
    integumentary: "皮肤+毛发/鳞片/羽毛。破裂=出血+感染风险"

  entity_pools:  # 全身级别的宏观池（不是每个部位独立跟踪）
    blood_volume: "1.0=正常, <0.5=虚弱, <0.2=休克, 0=死。bleeding伤口每tick扣减"
    body_fat: "0.0=极瘦, 1.0=肥胖。决定冬眠能力/饥饿耐受/繁殖条件"
    stamina: "短期体力。sprint/strike消耗，rest恢复。0=被迫休息"
    pain: "痛觉累积。>阈值1=迟缓, >阈值2=昏迷"
    consciousness: "清醒/眩晕/昏迷/死亡。从pain+blood_loss+brain_damage推导"

  missing_systems_to_implement:  # 补齐矮人要塞深度
    - pain_consciousness: "痛觉累积->昏迷。每个受伤部位产生痛觉值"
    - healing_scarring: "bleeding->止血->healing->scarred。severed不可愈合"
    - stance_prone: "四足需>=3腿站立。<3=倒地=不能移动+防御x0.3"
    - grasp_hold: "bite咬住=Constrain。jaw伤=松嘴。hand伤=掉工具"
    - fatigue_stamina: "高强度行动消耗stamina。0=被迫休息"
    - fur_feather_scale: "最外层覆盖。影响保温+防御"

  body_process_framework:  # 统一的身体过程框架（数据驱动）
    schema: |
      BodyProcess = {
        trigger: age/season/contact/per_action/random/injury_present,
        target: 具体body_part或organ或system,
        effect: degrade/grow/shed/infect/heal,
        rate: f32,
        reversible: bool,
        transmissible: bool,
        immunity_after: bool
      }
    examples:
      - "tooth_wear: trigger=per_consume, target=teeth, effect=degrade, rate=0.001"
      - "antler_shed: trigger=season:winter, target=antler, effect=shed"
      - "aging_vision: trigger=age>0.7, target=eye, effect=degrade, rate=0.01"
      - "plague: trigger=contact, target=全身, effect=degrade, rate=0.3, transmissible=true"
      - "wound_infection: trigger=injury:bleeding_present, target=wound_site, effect=infect"
      - "healing: trigger=time, target=injured_part, effect=heal, rate=0.05"
      - "cancer: trigger=random(age_weighted), target=random_organ, effect=degrade"
      - "pregnancy: trigger=mating_success, target=uterus, effect=grow, duration=species_specific"
    principle: "加新疾病/衰老效果/季节变化 = 加一行数据，不写代码"

  species_override:  # 种内差异化（不改模板，改参数）
    principle: "body_plan选模板，标签改参数，overrides加特殊部位"
    examples:
      - "犀牛: defense:armor -> skin.yield_strength x5"
      - "大象: 额外body_part:trunk（鼻=第五肢）"
      - "鹿(公): 额外body_part:antler（季节性）"
      - "穿山甲: skin.material=keratin_scale, yield_strength x8"
      - "蟒蛇: 只有右肺功能(左肺退化), 热感应窝(sense:infrared)"

  aging_interaction: "衰老不是独立过程——是所有BodyProcess的速率乘以age_modifier"
  
  hormone_abstraction: "不模拟激素浓度——通过need/personality/state标签代替效果"
```

---

## 六、生态系统

### 植物系统

```yaml
plant_distribution:
  principle: "植物能不能在某格生长 = 格子环境条件 in 植物耐受范围"
  matching_tags:
    cell_conditions: "水分(淹水/湿润/干燥) x 光照(全日照/半阴/全阴) x 海拔(低/中/高)"
    plant_tolerance: "flooding_tolerance x drought_tolerance x shade_tolerance"
  rule: "条件匹配->能长。不匹配->不能长。不硬编码'莲花只能在水里'"
  initial_generation: "方案A——按地形环预设合理分布，之后传播系统自维持"

plant_propagation:
  principle: "植物怎么繁衍 = dispersal标签决定传播方式 + growth标签决定速度"
  dispersal_methods:
    wind: "种子随风飘散，半径3-10格，方向随机"
    animal: "果实被吃->动物排泄->种子跟着动物路线走"
    water: "种子顺水流漂->沿水域扩散"
    gravity: "种子直接落脚下->只扩散到相邻格->形成密集群落"
    explosive: "豆荚爆裂->半径1-2格"
  growth_rate:
    fast: "草类——几天恢复"
    medium: "灌木——几十天"
    slow: "乔木——几百天"
  succession: "涌现效果：砍光森林->草先到(wind快)->灌木来(animal中)->树苗长(slow)->成熟森林"
  player_interaction: "抽=从格子取种子卡。叠=把种子放到新格子->条件匹配->发芽"

plant_storage:
  principle: "植物是格子上的数量卡，不是独立实体"
  model: "cell_resources: HashMap<(x,y), HashMap<plant_type, quantity>>"
  consume: "动物吃->quantity -= 1"
  regrow: "传播系统->符合条件的格子->quantity += 1"
```

### 繁殖系统

```yaml
reproduction:
  prerequisites:  # 全部从标签/运行时状态推导
    - "年龄 > 性成熟年龄（从 growth 标签推）"
    - "body_fat > 0.3（营养不良不繁殖）"
    - "生殖器官完好（ovary/testicle 未损伤）"
    - "state:pregnant 不存在"
    - "不在 state:starving"
    - "季节匹配（部分物种季节性繁殖）"

  mating:
    action: "MetaAction::Reproduce { partner }"
    condition: "同物种 + 异性 + 成年 + 邻格 + 双方繁殖需求激活"
    social_variation:
      solitary: "只在繁殖季寻找，交配后分开"
      pair: "固定配偶长期在一起"
      herd: "群内交配"
      pack: "通常只有头领繁殖"

  gestation:
    principle: "用真实数据标签，不用公式（妊娠期无可靠万能公式）"
    implementation: "每种动物在 card_defs.ron 标注 gestation_days: N"
    during_pregnancy: "营养需求增加、移动速度降低、不能再次交配"
    egg_layer: "先产卵（卵是一张卡）-> 孵化期 -> 幼崽出壳"

  birth:
    few_offspring: "1-2只（K策略，大型动物）"
    many_offspring: "3-8只（r策略，小型/鱼类）"
    fish_special: "产卵数百->概率存活率->实际幼鱼少"

  parental_care:
    with_care: "幼崽跟随母亲。mammary完好+body_fat>0.2->哺乳->存活"
    without_care: "出生即独立（鱼类、多数爬行类）"
    mammary_damage: "乳腺损伤->不能哺乳->幼崽存活率降低"

  inheritance:
    same_as_parents: "物种标签、body_plan、diet、habitat、capability"
    random: "personality（随机）、individual_modifier（正态分布）、sex（50/50）"
    not_inherited: "injury（伤疤不遗传）、experience（要靠Teach传授）、body_fat（从零积累）"
```

### 捕食链

```yaml
predation_chain:
  principle: "需求引擎的重新规划机制天然形成捕猎循环，不需要写循环逻辑"
  
  carnivore_knowledge:
    decomposition: "[Acquire(找animal), Act(Strike), Act(Consume)]"
    vs_current: "当前只有[Acquire, Consume]——缺Strike步骤"
  
  consume_animal_requirement:
    rule: "吃动物必须目标is_corpse=true（必须先杀死才能吃）"
    vs_current: "当前can_digest允许吃活的——需修改"
  
  natural_loop:
    - "虎饿了->制定计划[找猎物,Strike,Consume]"
    - "Strike->鹿HP减少但没死->Consume失败(鹿还活着)"
    - "计划失败->但还饿->重新制定计划->继续追同一只鹿"
    - "重复直到：鹿死了(Consume成功) 或 鹿逃了(Acquire失败->换目标)"
  
  give_up_conditions:
    - "猎物超出感知范围（跑太远）"
    - "猎物进入不可达地形（跳进水里）"
    - "自己stamina耗尽（跑不动了）"
    - "猎物太强（反击造成严重injury）"
```

### 腐烂与骨骼

```yaml
decomposition:
  formula: "积温日 ADD = sum max(daily_temp - 0C, 0) / mass_factor"
  mass_factor: "(mass_kg / 10.0)^0.25 — 大尸体腐烂更慢"
  principle: "万能公式——温度越高腐烂越快，冰冻=不烂。完全同构"

  soft_tissue_stages:
    fresh:        { add_threshold: 0,    desc: "刚死，外观无变化" }
    bloating:     { add_threshold: 100,  desc: "气体积累，发臭" }
    active_decay: { add_threshold: 200,  desc: "大量质量流失，吸引食腐动物" }
    advanced:     { add_threshold: 500,  desc: "软组织基本消失" }
    skeleton:     { add_threshold: 1000, desc: "只剩骨骼->产生骨骼资源卡" }

  bone_weathering:  # 骨骼=70%矿物质，本质是石头风化速度
    intact:       { add_threshold: 1000,  desc: "白骨暴露" }
    cracking:     { add_threshold: 5000,  desc: "表面裂纹" }
    fragmenting:  { add_threshold: 15000, desc: "断裂分解" }
    dissolved:    { add_threshold: 30000, desc: "完全融入土壤->肥力增加" }

  bone_as_resource:
    - "骨骼是资源卡，留在格子上"
    - "玩家可'抽'出来->做骨针/骨刀/骨笛"
    - "玩家可'砸'碎->骨粉->撒地上当肥料"
    - "食骨动物可 Consume 骨头（补钙）"
    - "玩家可'叠'到土里->埋入->加速分解+增加肥力"
    - "火烧->煅烧骨（更硬的材料，不是加速分解）"

  nutrient_cycle:
    - "尸体腐烂->养分回到土壤->格子 soil_fertility 增加->植物生长加速"
    - "碳循环闭合：植物->动物->土壤->植物"
    
  scavenger_interaction:
    - "active_decay 阶段->产生嗅觉信号->Perceive:smell->食腐动物感知"
    - "diet:scavenger 动物->Consume 尸体软组织部分"
    - "skeleton 阶段->无软组织可吃->食腐动物不再来"
```

### 温度系统

```yaml
planned_runtime_dimensions:
  - id: ambient_temperature
    type: "运行时（每格每 tick 变化）"
    source: "季节 + 海拔 + 水深 + 昼夜"
    priority: "必须实现——腐烂/代谢/冬眠/植物生长全依赖温度"
    affects_everything:
      - ectotherm_metabolism: "Q10=2.5: 温度每降10C代谢减半"
      - endotherm_energy_cost: "越冷->维持体温越费能量->饿得更快"
      - movement_speed: "变温动物冷了动不了"
      - hibernation_trigger: "温度降到阈值->metab:torpor动物进入冬眠"
      - plant_growth: "冬天停止生长->食物减少"
      - water_freeze: "鱼活动空间缩小"
      - decomposition: "ADD公式核心变量——温度决定腐烂速度"
      - disease_spread: "温暖潮湿->传染病传播加速"
    formula: "B_actual = B0 x Q10^((T - T_ref) / 10)"
    decomposition_formula: "ADD_per_tick = max(temp - 0, 0) / TICKS_PER_DAY"
    tags_involved: [thermo:endotherm/ectotherm, metab:torpor, habitat:aquatic]
    status: "未实现——必须优先实现"

  - id: body_condition
    type: "运行时（每个实体，随进食/消耗变化）"
    source: "进食量 - 代谢消耗 的累积"
    affects_everything:
      - fasting_endurance: "胖动物比瘦动物多活80%（Trondrud 2021 实测）"
      - hibernation_ability: "脂肪不够->不敢冬眠->冬天饿死"
      - reproduction: "体况太差->不繁殖"
      - combat_power: "饥饿动物战力下降"
      - movement_speed: "太瘦跑不动，太胖也慢——有最优区间"
      - offspring_survival: "母亲体况差->奶水不足->幼崽死亡率高"
    formula: "body_condition = fat_reserve / lean_mass（0.0=极瘦, 1.0=极胖）"
    tags_involved: [body_size, metab, repro]
    status: "未实现——优先级高，连接'吃了多少'和'能做什么'"
```

---

## 七、交互系统

玩家只有两个万能操作（上帝之手，不是元动作）：

- **Act（砸/Smash）** = 拖拽卡 A 到卡 B 上释放（左键）
- **Arrange（叠/Stack）** = 右键操作

结果由引擎根据 A 和 B 的标签+材质自动推断对应元动作：
```
石刀 + 狼 -> Strike(harm)
石刀 + 燧石 -> Strike(shape)
树枝 + 树枝 -> Combine
石刀 + 鹿尸 -> Strike(cut) + Consume
```

**手牌五种操作：**

| 操作 | 方式 | 效果 |
|---|---|---|
| **砸** | 左键拖拽碰撞 | 攻击（HP->0）或加工（2砸=出产品）。不可连续蹭，拉开再碰才算 |
| **叠** | 右键放置 | 改变本质——组合变新卡/加标签。不兼容时弹回 |
| **拿** | 从世界取到手牌 | 卡从格子移入手牌区 |
| **抽** | 从容器/格子抽取 | 从格子取种子卡 / 从容纳物中取出内容 |
| **放** | 从手牌放到世界 | Release 到指定格子，需 compose 验证 |

**玩家的每一个操作都有物理反馈——不存在"砸了 500 下没结果"。** 硬度差决定物理必然性。

**AI 的操作同源：** AI 攻击 = 砸，AI 搬运 = 幽灵。

---

## 八、公式 vs 数据原则

```yaml
formula_vs_data:
  use_formula:  # 有跨物种一致的物理/热力学规律
    - "饥饿致死天数: 100 x M^(1-beta) / B0 x torpor_mult（热力学）"
    - "攻击力: impact_force(mass, velocity, area, hardness)（物理）"
    - "代谢率: baseline_energy(mass, metab_rate)（Kleiber定律）"
    - "体型->质量: estimate_mass_from_tags（等比例缩放）"
    - "营养衰减: metab_rate / (TICKS_PER_DAY x TICK_SECONDS)（能量守恒）"
  
  use_real_data:  # 物种差异太大，无万能公式
    - "妊娠期: gestation_days 标签（查资料填）"
    - "性成熟年龄: maturity_days 标签"
    - "最大寿命: max_age 标签"
    - "每胎数量: litter_size 标签"
  
  principle: "能用公式推的用公式（同构推导）。不能的用真实数据（同构记录）。两种都是同构。"
```

---

## 九、关联映射

```yaml
cross_references:
  # 标签 -> 元数值
  tag_to_value:
    body_size -> estimate_mass_from_tags
    metab -> decay_rate, baseline_energy
    cognition -> combat_proficiency上限
    lifespan -> max_age, age_strength_factor

  # 标签 -> 元动作
  tag_to_action:
    diet -> Consume
    capability + body_size -> Strike
    body_plan + capability + body_size -> Move
    repro + social -> Reproduce
    cognition -> Decide

  # 标签 -> 标签
  tag_to_tag:
    diet + foraging + body_size -> 攻击策略
    social -> sexual_dimorphism(派生)
    reproduction + growth + lifespan -> 年龄曲线
```

---

## 十、实现规则

### 禁止事项

**5.1 禁止 type_name 硬编码**
```rust
// BAD
if entity.type_name == "wolf" { ... }
match entity.type_name.as_str() { "grass" => ... }

// GOOD
if card_has_tag(def, "predator") { ... }
if card_has_capability(def, "capability.move") { ... }
```

**5.2 禁止魔法数字**
```rust
// BAD
let speed = 0.25;  // 这是什么？为什么是 0.25？
need.current = need.current.min(15.0);  // 为什么是 15？

// GOOD
let speed = base_speed / body_size_modifier(entity);  // 追溯到 size 标签
need.current = need.current.min(eat_satiety_threshold(entity));  // 追溯到 need 定义
```

**5.3 禁止隐式行为锁**
```rust
// BAD
if entity.fed_today && drive.condition_fed {
    continue;  // 跳过这个驱动——但为什么跳过？多久？什么条件下恢复？
}

// GOOD
// 需求-满足匹配自然处理: 吃饱 -> eat 需求不激活 -> 觅食不触发
// 不需要显式的锁
```

**5.4 禁止代码层面的 if-else 行为链**
```rust
// BAD
if can_hunt { hunt(); }
else if can_forage { forage(); }
else if can_eat { eat(); }
// 加新行为 = 加新的 else if = 顺序耦合

// GOOD
let actions = match_actions(activated_needs, environment);
execute_best_match(actions);
// 加新行为 = 加新标签 + 加新元动作组合 = 无代码改动
```

### 审查纪律

> 来自实际犯错的经验总结。

**铁律一：签名变更 -> grep 全库调用方，逐个验证参数匹配。** 不止看函数体——看每个调用方。GDScript/动态语言不做类型检查，错参不会编译报错。

**铁律二：每次审查至少模拟一个边界条件。** 扫描半径边缘、安全区边界、首 tick 空值——这些才是 bug 的栖息地。"正常值"永远不会出问题。

**铁律三：审查时看 diff。** 对每行改动问"这是改了什么、它连着什么"。重构代码的"正确性"不在它本身——在它和旧代码的差分里。

### Handoff 纪律

- 每条改动 = 文件路径 + 函数名 + 现状 + 改为。**不写"应该""期望"。**
- 任何架构级 handoff 第一行必须写铁律——"所有决策从标签推导，零硬编码条件，禁用任意魔法数字"
- handoff 中代码模板的最后一行必须写"所有调用方同步验证"

### UI 纪律

- **玩家不读代码。后端机制的存在对玩家而言唯一的证据是视觉呈现。没有可视化 = 这个机制不存在。**
- 任何新卡新机制，必须写外观 UI（地图上看得到）+ 选中 UI（点上去查得到）

### 设计纪律

- **如果一个机制需要给自然行为加"驱赶"类补丁，说明自然行为本身没写对。** 反向追溯——不是狼怎么进门，是兔子为什么没逃。
- **不要给设计缺陷打代码补丁。** 补丁套补丁会产生竞态和状态混乱。
- 别加复杂度，先让基础跑通。

### 不变量

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

### 变更流程

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

## 十一、当前实现状态

> 同步自 FACT.md 三柱表。2026-06-18。

**已贯通公理：** Consume (can_digest) / Strike (impact_force) / Move (部分)

**架构铁律（运行中）：**
- 标签是存在，公理处理交互。禁止物种硬编码（type_name 匹配）
- compose 默认拒绝进入已占格，仅 corpses 和 incorporeal 例外
- 1 day = 420 tick
- 曼哈顿移动：任何移动只能单轴
- 所有代码改动通过 handoff，cargo test 全 PASS + smoke PASS

**已完成：**
- 三柱贯通（Consume/Strike/Move 公理）
- 饥饿致死（热力学公式）
- 时间尺度同构（2100 tick/天）
- 手牌框架（五种操作 + 快速 tick）
- 架构修复（150+ 违规清零）
- Harness 体系（CLAUDE.md + hooks + CI + ontology）
- 177 个测试，0 失败

**当前进行：**
- 策划路线敲定（生态自洽、身体模拟、物种名单）
- 格子 64x64 迁移（待实现）
- 格子资源模型（待实现）

**下一步：**
- 温度系统实现
- 身体模拟系统实现
- 植物传播系统实现
- 捕食链补完（Strike步骤 + 尸体要求）
- 繁殖系统实现
- 视觉层搭建

---

> **维护规则**：改代码前先看本体。本体和代码不一致时，以本体为准——要么修正代码，要么修正本体并更新变更记录。
