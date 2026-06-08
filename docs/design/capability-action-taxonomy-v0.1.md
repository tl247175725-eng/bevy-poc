# Capability / Action Taxonomy v0.1

本文件记录“世界规则标签机 / 卡牌封装”的下一层设计：**能力 Capability** 与 **行为 Action**。

目标不是增加抽象复杂度，而是把越来越多的内容收束到稳定结构中，让新增卡牌能自然进入已有世界规则，减少后续迭代成本。

## 1. 核心判断

卡牌不应是一堆独立代码分支。

一张卡应由以下部分封装：

- 静态标签：这张卡是什么
- 运行时状态：这张卡现在怎样
- 能力：这张卡原则上能做什么
- 行为规则：在什么条件下，对什么目标，执行什么行为，得到什么结果

标签只是入口，不是全部。

例如：

- `wolf` 和 `player` 都可以拥有 `capability.hunt`
- `wolf` 和未来的人类家庭成员都可以拥有 `capability.care_child`
- `wolf`、`sheep`、`rabbit` 都可以拥有 `capability.move`
- `wolf`、`sheep` 都可以拥有 `capability.reproduce`

差异不靠写死卡名，而靠物种、食性、群体、工具使用、风险策略等标签与能力组合表达。

## 2. 标签、能力、行为的分工

**标签 Tag** 描述稳定身份和属性：

- `animal`
- `predator`
- `prey`
- `juvenile`
- `diet.meat`
- `diet.plant`
- `diet.omnivore`
- `camp.storable`
- `organize.locked`

**能力 Capability** 描述一张卡原则上能进入哪些行为系统：

- `capability.move`
- `capability.hunt`
- `capability.forage`
- `capability.reproduce`
- `capability.care_child`
- `capability.define_domain`
- `capability.bond_to_domain`
- `capability.provide_service`
- `capability.produce_resource`
- `capability.transform_input`
- `capability.protect`
- `capability.attract_actor`
- `capability.store`
- `capability.trade`
- `capability.craft`
- `capability.use_tool`

**行为 Action** 描述实际可执行的动作模板：

- `action.approach`
- `action.flee`
- `action.follow`
- `action.return_home`
- `action.hunt`
- `action.feed`
- `action.guard`
- `action.lead_child`
- `action.collect`
- `action.place`
- `action.merge`
- `action.bind`
- `action.expand_domain`
- `action.produce`
- `action.transform`
- `action.serve`
- `action.craft`
- `action.trade`

原则：

- 标签回答“是什么”
- 能力回答“能进入哪个系统”
- 行为回答“现在做什么”
- 任务脚本只负责执行已经被规则选中的行为，不应自己到处重新解释卡牌身份

## 3. 示例：捕猎

`action.hunt` 不是狼专属，也不是玩家专属。

通用规则：

- actor 需要：
  - `capability.move`
  - `capability.hunt`
  - `diet.meat` 或 `diet.omnivore`
  - 当前存在食物需求，例如 `need.food`、`need.feed_child`
- target 需要：
  - `being`
  - 可作为肉食来源，或具备 `prey`
  - 不处于禁止捕猎状态
  - 可达
- 执行可能包括：
  - 接近目标
  - 攻击 / 击晕
  - 屠宰 / 搬肉
  - 回到进食点 / 巢穴 / 营地
- 失败恢复：
  - 目标消失：重新找目标
  - 路线被挡：清路或换路径
  - 食物已经足够：停止捕猎，转维护 / 回家

卡牌差异：

- 狼：不使用工具，优先带肉回窝，可为幼崽捕食
- 玩家：可使用工具，可烹饪，可按营地储备决定是否继续狩猎
- 未来杂食动物：可在 `diet.meat` 和 `diet.plant` 两套规则中择优

## 4. 示例：抚养

`action.care_child` 不应是幼狼专属逻辑。

通用规则：

- caregiver 需要：
  - `capability.care_child`
  - 成年或有效照顾者身份
  - 与 child 同物种、同群体，或存在明确照顾关系
- child 需要：
  - `juvenile`
  - `need.food`、`need.safety`、`need.home` 等需求
- 行为可能包括：
  - `action.feed`
  - `action.guard`
  - `action.lead_child`
  - `action.follow`
  - `action.return_home`
- 失败恢复：
  - 无家：跟随照顾者
  - 有家：走向家，不瞬移
  - 食物不足：照顾者进入觅食 / 捕猎

当前幼狼跟随成年狼、新狼窝出现后走向狼窝，应视为 `care_child + follow + return_home` 的早期具体实现。

## 5. 示例：域与羁绊

动物只是样例。世界规则标签机必须覆盖所有卡牌，包括结构、场域、资源源、经营单位、环境锚点。

**域 Domain** 描述一张卡对周围空间建立的规则影响。

例如篝火不是普通物品，而是营地锚点：

- 定义安全 / 活动范围
- 提供热源和烹饪点
- 驱赶部分危险
- 吸附草棚、桌子等营地结构

通用规则：

- anchor 需要：
  - `capability.define_domain`
  - 明确 domain 类型，例如 `domain.camp`、`domain.den`、`domain.resource`
- member 需要：
  - `capability.bond_to_domain`
  - 与 domain 类型兼容
- domain 影响：
  - 收纳范围
  - 睡眠许可
  - 经营范围
  - 安全 / 威慑范围
  - 生产或维护任务的目标选择

**羁绊 Bond** 描述卡牌之间的稳定结构关系。

例如：

- 草棚与篝火形成营地羁绊
- 桌子与篝火形成经营羁绊
- 蘑菇棚与草棚 / 营地形成生产羁绊
- 狼窝与狼群形成巢穴羁绊

羁绊不是一次性合成结果，而是后续规则查询的结构事实。

## 6. 非动物卡牌样例

篝火：

```text
tags:
  structure
  environment
  heat.source
  camp.anchor
  organize.locked

capabilities:
  capability.define_domain
  capability.protect
  capability.transform_input
  capability.bond_structure

domain:
  domain.camp

actions:
  action.transform(raw_meat -> cooked_food)
  action.transform(wood -> charcoal)
  action.expand_domain
```

草棚：

```text
tags:
  structure
  shelter
  home
  camp.fire_bond
  organize.locked

capabilities:
  capability.bond_to_domain
  capability.provide_service
  capability.expand_domain

services:
  service.sleep
  service.shelter
```

桌子：

```text
tags:
  structure
  businessUnit
  camp.fire_bond
  organize.locked

capabilities:
  capability.bond_to_domain
  capability.provide_service
  capability.attract_actor
  capability.trade

services:
  service.commerce
```

蘑菇棚：

```text
tags:
  structure
  mushroomFarm
  organize.locked

capabilities:
  capability.define_domain
  capability.produce_resource
  capability.attract_actor

domain:
  domain.mushroom_farm

actions:
  action.produce(mushroom)
  action.attract(mushroomFarmer)
```

狼窝：

```text
tags:
  structure
  animalHome
  den

capabilities:
  capability.define_domain
  capability.bond_to_actor
  capability.support_reproduce
  capability.provide_service

domain:
  domain.wolf_pack

services:
  service.home
  service.child_safety
```

森林：

```text
tags:
  rooted
  environment
  source.lumber
  source.twig
  organize.locked

capabilities:
  capability.produce_resource
  capability.regenerate
  capability.respond_to_tool

actions:
  action.produce(twig)
  action.transform_by_tool(axe -> wood/twig)
```

水源：

```text
tags:
  terrain.water
  source.water

capabilities:
  capability.produce_resource
  capability.accept_container
  capability.accumulate_pressure

actions:
  action.fill(bucket -> waterbucket)
```

## 7. 示例生物卡牌结构

狼：

```text
tags:
  animal
  predator
  diet.meat
  den.dweller
  autonomous

capabilities:
  capability.move
  capability.hunt
  capability.reproduce
  capability.care_child
  capability.return_home
```

羊：

```text
tags:
  animal
  prey
  diet.plant
  herd.animal

capabilities:
  capability.move
  capability.forage
  capability.reproduce
  capability.flee
```

玩家卡：

```text
tags:
  actor
  diet.omnivore
  camp.manager

capabilities:
  capability.move
  capability.hunt
  capability.forage
  capability.craft
  capability.store
  capability.trade
  capability.use_tool
```

## 8. 迁移原则

后续迁移顺序：

1. 先写 taxonomy，不急着重写运行时。
2. 在审计表中给现有卡补能力字段。
3. 选一个小切片迁移，例如 `hunt + care_child` 或 `camp domain + fire bond`。
4. 保持旧行为可验收：L0、L1、L2、L3 必须继续跑。
5. 每次迁移都必须能解释减少了哪些硬编码，而不是只增加抽象层。

禁止事项：

- 不允许为了抽象而抽象。
- 不允许把任务临时状态写成静态标签。
- 不允许一个标签只服务一张卡，除非明确标注为专名规则。
- 不允许行为规则绕开审计表和能力结构继续散落到各脚本中。
- 不允许只抽象动物行为，而让结构、场域、资源源继续游离在体系之外。

## 9. 下一步建议

下一轮优先做：

- 给 `card-rule-audit.md` 增加 `capabilities` 字段
- 为现有 50 张卡补首批能力标记
- 同时覆盖生物与非生物：`hunt + care_child` 和 `camp domain + fire bond` 都应进入候选样板

目标是让“狼捕猎 / 玩家捕猎 / 狼抚养幼崽”和“篝火定义营地域 / 草棚桌子并列羁绊 / 蘑菇棚生产域”都从散乱行为，逐步收束为同一套可解释的能力与行为规则。
