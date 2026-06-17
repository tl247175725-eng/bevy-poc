# 方寸商国：桃花源记 — 设计哲学 v5

> 本文件记录自 Bevy 迁移以来全部设计讨论的哲学沉淀。
> 与 v4 设计文档不同，本文件不描述"怎么做"，只描述"为什么这样做"。
> 用于：设计决策时的参照、新增功能时的一致性检查、新人（AI session）理解项目根基。

---

## 一、存在论：标签即存在

### 1.1 标签是存在的唯一载体

一个东西"是什么"，不来自它的名字，不来自它的类继承，不来自代码中任何地方的类型判断。
一个东西是什么，**仅来自它携带的标签**。

```
type_name: "wolf"        ← 只是显示名，不是身份
tags: [predator, pack_hunter, body.large, ...]  ← 这才是狼的"存在"
```

`type_name` 匹配是反模式。`card_has_tag(def, "predator")` 是正确的。

### 1.2 标签只定义"是什么"，不定义"怎么做"

```
正确: "capability.move" → 这张卡可以移动
错误: "move_speed=0.25" → 这是参数，不是标签（参数从标签派生，见 §4）
```

标签是布尔质性的声明。数值不直接写在标签上——数值从标签组合推导。

### 1.3 标签的组合产生新意义（叠加域）

单个标签定义自身。多个标签同时存在时，它们的逻辑域叠加。
"叠加域"不是代码显式定义的——它从公理和需求匹配中涌现。

```
cold + predator_nearby + can_craft + fire_nearby
→ 单一标签都无法单独定义"造篝火"的需求
→ 但四者叠加 → 需求匹配指向篝火
→ 这不是标签显式说了"造篝火优先"，是需求叠加的必然结果
```

---

## 二、公理层：世界的物理引擎

### 2.1 四条公理构成世界的"允许"边界

| 公理 | 回答 | 输入 | 输出 |
|---|---|---|---|
| compose | 这张卡能进这个格吗 | 格状态 + 卡 profile | Allowed / Denied |
| traverse | 这张卡能跨介质吗 | 卡 profile + 从介质 + 到介质 | Allowed / Denied |
| perceive | A 能感知 B 吗 | A profile + B profile + 距离 + 介质 | Detected / Undetected |
| transform | A 作用于 B 的能量转化 | A profile + B profile + 动作类型 | 能量收支 |

公理只回答"能不能"，不回答"该不该"。
该不该——那是需求-满足匹配层的事。

### 2.2 公理是纯函数

- 不依赖任何全局状态（除传入参数）
- 不产生副作用
- 输入相同 → 输出相同
- 每条公理独立于其他三条

这是消除耦合的核心机制。

### 2.3 公理不判断"意图"

错的: compose 中说"羊不能进狼窝因为羊怕狼"
对的: compose 只说"格里已有狼 → Denied"。羊为什么不去——那是需求层的安全需求压制了觅食需求

---

## 三、决策层：哲学僵尸的"看似智能"

### 3.1 没有"决策"，只有"对应"

传统 AI 模型: 感知 → 分析 → 决策 → 规划 → 执行

我们的模型:
```
标签（是什么）
  ×
公理（允许什么）
  ×
需求（缺什么）
  ×
环境（有什么）
  ↓
匹配（什么行动同时覆盖最多急迫需求）
  ↓
执行（元动作序列）
```

没有"权衡利弊"的步骤。没有"计算得分"的步骤。
只有: **当前状态 → 需求激活 → 环境匹配 → 执行**。

### 3.2 需求-满足匹配替代效用评分

评分模型（Sims 式）: 每个对象广播广告，实体选最高分 → 实体是被动的，对象是主动的

需求匹配模型（我们的）:
- 需求在实体身上（标签定义: need:eat）
- 环境提供满足物（草满足 eat，篝火满足 warmth+safety）
- 匹配逻辑: 找出覆盖最多急迫需求的行动
- 不需要对象"广播广告"——实体主动寻找，因为需求是它自己的本质

### 3.3 行为来自标签叠加，不是外部评分

```
羊看到草 + 狼在附近 + 饿度中等
→ 激活需求: eat(中), safety(高)
→ 匹配:
    草: 满足 eat, 不满足 safety, 靠近狼
    远离狼: 满足 safety, 不满足 eat
→ 当前最高急迫需求是 safety → 远离狼
→ 这不是因为"远离狼得分高"——是因为只有 safety 需求被高急迫度激活
```

### 3.4 不选唯一最优，不做完美机器

从匹配结果中，满足"足够好"条件的选项随机选一。
避免机器感，也给玩家留下干预空间。

---

## 四、元动作与元数值：世界的语法

### 4.1 元动作——不可分解的行为基元

所有标签驱动的复杂行为 = 元动作的组合。

| 元动作 | 定义 | 示例派生 |
|---|---|---|
| move | 改变位置（曼哈顿单轴） | 巡逻 = move + wait循环 |
| strike | 对目标施加 1 单位力 | 攻击 = move→strike, 砸石 = strike×2 |
| consume | 消耗目标并转化能量 | 吃草、喝水、烧柴 |
| combine | 将两物合并为新物 | 造篝火 = combine(树枝,燧石) |
| release | 将持有物放置到世界 | 造完篝火→release到格上 |
| wait | 维持当前状态 N tick | 睡觉、孵蛋 |
| hide | 进入容纳态（不占格） | 藏草丛、进树洞 |
| emerge | 从容纳态退出 | 出草丛、出树洞 |

### 4.2 元动作之间零耦合

- strike 不知道 move
- consume 不知道 combine
- hide 不知道 emerge

和公理层一样: 每个元动作是独立的纯函数或系统。
标签把它们组合起来产生行为。

### 4.3 元数值——不可分解的测量基元

所有标签中出现的数值都必须追溯到元数值。

| 元数值 | 单位 | 定义 |
|---|---|---|
| tick | 模拟步 | 1 tick = 模拟层最小时间单位 |
| second | 秒 | 1 second = 1/N tick（由帧率决定） |
| minute | 分 | 1 minute = 60 seconds |
| hour | 时 | 1 hour = 60 minutes |
| day | 天 | 1 day = 420 tick（铁律）|
| cell | 格 | 1 cell = 空间最小单位 |
| hp | 生命值 | 1 hp = 1 单位生命 |
| energy | 能量 | 1 energy = 1 单位能量（用于 transform） |
| weight | 重量 | 从 size 标签派生: tiny=1, small=2, medium=3, large=5 |

### 4.4 派生规则

```
move_speed = cell / tick × 体型修正
  body.tiny  → 修正 1.5（跑得快）
  body.large → 修正 0.7（跑得慢）

hunger_decay = (1.0 / day) × 代谢修正
  body.large → 修正 2.0（消耗快）
  cold_env   → 修正 1.5（取暖消耗）

strike_damage = 1 hp（基础） + weight 加成
  大型动物打击更重

consume_energy = target.energy × efficiency
  efficiency 由 eater 的标签决定
```

**原则: 任何数值的"为什么是这个数"必须能追溯到元数值或标签。否则就是魔法数字，不可接受。**

---

## 五、禁止事项

### 5.1 禁止 type_name 硬编码

```rust
// ❌
if entity.type_name == "wolf" { ... }
match entity.type_name.as_str() { "grass" => ... }

// ✅
if card_has_tag(def, "predator") { ... }
if card_has_capability(def, "capability.move") { ... }
```

### 5.2 禁止魔法数字

```rust
// ❌
let speed = 0.25;  // 这是什么？为什么是 0.25？
need.current = need.current.min(15.0);  // 为什么是 15？

// ✅
let speed = base_speed / body_size_modifier(entity);  // 追溯到 size 标签
need.current = need.current.min(eat_satiety_threshold(entity));  // 追溯到 need 定义
```

### 5.3 禁止隐式行为锁

```rust
// ❌
if entity.fed_today && drive.condition_fed {
    continue;  // 跳过这个驱动——但为什么跳过？多久？什么条件下恢复？
}

// ✅
// 需求-满足匹配自然处理: 吃饱 → eat 需求不激活 → 觅食不触发
// 不需要显式的锁
```

### 5.4 禁止代码层面的 if-else 行为链

```rust
// ❌
if can_hunt { hunt(); }
else if can_forage { forage(); }
else if can_eat { eat(); }
// 加新行为 = 加新的 else if = 顺序耦合

// ✅
let actions = match_actions(activated_needs, environment);
execute_best_match(actions);
// 加新行为 = 加新标签 + 加新元动作组合 = 无代码改动
```

---

## 六、语义质量与系统稳定性

> 基于 2025-2026 年学术研究的双重搜索验证。
> 来源：Executable Ontologies (Boldachev 2025)、Dwarf Fortress Simulation Principles (Adams 2015)、
> 多智能体动力系统 (arXiv 2024-2025)、博弈论与极限环分析 (AAAI 2025)、
> 可执行本体论在游戏中的应用 (ResearchGate 2026)。

### 6.1 定义质量 = 与现实世界的同构度

本体论工程学标准：给定一个本体 OL 和一个理想本体 OC，OL 的质量 = OL 与 OC 之间的同构程度。

**保结构映射**：不是搬运现实的细节，而是搬运现实的结构原理。
- 温度、降雨、海拔、排水的交互 → 决定生物群系（不是直接定义生物群系）
- 速度、重量、能量的交互 → 决定撞击伤害（不是直接定义伤害值）
- 安全需求、制造能力、环境威胁的叠加 → 决定行为选择（不是直接定义行为优先级）

Tarn Adams (Dwarf Fortress): "不要直接定义生物群系。分别处理温度、降雨、海拔、排水。这些场的交互决定最终结果——更自然，内部更自洽。"

**可执行本体论 (EO) 范式**：
不是问"智能体该怎么行动？"——而是问"这个世界里什么是真的？当条件满足时，什么变得可能？"
行为从语义结构中涌现，而非被显式编程。我们的标签→公理→需求匹配→元动作架构正是 EO 范式的实现。

### 6.2 语义耦合的避免

LUDOCORE 系统（Smith et al. 2010）提出：用事件演算替代条件判断。Fluent（随时间变化的谓词）+ Event（离散事件）。新规则是纯粹的新增声明，不需要修改已有规则。

Tarn Adams 谈继承： "当你声明一个类是一种物品，它把你锁在那结构里。如果有一个物品行为像 A 又像 B，在类继承里几乎不可能。把不同组件开关——就简单得多。"

**我们的标签平铺架构天然避免此问题**：任何实体可以携带任何标签组合，不需要经过任何继承树。

### 6.3 振荡的数学本质

Poincaré-Bendixson 定理：二维动力系统的极限集只能是不动点或极限环。三维以上可能出现混沌。

**关键发现**：限制系统行为的不是维度（智能体数量），是**交互图的拓扑结构**。即使有任意多智能体，如果交互图稀疏（每个智能体只和少数邻居交互），极限行为仍然是简单的——只有极限环，没有混沌。

我们的实体已经有稀疏交互图：每个实体只感知曼哈顿范围 4-8 格内的邻居。这在数学上天然抑制混沌。

### 6.4 振荡区分：破坏性 vs 良性

| 破坏性振荡 | 良性振荡 |
|---|---|
| 每 tick 在两个同分驱动间切换 | 需求饱和后自然切换到下一个需求 |
| 碰撞-位移-再碰撞的死循环 | Near-tie 随机选 → 看起来"犹豫"但合理 |
| fed_today 锁死 → 无目的空转 | 偶尔的策略摇摆 → 丰富行为模式 |

"不稳定性、振荡和部分收敛不应仅被视为学习失败——它们包含关于策略探索的信息。" (Instability as Insight, 2025)
**不是所有振荡都要被消灭。需求匹配引擎应区分"破坏性死锁"和"良性探索"。**

### 6.5 底层防振荡法则

**行为惯性（Hysteresis）**：
- 当前正在执行的元动作序列不被新决策打断
- 除非触发安全急迫度（威胁等级 > 当前行为继续的阈值）

**执行冷却（Cooldown）**：
- 同一元动作组合执行后有最小间隔
- 防止短暂需求波动导致的频繁切换

**急迫度阈值（Activation Threshold）**：
- 新需求的急迫度必须 > 当前需求的 1.2 倍才切换
- 创造耗散——把系统能量导向稳定轨道而非无限循环

**熵正则化（Entropy Regularization）**：
- 需求慢衰减后的自然切换 = 天然的探索机制
- 持续引入变化，防止锁定，同时确保不无限循环
- 学术确认："熵正则化增强系统耗散性——将学习动态推向更平滑、更健壮的策略空间区域" (MARL 2024)

---

## 八、元数值体系（铁律）

> 2026-06-13 多轮深度讨论确认。同构 + 卡牌媒介兼容。
> 多 AI 独立交叉验证收敛。

### 8.0 元数值分层

元数值 ≠ 派生值。元数值是不可推导的测量基元——派生值由元数值组合而成。

| 层 | 含义 | 示例 |
|---|---|---|
| A 必须 | 世界无法闭环，物理引擎刚需 | tick/cell/mass/temperature/pH |
| B 强推荐 | 涌现依赖，高层行为的前提 | hp/trust/reputation/scarcity |
| C 可选 | 看后期，不影响当前开发 | 电磁/超自然参数 |
| D 派生 | **不进入元数值表** | speed/damage/price/morale |

### 8.1 A层·必须（世界无法闭环）

**时间：**
tick / day / phase / season / year / duration / cycle / cooldown
- 1 day = 420 tick（铁律，基于人类事件分割理论研究）
- 1 phase = 60 tick（清晨/早晨/上午/中午/下午/傍晚/深夜）

**空间：**
cell / position / direction / angle / height / elevation / depth / area / volume / capacity

**物质：**
substance_id / mass / density / composition / concentration / purity

**材料（真实物理值，不分级）：**
hardness / yield_strength / fracture_strength / toughness / elastic_modulus / stiffness
friction / roughness / adhesion / cohesion / wear_rate / plasticity / max_edge

标签 = 钥匙，数据在引擎内置材质数据库：
```
铁: material:iron → density=7874, hardness=4, yield=500MPa...
木: material:wood → density=700, hardness=1, toughness=high
精金: material:adamantine → density=200, hardness=10, edge=100000
```

**热：**
temperature / body_temperature / heat_capacity / thermal_conductivity / thermal_insulation
melting_point / boiling_point / freezing_point / ignition_temperature

**流体与气候：**
pressure / humidity / soil_moisture / wind_direction / wind_speed / precipitation / sunlight / flow_resistance

**化学与腐败：**
pH / salinity / toxicity / reactivity / oxidation_rate / corrosion_rate / solubility / volatility
decay_rate / nutrient_profile / flammability / oxygen_availability

**光与声与气味：**
light_intensity / color / opacity / transparency / contrast / shadow
sound_intensity / sound_frequency / attenuation / noise
odor_intensity / odor_profile / odor_volatility

**感官能力（四通道：视觉/听觉/嗅觉/触觉）：**
vision_range / vision_acuity / hearing_range / hearing_acuity / smell_range / smell_acuity
touch_range / touch_acuity / taste_sensitivity / balance_stability / thermoception_sensitivity / pain_sensitivity

感知标签表达：
```
sense:vision(r=N, acuity=high|medium|low, motion_sensitive=true|false)
sense:hearing(r=N, passive=true|false)
sense:smell(r=N, passive=true|false)
sense:touch(r=1)
```
感知范围 = 感官标签 × 体型修正 × 时间段修正 × 环境条件
感知结果 = DetectionQuality { range, fuzzy_level, identified }（不返回二元，返回模糊度）

**信息：**
bit / signal_strength / signal_noise / bandwidth / latency / storage_capacity
memory_decay / accuracy / uncertainty / signal_confidence / knowledge_complexity

### 8.2 B层·强推荐（涌现依赖）

> 2026-06-13 经文献交叉验证筛选。标准：不能从已有元数值稳定推出。

**生命：**
hp(工程保留) / energy / hydration / nutrition / metabolism_rate / recovery_rate
immune_strength / infection_load / toxin_load / heart_rate / respiration_rate

**繁殖：**
age / lifespan / growth_rate / fertility / gestation_progress
heritability / mutation_rate

**生态：**
soil_nutrient

**心智：**
need / attention / trust / knowledge / skill / belief / belief_confidence
willpower / decision_threshold

**社会：**
reputation / commitment_strength / relationship_strength / kinship_distance / norm_strength

**经济：**
demand / supply

**从 B 层移除（→ 标签，不是数值）：**
sex / genotype / phenotype_trait / dominance / empathy / ownership / permission / obligation / group_identity

### 8.3 C层·可选（看后期）

电磁：electric_charge / current / potential / conductivity / magnetic_field / radiation
超自然：spiritual_energy / essence / mana / qi / creation_cost / destruction_resistance / reality_stability

### 8.4 D层·派生（不进入元数值表）

**物理派生：**
speed / damage / attack_power / defense / armor / visibility / comfort / weight
distance — 由 position_a - position_b 即时计算
phase_state(solid/liquid/gas) — 由 temperature + pressure 跨越 melting/boiling 点确定

**生命派生：**
hunger / thirst / fatigue / stamina / pain / stress / maturity
offspring_count / genetic_diversity

**生态派生：**
biomass / growth_stage / water_need / germination_rate / pollination_rate / biodiversity / ecological_harmony

**心智派生：**
drive / preference / utility / salience / curiosity / fear / anger / loyalty / tension / alertness / affinity

**社会派生：**
authority / status / rank / prestige / influence / debt / violation_severity / group_cohesion

**经济派生：**
scarcity / cost / risk / expected_value / exchange_value / labor_value / price / yield / productivity / wealth

**综合派生：**
morale / danger / beauty / influence_score / technology_level / social_stability / task_priority / power

**派生规则示例：**
```
speed = distance / duration
weight = mass × gravity（元数值是 mass，不是 weight）
hunger = nutrition_need - nutrition
damage = force × edge × material_response × angle
price = demand × scarcity × exchange_value × trust_modifier
```

### 8.5 元数值判断标准

1. 不能由已有元数值稳定推出
2. 是大量规则、状态或结果的共同输入
3. 可以被元动作/Elapse/Chance 持续读取和改变
4. 派生值不进入元数值表
5. 所有游戏中的数字都应能从元数值派生

**原则：任何数值的"为什么是这个数"必须能追溯到元数值或标签。否则就是魔法数字，不可接受。**

---

## 九、元动作体系（铁律）

> 2026-06-13 多 AI 独立交叉验证收敛至 25 个。
> 五条判断标准来自形式本体论（BFO）、游戏语法理论（Crawford/Koster）、矮人要塞互动系统——三方交叉验证。

### 9.1 分层全景

**25 个元动作，7 层：**

| 层 | 元动作 | 数量 |
|---|---|---|
| 物理 | Move / Strike / Consume / Combine / Separate / Constrain / Release / Pause / Hide / Emerge / Alter | 11 |
| 信息 | Signal / Receive / Process / Store | 4 |
| 生物 | Reproduce / Inherit / Transform | 3 |
| 心智 | Decide / Teach / Bond | 3 |
| 社会 | Assign / Commit | 2 |
| 超自然 | Create / Destroy | 2 |

另：非主体底层机制 Elapse（时间流逝）/ Chance（概率结算）——2 个。

**总计：25 + 2 = 27 个世界语法基元。**

### 9.2 物理层（11个）

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Move | 空间位置变化 | 曼哈顿单步，compose+traverse 验证 |
| Strike | 唯一冲击元动作 | 战斗/加工/打火同源，分歧来自参数（力度/角度/工具材质） |
| Consume | 摄入/消耗——资源转入自身 | AxiomEngine::transform，能量转化 |
| Combine | 属性代数合并 | **无配方表**，产物由输入材质决定 |
| Separate | 整体变多个部分 | 砍树=Separate(树→木材+树枝) |
| Constrain | 限制行动自由或状态空间 | 捆绑/关笼/筑墙/筑坝 |
| Release | 解除约束，放到世界 | 从携带态放置，需 compose 验证 |
| Pause | 主动不行动 | 标签分化 → Rest/Think/Observe/Appreciate/Examine（pause:标签） |
| Hide | 降低可感知性，进入容纳态 | 藏草丛/进树洞/入水，vacate compose 格 |
| Emerge | 恢复可感知性 | 从容纳态退出，重新占有 compose 格 |
| Alter | 改变物质属性 | 烹饪/燃烧/冶炼/发酵——不改变"是什么"，改变属性值 |

### 9.3 信息层（4个）

物理层之上存在信息层。信号不只是物理波动——它携带语义内容。

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Signal | 发送携带语义内容的信息 | 狼嚎=宣示领地, 羊叫=警报, 人说话=句子 |
| Receive | 解码并理解语义内容 | 听到嚎叫≠听到声音——是"知道有狼在宣示" |
| Process | 推理/比较/判断/规划 | **不是 Pause**——是主动心智工作。思考不是等待的副产品 |
| Store | 记忆/记录/铭刻 | 主动编码，不是等待的副产品。和 Pause(think) 的被动记忆不同 |

层叠示例：
```
Signal(狼嚎, claim_territory) 
  → 物理层: Strike(sound) — 发出声波
  → 信息层: Signal(claim_territory, r=16) — 声波携带语义

Receive(羊, 狼嚎)
  → 物理层: perceive(hearing) — 检测到声波
  → 信息层: Receive(claim_territory) — 解码为"有狼"
  → 心智层: Process — 判断威胁等级
  → 需求层: safety 急迫度上升 → Decide → 行为激活
```

### 9.4 生物层（3个）

生物现象不可归约为物理元动作 + 标签。

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Reproduce | 产出新实体 | 两源实体 + 继承规则 → 新 entity |
| Inherit | 父→子传递标签 | Reproduce 的子过程——标签不是随机，是继承 |
| Transform | 生物体内变化 | 成长/变态/愈合/衰老——不同于 Alter（Alter 改变物质属性，Transform 改变生物体形态/阶段） |

**Grow ≠ Pause——Grow 是 Transform（生物体内变化，不是等待）。**

### 9.5 心智层（3个）

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Decide | 从"知道了"到"去做了"的跳变 | 生成意向。所有信息的终点、所有行动的起点 |
| Teach | 持久性能力传授 | 不是 Signal——是永久转移标签。Signal 说完就完，Teach 改变了接收方的能力 |
| Bond | 创建持续性关系链接 | 情感/信任/敌意/归属。不是 Combine——Bond 创建关系，不创建物 |

Decide 的关键性：需求匹配引擎的输出不是"执行某个行动"——是激活 Decide，Decide 再驱动元动作序列。没有 Decide，Receive+Process 只是"知道"但不会"去做"。

### 9.6 社会层（2个）

社会现象不可归约为心智元动作 + 标签。

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Assign | 命名/定价/所有权/身份/权限 | 社会/符号层的标记。不是信息层——Assign 改变社会现实，不只是传递信息 |
| Commit | 生成面向未来的约束 | 契约/债务/誓言/禁令。不是 Bond——Bond 是情感链接，Commit 是规范性约束 |

### 9.7 超自然层（2个）

| 元动作 | 定义 | 关键说明 |
|---|---|---|
| Create | 无前因产出新实体 | 不同于 Reproduce——Reproduce 需要父体，Create 从无到有 |
| Destroy | 从因果链彻底移除 | 不同于 Strike 致死——Strike 杀死留下尸体，Destroy 从世界彻底消失 |

**超自然 ≠ 规则豁免。超自然 = 极端物理参数 + 公理覆盖 + 灵魂层状态。**
90% 的超自然概念是标签组合——不需要改框架。

### 9.8 非主体底层机制（2个）

| 机制 | 定义 |
|---|---|
| Elapse | 时间流逝——世界推进。所有 duration/cooldown/decay 的驱动源 |
| Chance | 概率结算——多结果随机分配。所有 mutation/critical/drop 的随机源 |

### 9.9 元动作判断标准（五条武器）

| 标准 | 问 | 答"是"→ |
|---|---|---|
| **不可分解性** | 能分解为已有元动作的序列？ | 是→**不是**元动作 |
| **产物新颖性** | 创造了已有元动作创造不了的东西？ | 是→**是**元动作 |
| **跨层效应** | 跨越物理/信息/生物/心智/社会层？ | 是→**是**元动作 |
| **因果独立性** | 能用已有元动作的因果链完整描述？ | 是→**不是**元动作 |
| **标签不可替代性** | 用已有元动作+标签能达到同样效果？ | 是→**不是**元动作 |

已验证的排除案例：Share/Copy/Adapt/Mutate/Exchange/Merge/Split/Transfer/Attach/Detach——全部被五条标准判定为不是元动作。

### 9.10 无配方原则（铁律）

**Combine 不做 a+b=c。不做配方表。**
产物由输入的材质属性代数合并决定。合并后的物品功能（能不能砍/砸/切）由属性集合推导，不由配方判定。
引擎不认识"刀"——只认识 {hard≥4, shape:blade, edge:present}。
任何"带把的硬片"都可以当刀。好不好用由物理算，不由配方判。

### 9.11 交互层 vs 世界层

**玩家只有两个万能操作（上帝之手，不是元动作）：**
- **Act（砸/Smash）** = 拖拽卡 A 到卡 B 上释放
- **Arrange（叠/Stack）** = 右键操作

结果由引擎根据 A 和 B 的标签+材质自动推断对应元动作：
```
石刀 + 狼 → Strike(harm)
石刀 + 燧石 → Strike(shape)
树枝 + 树枝 → Combine
石刀 + 鹿尸 → Strike(cut) + Consume
```

**玩家的每一个操作都有物理反馈——不存在"砸了 500 下没结果"。** 硬度差决定物理必然性。

### 9.12 防振荡法则

1. **行为惯性**: 当前元动作序列不被打断（除非安全急迫度触发）
2. **执行冷却**: 同一元动作组合有最小间隔
3. **急迫度阈值**: 新需求必须 > 当前需求 1.2 倍才切换
4. **良性振荡保留**: 需求饱和后的自然切换不消灭

---

## 十、元本质完备性声明

**元本质 = 25 个元动作 + 元数值 A/B 层 + 标签。**
不需要第四根柱子。

### 为什么元关系和元状态不是独立柱子

**元关系（Owns/Contains/ParentOf/等）：**
- 是对应元动作执行后留下的**标签状态**，不是新基元
- Owns = Assign(ownership) 执行后的残留标签
- Contains = Hide(entity, into=container) 执行后的残留标签
- ParentOf = Reproduce + Inherit 执行后的残留标签
- 所有关系都可以被"哪个元动作创建了它 + 什么标签留下来了"完整描述

**元状态（alive/dead/solid/liquid/等）：**
- 是元数值跨越**阈值**的结果，不是新基元
- alive = hp > 0
- dead = hp ≤ 0
- solid/liquid/gas = temperature 跨越 melting_point/boiling_point
- 所有状态都是元数值 + 阈值的派生

**元意义**：标签携带语义——不需要独立的意义层。

### 设计原则

1. **三柱职能边界（铁律）**：标签描述形态（是什么），元数值描述强度（有多强），元动作描述变化（在做什么）。三柱不越界，不重复。
2. **无配方**：Combine 不做 a+b=c，属性代数合并
2. **超自然 ≠ 规则豁免**：超自然 = 极端物理参数 + 公理覆盖 + 灵魂层状态
3. **上帝视角两操作**：Act 和 Arrange 不是元动作——是交互层动词，引擎自动推断
4. **防振荡**：惯性 + 冷却 + 阈值 + 保留良性振荡
5. **元关系不是新柱子**：Owns/Contains/ParentOf = 元动作执行后留下的标签状态
6. **元状态是派生**：alive/dead/solid/liquid = 元数值跨越阈值的结果

---

## 十一、Z 轴与多层棋盘体系（铁律）

> 2026-06-14 确认。棋盘之间的边界是世界的语法。

### 11.0 位置体系

```
Entity { x: u8, y: u8, z: i16 }  // 负=地下, 0=地表, 正=空中
spatial_index: 3D 桶，每层独立 2D 棋盘
```

### 11.1 三层表现

| 类型 | Z 范围 | 表现 | 权限 |
|---|---|---|---|
| 抽象通道 | 命名层之间 | 进度条 + 产出表 + 随机发现事件 | 挖/取/运输，不可建基地 |
| 命名层棋盘 | 地质年代/大气层分界 | 完整棋盘（36×24） | 全部权限 |
| 天空悬层 | 任意高度 | 完整棋盘 | 需悬浮能力+持续动力消耗 |

### 11.2 棋盘激活

```
激活条件（二选一）:
  1. 命名层 → 触及层边界 → 自动激活该层棋盘
  2. 放置基地卡 → 任意 Z 坐标手动激活棋盘
```

### 11.3 通行规则

```
同一 Z 层: 曼哈顿移动
跨 Z 层: 需要运输卡（竖梯/电梯/火箭等）→ Z 层间不自动连通
天空悬层: 位置 (x,y,z) 不自动连通地面 (x,y,0)
```

### 11.4 命名层锚点

**地下——地质年代（深度为游戏压缩值）：**

| 命名层 | 深度 | 特征资源 |
|---|---|---|
| 更新世 | 100m | 猛犸化石、冰期沉积 |
| 上新世 | 300m | 铁矿、褐煤 |
| 中新世 | 500m | 石油层 |
| 白垩纪 | 800m | 白垩岩、恐龙化石 |
| 石炭纪 | 1200m | 煤矿层 |
| 寒武纪 | 2000m | 页岩、三叶虫化石 |
| 前寒武纪 | 3500m | 铁矿石、花岗岩 |

**天空——大气分层（基于 NOAA 2026 标准大气）：**

| 命名层 | 高度 | 环境 |
|---|---|---|
| 对流层 | 0-12km | 正常~补充氧 |
| 平流层 | 12-50km | 需增压服 |
| 中间层 | 50-85km | 太空级维生 |
| 热层 | 85-600km | 航天器 |
| 外大气层 | 600km+ | 轨道空间 |

### 11.5 地下与天空的不同物理

| | 地下 | 天空 |
|---|---|---|
| 支撑方式 | 下方稳定链→地表 | 悬浮动力持续消耗 |
| 能量需求 | 无（静态结构） | 持续消耗（燃料/能源） |
| 可移动性 | 不可移动 | 可移动（漂移/重定位） |
| 通行方式 | 物理连通（楼梯/竖井） | 运输链路（电梯/火箭） |
| 可悬空 | ❌ | ✅ |

### 11.6 非当前层处理

```
当前层: 完整 tick 模拟
非当前层: 概率积分估计（时间 × 能力 × 资源 → 最可能结果）
非激活层: 不存在（不存数据，不消耗性能）
```

---

## 十二、标签纯度与抽象层级（铁律）

> 2026-06-16 确认。基于功能性状生态学 + BFO 本体论工程学。

### 12.1 抽象层级：卡的定义

一张卡 = 在所选粒度层级上，能够同时满足以下三个条件的实体：
1. **独立承载标签**（不是继承自父实体的）
2. **独立参与元动作**（可以被 Move/Strike/Consume/Store）
3. **独立存在于格子上**（有自己的位置状态）

不满足 → 不是卡，是标签或元数值。
同类别深度一致：哺乳动物全模拟器官，鸟全模拟翅膀，虫不模拟器官。不跨类别不一致。

### 12.2 标签纯度标准

| 标准 | 测试 |
|---|---|
| **不可含物种名** | `predator` 不是 `wolf`。标签是描述属性，不是分类物种 |
| **可传递** | 任何带 `predator` 的实体触发相同的 fear 反应——不查物种 |
| **可组合** | `predator + nocturnal + pack_hunter` 自动涌现行为，不需要额外规则 |
| **趋同验证** | 该性状在三个以上不相关谱系中独立进化过→高纯度。只有一种动物有→不是标签 |
| **行为闭包** | 有此标签→引擎能产出完整行为后果——不需要额外查物种 |

### 12.3 标签五维度（功能性状生态学——Winemiller 2015）

| 维度 | 含义 | 标签前缀 |
|---|---|---|
| **栖息地** | 在哪里生活 | `habitat:` |
| **生活史** | 怎么生长繁殖 | `repro:` `growth:` |
| **营养** | 吃什么、怎么获取 | `diet:` `foraging:` |
| **防御** | 怎么不被吃 | `defense:` |
| **代谢** | 怎么处理能量 | `metab:` `thermo:` |

加上：
- **社会**（Social）：`social:`
- **身体**（Body）：`body:` —— 结构标签（器官、肢体、大小）
- **能力**（Capability）：`capability:` —— 可执行的行为

### 12.4 标签技术架构

- **集中注册表**（TagRegistry）——"生态位周期表"，所有标签的 bit 位在此定义
- **每实体 bitmask**（TagBits）——实体在性状空间中的位置
- **查询** = bitmask AND，O(1)
- 文献依据：C++ Data Oriented Design (2026) 最小集合驱动 + bitmask 签名

---

## 十三、抽象提纯标准与深度统一规则（铁律）

> 2026-06-16 确认。基于 BFO 四原则 + 功能性状生态学。

### 13.0 核心原则：同构——让万物按本来的本质存在

标签不是我们赋予实体的行为。标签是实体本来的样子。引擎读标签，行为自然流出。

### 13.1 BFO 抽象深度四原则

| 原则 | 含义 | 我们的规则 |
|---|---|---|
| **Adequatism（足够性）** | 建模到目的所需即可 | 能被砸/叠/拿/抽操作→必须建模。不能被操作→不建模 |
| **Perspectivalism（视角性）** | 不同域可不同粒度，但同域必须一致 | 不同类别（动物/植物/地形）可不同维度，但同类内深度完全相同 |
| **Realism（实在性）** | 只描述真实存在的东西 | 标签从性状提取，不发明。趋同验证（三谱系以上独立进化） |
| **Fallibilism（可错性）** | 可修订 | 发现缺标签→补。但补了之后同类全补，保持深度一致 |

### 13.2 抽象深度统一规则

**1. 同类绝对一致：** 同类别内所有成员共用同一套标签模板。模板深度由该类别中最复杂的成员决定。

**2. 跨类各自独立：** 不同类别可用不同维度。动物用 `diet`，植物用 `nutrition`——不是"植物更浅"，是"植物有不同的本质"。

**3. 目的测试：** 该属性是否影响游戏操作（砸/叠/拿/抽）或引擎计算（Need/Perceive/Decide）？是→必须建模。否→不建模。

**4. 粒度上下限：** 上限=器官级别（肝/心/肺可被砸伤）。下限=不模拟细胞/分子/原子。指甲生长速度不模拟——不可被独立操作。

### 13.3 各类别标签深度模板

| 类别 | 必修标签维度 | 禁入标签维度 |
|---|---|---|
| **动物-哺乳类** | diet/foraging/foraging_stratum/defense/thermo/metab/repro/growth/social/activity/movement/cognition/sense/body_size/body_plan/capability/habitat/habitat_range + 器官标签 | — |
| **动物-鸟类** | 同上 + body_plan:avian + capability:fly | — |
| **动物-爬行类** | 同上（thermo:ectotherm），cognition 上限 basic_learning | cultural_transmission |
| **动物-鱼类** | 同上，body_plan:fish | cognition |
| **植物-树木** | habitat/growth_form/woodiness/lifespan/nutrition/growth/repro/metab/defense/fire_response/drought_tolerance/flooding_tolerance/shade_tolerance/dispersal/body_plan:plant/body_size | cognition/activity/social/movement/foraging |
| **植物-草本** | 同上 + growth:fast | 同上 |
| **地形** | habitat/state/body_plan | 全部生物标签 |

### 13.4 纯度验证五条

每次新增标签，必须通过五条测试：

1. **不可含物种名** — `predator` 不是 `wolf`
2. **可传递** — 有此标签→有确定行为后果，不需额外查物种
3. **可组合** — 多个标签共存→自动涌现新行为
4. **趋同验证** — 该性状在三个以上不相关谱系中独立进化过
5. **行为闭包** — 引擎仅凭此标签集合能产出完整行为链

---

## 十四、未来方向：从哲学僵尸到涌现世界

当前阶段的目标是:
1. 框架提纯: 确保所有逻辑走标签→公理→需求匹配→元动作的管线
2. 元动作+元数值: 建立不可分解的语法基础（✅ 已完成）
3. 需求匹配引擎: 从标签+环境+需求到元动作序列的自动映射
4. 定义完备: 在语法基础上定义所有现有卡的行为

后续方向（不在当前范围）:
- 记忆系统: 实体记住"上次在这被狼追"→影响后续行为
- 社会层: pack 协作、flock 信息传递、territory 标记
- 经济层: 玩家商业行为作为"楔子"插入自转的生态系统
