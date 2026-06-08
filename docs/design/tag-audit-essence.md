# 标签本质性审计与设计哲学

> 基于代码当前状态，2026-06-03

---

## 一、设计哲学

### 核心原则

**标签是数值的唯一依据。** 不调数值，调标签。每项数值参数必须能从卡片的标签集合中推导出来。缺约束就补标签，不调数字。

**标签是行为原子。** 一个标签 = 一个不可再拆的逻辑单元。删除该标签，对应的行为应该消失；该标签影响的行为不应该超出其语义范围。

**标签模拟真实因果。** 卡的标签集合必须完整覆盖它在生态中"应该发生的行为"和"不该发生的行为"。狐狸一天杀 13 只兔子——这不是数值错误，是缺少"饱足即停"的标签约束。

**固定数值，浮动标签。** 小型生物 HP 统一 = 1。脆弱性不由数值浮动表达，而由标签（escape、cover、burrower）表达。两只 HP=1 的动物，谁活下来取决于标签组合对环境的适配。

### 标签库原则

标签一旦定义，所有后续卡共享复用。新卡优先匹配已有标签组合。确实无法用已有标签表达的，才新增标签。

---

## 二、HP 审计

| 卡 | HP | 体型标签 | 判定 |
|----|-----|---------|------|
| 田鼠 fieldMouse | 1 | smallPrey | ✅ 最小=1 |
| 幼鼠 fieldMousePup | 1 | smallPrey, juvenile | ✅ |
| 兔 rabbit | 1 | smallHerbivore | ✅ 小型=1 |
| 羊羔 lamb | 1 | largePrey, juvenile | ✅ 幼体=1 |
| 幼狼 wolfCub | 1 | predator, juvenile | ✅ |
| 幼狐 foxCub | 1 | carnivore, juvenile | ✅ |
| 羊 sheep | 2 | largePrey, herbivore | ⚠ 同为大型猎物，羊 HP=2，鹿 HP=3。差值无标签依据 |
| 幼鹿 deerFawn | 2 | largePrey, juvenile, herbivore | ⚠ 同为幼体，鹿崽 HP=2，羊羔 HP=1。是否因 largePrey？ |
| 鹿 deer | 3 | largePrey, herbivore | ✅ 介于羊(2)和水牛(4)之间 |
| 狐 fox | 3 | mesopredator, carnivore | ✅ 中型捕食者 |
| 狼 wolf | 4 | predator | ✅ 顶级捕食者 |
| 水牛 waterBuffalo | 4 | largePrey, herbivore, tough | ✅ tough 标签解释高 HP |
| 水牛犊 waterBuffaloCalf | 2 | largePrey, juvenile | ✅ 幼体脆弱 |
| 玩家 player | 6 | actor | ✅ |

### HP 异常

1. **羊 HP=2 vs 鹿 HP=3**。两者都是 largePrey + herbivore。鹿有 `wildPrey` 标签，羊没有。但 `wildPrey` 当前是孤立标签（无逻辑引用），不驱动任何行为，不能作为 HP 差异的依据。如果鹿的身体更大 → 应该有标签（如 `robust` 或 `body.large`），而不是靠隐形差异。

2. **鹿崽 HP=2 vs 羊羔 HP=1**。同为 juvenile。差异需要标签解释——幼鹿有 `wildPrey` 但羊羔没有？同样的问题：`wildPrey` 不驱动行为。

---

## 三、孤立标签（24 个，零逻辑引用）

这些标签写在了 card_db.gd 里，但在游戏逻辑中从未被任何代码读取或决策：

### 生态角色类
| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `scavenger` | fox | 狐的清腐走 `capability.scavenge`，tag 本体冗余。保留作为身份标识，或删除靠能力推导 |
| `wildPrey` | deer, deerFawn | 无代码消费。如果代表"野生警觉"，应接具体行为（逃跑判定加成），否则删除 |
| `omnivore.small` | fieldMouse | 无代码消费。如果代表杂食，应接入 forage 目标扩展（虫+浆果），否则删除 |
| `burrower` | fieldMouse | 无代码消费。应接入地下/掘穴行为 |

### 人类角色类
| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `human` | taoyuan 三人 | 无代码消费。与 `being` 重复？保留作为身份标识，暂不动 |
| `elder`, `forager`, `youth` | 桃源人 | 无代码消费。桃源人行为走 `taoyuan` + 社交距离，角色区分未用到。保留待桃源社会开发 |

### 物品/材料类
| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `toolHead` | tri, square | 无代码消费。如果用于工具合成配方，应接入 transform_input |
| `craftPart` | grassRope | 同上 |
| `novelty` | copperCraft | 无代码消费 |
| `stoneSource`, `woodSource`, `mushroomSource` | mountain, tree, mushroomWood | 资源源识别走的是 capability（define_domain + produce_resource），不是 tag。tag 本体冗余 |

### 植物/自然类
| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `foodSource` | grass | grass 的"被吃"逻辑走的是 `is_living_grass` 函数，不是 tag。但 tag 可保留作为信息栏展示 |
| `plant.patch` | bush | 无代码消费。`bush` tag 已足够 |
| `berry.source` | bush | 浆果再生走 BUSH_BERRY_INTERVAL_SECONDS 常量 + `is_bush` 函数，不读 tag |
| `microfauna.host` | bush | 虫再生走常量 + `is_bush`/`is_living_grass` 函数，不读 tag |
| `regenerates` | bush | 同上，再生走 set_timeout，不读 tag |
| `natural` | bush | 无代码消费 |
| `organic` | 所有尸体 | 无代码消费。尸体行为走 `is_corpse` + card_type |

### 其他
| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `soil` | humus | 无代码消费。humus 走 `fertile` tag，`soil` 是装饰 |
| `wet` | wetWood | 无代码消费 |
| `naturalDrop` | twig | 只做信息展示 |

---

## 四、死标签（纯装饰，连 selection_info_panel 也没列入 _IDENTITY_ORDER）

| 标签 | 所在卡 | 建议 |
|------|--------|------|
| `consumable` | 所有食物/肉 | 删除或接入消耗逻辑 |
| `food` | 同上 | 同上 |
| `food.raw` | 生肉 | 同上 |
| `raw` | 生肉 | 同上 |

这些标签在 _SKIP_TAGS 中被跳过展示，也不参与任何游戏逻辑。

---

## 五、数值参数审计：无标签依据的浮动常量

以下常量在 game_state.gd 中定义，但没有对应的标签声明"为什么是这个值"。

### 繁殖类

| 常量 | 值 | 缺乏的标签依据 |
|------|-----|-------------|
| DEER_POP_CAP=5 | 鹿上限 | 没有 `population.dense` 或领地标签 |
| DEER_REPRODUCE_MIN_GRASS=4 | 草阈值 | 数值裸奔 |
| DEER_FAWN_CAP=1 | 幼鹿上限 | 同上 |
| FIELD_MOUSE_POP_CAP=8 | 鼠上限 | 同上 |
| FIELD_MOUSE_JUVENILE_CAP=2 | 幼鼠上限 | 同上 |
| FOX_DEN_CAP=3 | 狐窝上限 | 同上 |

### 时间类

| 常量 | 值 | 缺乏的标签依据 |
|------|-----|-------------|
| RABBIT_EAT_TICK_SECONDS=45 | 兔进食耗时 | 缺 `grazer.slow` 或同类标签 |
| CORPSE_DECAY_SECONDS=15 | 腐解时间 | 尸体差异无标签表达（羊/鹿/狼不同） |
| WOLF_DIGEST_SECONDS=28 | 消化冷却 | 狼 vs 狐应有不同冷却？目前共用 |

### 饥饿/生长类

| 常量 | 值 | 缺乏的标签依据 |
|------|-----|-------------|
| WOLF_MEAT_PER_DAY=2 | 狼日食量 | 应该从 `predator` + `largePrey diet` 推导 |
| FOX_SCAVENGE_PER_DEN_DAY=2 | 狐清腐上限 | 应该从 `scavenger` 标签推导 |
| WOLF_CUB_GROW_DAYS=6 | 幼狼生长 | 应该从 `predator` + `juvenile` 推导 |
| FOX_CUB_GROW_TICKS=60 | 幼狐生长 | 同上，但单位不一致（天 vs tick） |
| ECOLOGY_STARVE_GRACE_DAYS=3 | 饿死宽限 | 应该从体型/代谢标签推导 |
| HUNT_MISS_COOLDOWN=3.6 | 扑空冷却 | 应该从速度能力推导 |

---

## 六、缺失的标签

根据当前生态问题和设计哲学，需要新增以下标签：

### P0：影响生态平衡

| 标签 | 语义 | 应挂在 |
|------|------|--------|
| `small_gut` | 食量小，吃一点就饱 | fox, fieldMouse |
| `big_gut` | 食量大，能吃很多 | waterBuffalo |
| `satiation` 或饮食上限机制 | 狐狸为什么一天只该吃 2 只而不是 13 只 | 所有捕食者 |

狐狸当前缺少的不是"能猎什么"，而是"猎够了就停"。`mark_ecology_fed` 只是标记"吃过了"，但没有阻止继续吃。需要猎后强制冷却或日猎上限。

### P1：补全孤立标签的逻辑

| 标签 | 应驱动的行为 |
|------|------------|
| `burrower` | 田鼠可进入地下状态，减少被猎概率 |
| `wildPrey` | 鹿逃跑判定加成，或在有玩家/狼时更早预警 |
| `omnivore.small` | 田鼠觅食目标多样化（虫+浆果+种子） |

### P2：数值标签化

| 概念 | 建议标签 | 代替的常量 |
|------|---------|-----------|
| 繁殖力 | `breed.slow` / `breed.normal` / `breed.fast` | 各类 REPRODUCE_MIN_* / POP_CAP |
| 代谢速度 | `metab.slow` / `metab.normal` / `metab.fast` | STARVE_GRACE_DAYS 差异 |
| 体型层级 | `body.small` / `body.medium` / `body.large` | HP 差异的标签依据 |
| 食谱范围 | `diet.narrow` / `diet.wide` | HUNT_DIET / 猎物偏好 |

---

## 七、待整合的标签

以下标签可以考虑二次描述整合（不删底层，增设高层行为描述符）：

| 新标签 | 由以下标签推导 | 用途 |
|--------|-------------|------|
| `timid` | escape_small + flee + be_hunted | "见敌必逃"——兔、鼠的共通行为模式 |
| `protective` | care_child + return_home + reproduce | "护崽回巢"——狼、狐的共通行为模式 |
| `grazing` | forage + move_normal/slow + eat_timer | "持续进食"——羊、鹿、水牛的共通进食模式 |
| `caching` | carry + return_home + scavenge | "储食回窝"——狐的储肉行为 |

这些不是替代底层标签，而是给行为管线提供可复用的行为片段描述符。

---

## 八、行动建议

| 优先级 | 行动 | 预期收益 |
|--------|------|---------|
| **P0** | 给狐狸加猎后饱足机制（small_gut 或日猎上限） | 解决狐狸杀兔过度的核心问题 |
| **P0** | HP 差异补标签依据：鹿崽加 body.medium 或解释为什么 HP=2 | 消除"数值拍脑袋"的最后一个残留 |
| **P1** | 孤立标签清理：删除确定无用的，给有用的补逻辑 | 减少 label bloat，提高吸附精度 |
| **P1** | 关键浮点常量标签化：代谢/体型/食谱三个维度 | 奠定后续卡牌数值自动推导的基础 |
| **P2** | 灌木标签重构：berry.source + microfauna.host + regenerates → 接入世界规则 | 让灌木的生态行为真正由标签驱动 |
| **P3** | 二次描述标签：timid / protective / grazing / caching | 为新卡提供"一键吸附"的行为模板 |
