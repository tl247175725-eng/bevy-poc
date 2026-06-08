# 方寸商国：桃花源记 — 设计审查与思维导图

> 日期：2026-06-02 | 版本：v3.0-A 封板审查

---

## 一、项目骨架

```
方寸商国：桃花源记
├── 两大系统
│   ├── 世界规则结构化标签机（WorldRules）
│   └── 卡牌封装（CardDB → 运行时状态 → 规则参与）
├── 一个约束
│   └── 六维评价（identity / material_form / capability / relation_domain / action / rule_modifier）
└── 一个机制
    └── 能力驱动的行为吸附（capability → behavior pipeline）
```

---

## 二、架构管线全景

```
新卡注册
  card_db.gd ──→ CARD_CAPABILITIES ──→ CardRuleAudit（六维归类）
       │                    │
       ▼                    ▼
  ecosystem_behavior_key()  ← 读 capability 签名，返回行为 key
       │
       ▼
  EcosystemTickRegistry.tick_card()
       │
       ├─ herbivore_grazer ──→ HerbivoreGrazerBehavior（兔/鹿/羊/野牛/幼体）
       ├─ predator_den ──→ PredatorDenBehavior（狼/狐，幼体/窝内/迁出）
       ├─ cover_forager ──→ FieldMouseBehavior（田鼠）
       ├─ traveler / mushroom_farmer / taoyuan（社交域，独立入口）
       │
       └─ CapabilityBehaviorPipeline（捕食者有序片段）
            flee → fire → den_work → scavenge → hunt
```

### 管线片段准入

| 片段 | 准入条件 | 截断 |
|------|---------|------|
| flee | mesopredator tag + 近玩家/狼 | true |
| fire | 近篝火 + 未叼肉 | true |
| den_work | can_return_home | true |
| scavenge | capability.scavenge + 有窝 | true |
| hunt | 全部能力捕食者 | 末段 |

---

## 三、卡片总览

### 3.1 动物（自动生态卡）

| 卡名 | 标签 | 能力 | 行为 key | 六维 |
|------|------|------|---------|------|
| 狼 wolf | being, animal, predator, autonomous | hunt, move_quick, return_home, care_child, reproduce | predator_den | ✅ |
| 狼崽 wolfCub | being, animal, predator, juvenile | move_juvenile, follow, grow, be_cared_for, return_home | predator_den | ✅ |
| 狐 fox | being, animal, carnivore, mesopredator, scavenger, cover_user | hunt, move_quick, escape_fast, escape_cover, use_cover, scavenge, return_home, care_child, reproduce | predator_den | ✅ |
| 狐崽 foxCub | being, animal, carnivore, cover_user, juvenile | move_juvenile, follow, grow, be_cared_for, return_home | predator_den | ✅ |
| 猞猁 lynx | being, animal, mesopredator, cover_user | hunt, move_normal, use_cover | mesopredator_hunt | ⚠ 缺 carnivore |
| 羊 sheep | being, animal | forage, move_normal, reproduce, flee, be_hunted | herbivore_grazer | ⚠ 缺 herbivore |
| 羊羔 lamb | being, animal, juvenile | follow, grow, be_cared_for | herbivore_grazer | ⚠ 缺 herbivore |
| 鹿 deer | being, animal, herbivore, wildPrey, largePrey | forage, move_normal, escape_fast, flee, be_hunted, reproduce | herbivore_grazer | ✅ |
| 鹿崽 deerFawn | being, animal, herbivore, wildPrey, juvenile | follow, grow, be_cared_for | herbivore_grazer | ✅ |
| 兔 rabbit | being, animal, smallHerbivore | forage, move_quick, escape_small, flee, be_hunted | herbivore_grazer | ✅ |
| 野牛 bison | being, animal, herbivore, largePrey | forage, move_slow, flee, be_hunted | herbivore_grazer | ✅ |
| 田鼠 fieldMouse | being, animal, smallPrey, omnivore.small, burrower, cover_user | forage, move_fast, escape_small, escape_cover, use_cover, reproduce, flee, be_hunted | cover_forager | ✅ |
| 田鼠崽 fieldMousePup | being, animal, smallPrey, cover_user, juvenile | follow, grow, be_cared_for | cover_forager | ✅ |

### 3.2 建筑

| 卡名 | 标签 | 能力 | 说明 |
|------|------|------|------|
| 狼窝 wolfDen | den, shelter, animalHome | define_domain, bond_to_actor, support_reproduce, provide_service | 干草筑窝，容量 3，砸毁产干草 |
| 狐窝 foxDen | den, shelter, animalHome, den.candidate.fox | define_domain, bond_to_actor, support_reproduce, provide_service | 占灌木转化，砸毁产灌木 |
| 篝火 fire | heat, environment, camp.anchor, organize.locked | define_domain, protect, transform_input, provide_service | 营地锚点，狼/狐惧火 |
| 草棚 hut | shelter, structure, home, camp.fire_bond | bond_to_domain, provide_service, shelter, expand_domain | 玩家休息 |
| 桌子 table | businessUnit, structure, camp.fire_bond | bond_to_domain, provide_service, attract_actor, trade, dining | 旅人交易 |
| 蘑菇棚 mushroomGreenhouse | shelter, structure, mushroomFarm | define_domain, produce_resource, attract_actor, provide_service | 蘑菇农工作 |
| 山 mountain | rooted, environment, stoneSource, source.stone | define_domain, produce_resource | 石头资源点 |
| 树林 tree | rooted, forest, woodSource, source.lumber, source.twig | define_domain, produce_resource, regenerate, respond_to_tool | 木材资源点 |

### 3.3 植物/资源/物品

| 类别 | 卡名 | 标签 | 说明 |
|------|------|------|------|
| 植物 | 活草 grass | cover, grass, foodSource | 河岸再生，被羊/鹿/兔吃，藏田鼠 |
| 植物 | 灌木 bush | cover, bush, cover.small, regenerates, shelter, microfauna.host, den.candidate.fox | 再生，藏田鼠，狐窝前身，产浆果 |
| 植物 | 干草 dryGrass | cover, dry, fiber, material | 狼窝建材 |
| 木材 | 木头 wood | wood, fuel, material.lumber | 燃料、建材 |
| 木材 | 树枝 twig | wood, fuel, material.lumber, naturalDrop | 树掉落 |
| 木材 | 木构件 woodStruct | structure, material | 建筑中间件 |
| 木材 | 湿木头 wetWood | wood, wet, material | 蘑菇农产出 |
| 木材 | 菇木 mushroomWood | wood, mushroomSource, material | 蘑菇生长 |
| 石材 | 石头 stone | hard, blunt, material.stone | 基础材料 |
| 石材 | 碎石 shard | hard, material, material.shard | 敲击产出 |
| 石材 | 三角碎石 tri | hard, sharp, toolHead | 工具头 |
| 石材 | 正方碎石 square | hard, toolHead | 工具头 |
| 工具 | 石刀 knife | sharp, tool, weapon, commodity | 狩猎/屠宰 |
| 工具 | 长矛 spear | sharp, tool, weapon, commodity | 狩猎/防御 |
| 工具 | 斧头 axe | sharp, tool, weapon, commodity | 伐木 |
| 工具 | 锄头 hoe | tool, commodity | 功能工具 |
| 工具 | 锤子 hammer | hard, blunt, tool, commodity | 敲击 |
| 容器 | 空桶 bucket | container, container.water | 装水 |
| 容器 | 水桶 waterbucket | water, container | 一桶水 |
| 容器 | 半桶 halfbucket | water, container | 半桶水 |
| 货币 | 铜钱 coin | currency, anchor, copper | 营地布局锚点 |
| 货币 | 铜块 copperBlock | copper, material | 铜原料 |
| 货币 | 铜饰 copperCraft | consumable, craft, copper, novelty, commodity | 商品 |
| 燃料 | 木炭 charcoal | fuel | 燃料 |
| 食物 | 生肉 (5种) sheepMeat/rabbitMeat/deerMeat/wolfMeat/humanMeat | food.raw, raw | 狼/狐捕猎产出 |
| 食物 | 熟肉 cookmeat | food.edible, cooked, commodity | 烹饪成品 |
| 食物 | 浆果 berry | food.edible, basic | 灌木产出 |
| 食物 | 蘑菇 mushroom | food.edible, basic | 菇木产出 |
| 材料 | 草绳 grassRope | fiber, craftPart, material | 草加工品 |
| 尸体 | 尸体 (4种) sheepCorpse/deerCorpse/wolfCorpse/playerCorpse | corpse, organic | 屠宰产肉，狐清腐 |

### 3.4 社交域（非生态管线）

| 卡名 | 标签 | 能力 | 说明 |
|------|------|------|------|
| 桃源长老 taoyuanElder | being, human, taoyuan, observer, elder | move, observe, social_boundary | 远观营地，社交距离 6 |
| 桃源采集者 taoyuanForager | being, human, taoyuan, observer, forager | move, observe, social_boundary | 观察取水，社交距离 4 |
| 桃源青年 taoyuanYouth | being, human, taoyuan, observer, youth | move, observe, social_boundary | 好奇张望，社交距离 3 |
| 旅人 traveler | being, customer | move, trade, consume_service, attracted_by_service | 被桌子吸引，交易 |
| 蘑菇农 mushroom_farmer | being, worker | move, craft, use_tool, work_domain | 营地居民，全工作链 |
| 玩家 player | being, actor | move, hunt, forage, craft, store, trade, use_tool, carry | 手操 |

### 3.5 初始生成（_spawn_initial_cards）

| 类别 | 内容 |
|------|------|
| 动物 | 羊×2、狼×2、兔×2、鹿×2、猞猁×1、野牛×1 |
| 第二波 | 灌木×8、田鼠×4、狐×2（公母各1，窝由狐狸自行建造） |
| 桃源 | 长老×1、采集者×1、青年×1（地图右侧边界随机） |
| 建筑 | 篝火×1、草棚×1、桌子×1、树×2、山×2 |
| 物品 | 石头×2、树枝×1、木头×2、空桶×1、干草×2 |
| 植物 | 活草×10 |
| 未生成 | 旅人（封存）、蘑菇农（按延迟生成）、狐窝（狐狸自行建造） |

### 3.6 生命周期规则

| 物种 | 繁殖条件 | 上限 | 成长 | 饿死 |
|------|---------|------|------|------|
| 羊 | 公+母 + 草≥3 + 无狼4格内 + 无羔羊 | min(4, ⌈草/3⌉) | 60 tick 成年 | 宽限 3 天 |
| 鹿 | 公+母 + 草≥阈值 + 无狼 + <上限 + 幼鹿<上限 | POP_CAP / FAWN_CAP | 指定 tick 数 | 宽限 3 天 |
| 田鼠 | 公+母 + 灌木虫≥阈值 + <上限 | POP_CAP / JUVENILE_CAP | 指定 tick 数 | 宽限 3 天 |
| 狐狸 | 公+母 + 有窝 + 无幼狐 | DEN_CAP（窝数量） | 指定 tick 数 | 宽限按 mesopredator |
| 狼 | 公+母 + 有窝 + 无幼狼 + 窝内<容量 | DEN_CAPACITY=3 | growthDay≥阈值 | 幼狼 starveDays≥阈值死亡 |

> 注：狐狸繁殖依赖狐窝——窝由狐狸行为管线中的 den_work 片段自行建造，不由 population_manager 生成。

---

## 四、吸附机制验证

### 验证卡

| 卡 | 注册内容 | 自动获得 | F5 结果 |
|----|---------|---------|---------|
| 猞猁 lynx | hunt + use_cover + mesopredator | mesopredator_hunt key → 能力管线 → 自动捕猎 | ✅ 猎杀 5 次 |
| 野牛 bison | forage + move_slow + flee + be_hunted | herbivore_grazer key + slow profile → 自动吃草 | ✅ 正常吃草 |

### 吸附闭环确认

新卡只需三步即可自动接入生态：
1. `card_db.gd` — 注册 card_type + 标签
2. `world_rules.gd` CARD_CAPABILITIES — 登记能力数组
3. `world_manager.gd` — spawn

**不需要改**：`ecosystem_behavior_key`、`EcosystemTickRegistry`、猎物判定、生态饱食、行为文件。

---

## 五、当前生态状态（最近 F5）

### 5.1 运行数据

| 指标 | 数值 |
|------|------|
| Tick 速率 | 2.74–2.87/s（全程正常） |
| L0 断言 | 805 |
| 活草 | 11–12 |
| 灌木 | 8 |

### 5.2 捕食者表现

| 捕食者 | 猎杀数 | 状态 |
|--------|--------|------|
| 狼 ×2 | 3 次 | 正常捕猎/饱腹巡林/窝内潜伏 |
| 狐 ×2 | 3 次 | 可猎杀，但常躲狼，未筑窝 |
| 猞猁 ×1 | 5 次 | 正常捕猎 |
| 玩家 | 数次 | 手操猎杀 |

### 5.3 生态事件（典型 session）

- 狼猎 fieldMouse / rabbit
- 狐猎 fieldMouse / fieldMousePup
- 猞猁猎 fieldMouse / rabbit
- 繁殖：deerFawn、fieldMousePup、wolfCub、lamb
- 狼窝正常运作（进出窝、叼肉）

---

## 六、已知缺口

### 6.1 功能性

| 问题 | 严重度 | 状态 |
|------|--------|------|
| 狐狸不筑窝 | 中 | 观察中（可能需更长时间或远离狼群） |
| 猞猁不在 §5 捕食者详情 | 低 | 诊断系统未覆盖新捕食者 |
| 狼崽可能断粮饿死 | 中 | 窝内有肉但投喂逻辑待验证 |

### 6.2 标签补全（P1）

| 卡 | 缺失 | 影响 |
|----|------|------|
| sheep / lamb | herbivore 标签 | domain 分类不完整 |
| lynx | carnivore 标签 | 与 fox 同类不一致 |

### 6.3 代码清理（P2）

| 位置 | 内容 |
|------|------|
| world_rules.gd | `is_bush`、`is_field_mouse*`、`is_adult_fox`、`is_fox_den` 仍用 card_type |
| world_rules_camp.gd | wolfDen type_name 硬编码 |
| card_base.gd:44 | 视觉可见性用 card_type 列表 |
| behaviors/ | 5 个旧文件是 8 行空壳，可删除 |

### 6.4 未实现的设计

| 内容 | 设计文档 |
|------|---------|
| 桃源社会完整逻辑 | v3.0-b-taoyuan-society-design.md |
| 玩家经济接入 | 未开始 |
| 旅人封存+餐饮 | v3.0-a 原案 |
| 狐狸繁殖闭环（筑窝→幼崽→成长） | 管线已就绪，缺繁殖触发 |

---

## 七、设计迭代方向

```
当前：v3.0-A 生态底座封板
  │
  ├─→ A1：修功能性 bug（狐窝、狼崽饿死）
  ├─→ A2：标签补全（sheep+herbivore、lynx+carnivore、is_* 去 card_type）
  ├─→ A3：旧 behavior 空壳删除
  │
  ├─→ B：桃源社会（v3.0-b）
  │     └─ 桃源人行为闭环、与玩家/营地的距离逻辑
  │
  ├─→ C：玩家经济
  │     └─ 交易、工具、烹饪、建造
  │
  ├─→ D：生态扩展
  │     ├─ 狐狸繁殖闭环
  │     ├─ 新物种（吸附验证后的第一张"真卡"）
  │     └─ 季节/天气影响
  │
  └─→ E：商业层
        └─ 方寸商国核心玩法
```

---

## 八、技术负债

| 项 | 说明 | 预计成本 |
|----|------|---------|
| 标签补全（P1） | sheep/herbivore、lynx/carnivore、is_* 函数 | 一轮 |
| 旧 behavior 清理 | 5 个空壳文件删除 | 半轮 |
| 诊断系统扩展 | §5 覆盖猞猁等新捕食者 | 半轮 |
| `_CARE_CHILD_TYPES` | 当前 5 对够用，>10 对再标签化 | 延后 |
| `game_ui.gd` 声明式渲染 | 当前委托 WorldRules 已够用，不急 | 延后 |
