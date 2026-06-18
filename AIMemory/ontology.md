# 桃花源抽象组件本体

> **单一真相源。** 所有标签维度、元数值、元动作在此定义。
> **源码级。** 和代码一起版本控制。改代码前先改本体。
> **机器可读。** 结构化 YAML，AI 和脚本均可解析。

---

## 世界设定（策划确认）

```yaml
world_setting:
  environment: "亚热带火山环境——火山土壤极肥沃，支撑高生物多样性"
  area_basis: "50人狩猎采集部落的生存面积 → 约16 km²"
  grid: "32×32 = 1024格"
  cell_size: "约125m×125m ≈ 1.5公顷"
  species_scope: "中国历史上存在过的所有物种（含已灭绝）共存"
  stability: "高度稳定生态——不会轻易打破，只有极端干预才能破坏平衡"
  current_status: "16种动物是未完成子集——物种名单待补全"
  population_note: "当前捕食者/猎物比例失衡是因为物种不全，不是数字错误"
```

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

## 身体模拟系统（策划已确认——齐平矮人要塞深度）

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
    respiratory: "肺×2+气管+鳃(鱼)。窒息/溺水"
    nervous: "脑+脊髓+神经。瘫痪/昏迷/即死"
    digestive: "胃肠+肝+肾。穿透=感染。肝伤=代谢崩溃"
    reproductive: "睾丸×2/卵巢×2+子宫+乳腺。损伤=不育/不能哺乳"
    sensory: "眼×2+耳×2+鼻+口舌+侧线(鱼)。损伤=感知通道降级"
    musculoskeletal: "各部位骨骼+肌肉+肌腱。骨折/肌肉撕裂/肌腱断裂"
    integumentary: "皮肤+毛发/鳞片/羽毛。破裂=出血+感染风险"

  entity_pools:  # 全身级别的宏观池（不是每个部位独立跟踪）
    blood_volume: "1.0=正常, <0.5=虚弱, <0.2=休克, 0=死。bleeding伤口每tick扣减"
    body_fat: "0.0=极瘦, 1.0=肥胖。决定冬眠能力/饥饿耐受/繁殖条件"
    stamina: "短期体力。sprint/strike消耗，rest恢复。0=被迫休息"
    pain: "痛觉累积。>阈值1=迟缓, >阈值2=昏迷"
    consciousness: "清醒/眩晕/昏迷/死亡。从pain+blood_loss+brain_damage推导"

  missing_systems_to_implement:  # 补齐矮人要塞深度
    - pain_consciousness: "痛觉累积→昏迷。每个受伤部位产生痛觉值"
    - healing_scarring: "bleeding→止血→healing→scarred。severed不可愈合"
    - stance_prone: "四足需≥3腿站立。<3=倒地=不能移动+防御×0.3"
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
      - "犀牛: defense:armor → skin.yield_strength ×5"
      - "大象: 额外body_part:trunk（鼻=第五肢）"
      - "鹿(公): 额外body_part:antler（季节性）"
      - "穿山甲: skin.material=keratin_scale, yield_strength ×8"
      - "蟒蛇: 只有右肺功能(左肺退化), 热感应窝(sense:infrared)"

  aging_interaction: "衰老不是独立过程——是所有BodyProcess的速率乘以age_modifier"
  
  hormone_abstraction: "不模拟激素浓度——通过need/personality/state标签代替效果"
```

## 待实现的运行时维度（策划已确认重要）

```yaml
planned_runtime_dimensions:
  - id: ambient_temperature
    type: 运行时（每格每 tick 变化）
    source: "季节 + 海拔 + 水深 + 昼夜"
    affects_everything:
      - ectotherm_metabolism: "Q10=2.5: 温度每降10°C代谢减半"
      - endotherm_energy_cost: "越冷→维持体温越费能量→饿得更快"
      - movement_speed: "变温动物冷了动不了"
      - hibernation_trigger: "温度降到阈值→metab:torpor动物进入冬眠"
      - plant_growth: "冬天停止生长→食物减少"
      - water_freeze: "鱼活动空间缩小"
    formula: "B_actual = B₀ × Q10^((T - T_ref) / 10)"
    tags_involved: [thermo:endotherm/ectotherm, metab:torpor, habitat:aquatic]
    status: "❌ 未实现——优先级高，影响所有变温动物行为和季节循环"

  - id: body_condition
    type: 运行时（每个实体，随进食/消耗变化）
    source: "进食量 - 代谢消耗 的累积"
    affects_everything:
      - fasting_endurance: "胖动物比瘦动物多活80%（Trondrud 2021 实测）"
      - hibernation_ability: "脂肪不够→不敢冬眠→冬天饿死"
      - reproduction: "体况太差→不繁殖"
      - combat_power: "饥饿动物战力下降"
      - movement_speed: "太瘦跑不动，太胖也慢——有最优区间"
      - offspring_survival: "母亲体况差→奶水不足→幼崽死亡率高"
    formula: "body_condition = fat_reserve / lean_mass（0.0=极瘦, 1.0=极胖）"
    tags_involved: [body_size, metab, repro]
    status: "❌ 未实现——优先级高，连接'吃了多少'和'能做什么'"
```

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
