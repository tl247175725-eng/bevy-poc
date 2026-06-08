# Card Rule Audit

> 人工审计表。基础事实以 `docs/design/generated/card-rule-facts.md` 为准；维度归类以 `docs/design/generated/rule-dimension-facts.md` 为准。

## 新卡接入 Checklist

新增或修改卡牌时，先完成这张表，再写任务脚本。目标是让卡牌靠标签自然进入世界结构，而不是让任务层记住一堆专名。

| 项 | 必填内容 | 守卫 |
|---|---|---|
| CardDB 定义 | `type`、中文名、tags、基础属性、商品值 | `generated/card-rule-facts.md` 能导出 |
| 工厂脚手架 | 运行 `tools/run_card_factory_scaffold.ps1 <type> ...` 生成粘贴片段 | `generated/scaffold/<type>/` 或 user:// 报告 |
| 工厂分级 | `factory_lane` 是否符合卡牌用途 | L0 factory lane 完整性 PASS |
| 标签维度 | identity / material_form / capability / relation_domain / action / rule_modifier 属于哪几类 | `generated/rule-dimension-facts.md` 有归类；新维度需先讨论 |
| 能力封装 | `CARD_CAPABILITIES` 是否登记稳定能力 | L0 audit integrity PASS |
| 收纳规则 | 是否 `camp.storable`；是否 `organize.locked`；两者冲突必须说明例外 | L0 tag conflict PASS |
| 寻路规则 | 是否可穿过、是否阻挡、原因是什么 | 优先走 `WorldRules.is_pathfinding_passable_occupant`；新增类别要补 L0 / L1.5 |
| 规则 IO | 作为输入、目标、产出参与哪些 relation / impact / handler | relation / impact 引用完整性 PASS；L1.5 smoke 可识别 |
| AI 可达性 | 玩家任务、生态行为、手动交互、环境自动、当前不可达 | 核心生产链节点必须有入口标记；例外必须登记原因 |
| 运行时状态 | 地上、手持、在窝内、眩晕、尸体、成熟等待等状态是否改变规则参与 | 状态判断必须走 `WorldRules` 谓词或登记局部 FSM |
| 专名规则 | 为什么不能只靠标签组合表达 | 写入本审计表 risk；任务层不得内联 `card_type ==` |
| 回归测试 | 至少 L0；影响主线时跑 L2b live | 记录到 `CHANGELOG.md` 和 `CODEX_HANDOFF.md` |

**默认策略：**

- 可搬且普通物资：优先考虑 `camp.storable`，并确认是否能被寻路穿过。
- rooted / shelter / businessUnit / domain anchor：默认结构锁定并阻挡。
- being / corpse：默认阻挡；驱赶、绕路、清尸必须通过行为规则处理。
- 未知未登记类别：默认阻挡；新增卡必须显式归入“可穿过”或“阻挡”。
- 只是“像某物”的新卡不要继承专名逻辑；先继承标签能力，例如 `material.lumber`、`capability.hunt`、`action.follow`。

| type | name | factory_lane | tags | capabilities | movable | storable | locked | rule IO | runtime state | AI reachability | risk |
|---|---|---|---|---|---|---|---|---|---|---|---|
| axe | 斧头 | production_node | sharp, tool, weapon, commodity, camp.storable | capability.be_carried, capability.be_stored, capability.use_tool, capability.hunt, capability.respond_to_resource | 是 | 是 | 否 | 作输入:砸tree→handler:chop_tree；作产出:tri+wood; tri+twig | 工具，可砍树 | 玩家任务/手动交互 | 砍树产物与森林生命值耦合 |
| berry | 浆果 | camp_resource | consumable, food, food.edible, basic, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed | 是 | 是 | 否 | 无直接规则 | 可直接食用/收纳 | 玩家任务/手动交互/生态 | 来源灌木规则需后续明确 |
| bucket | 空桶 | camp_resource | container, container.water, camp.storable | capability.be_carried, capability.be_stored, capability.accept_resource, capability.container | 是 | 是 | 否 | 特殊: 河流/浅滩取水 → waterbucket | 空容器，可取水 | 玩家任务/手动交互 | 空/满桶任务恢复要清晰 |
| bush | 灌木丛 | environment_source | bush, natural, shelter, organize.locked | capability.define_domain, capability.produce_resource, capability.shelter | 否 | 否 | 是 | 无直接规则 | cover，遮蔽/食物源，结构锁定 | 生态/玩家任务/手动交互 | 躲藏和采集职责需拆清 |
| charcoal | 木炭 | production_node | fuel | capability.be_carried, capability.fuel, capability.be_used_as_material | 是 | 否 | 否 | 作产出:wood+fire; twig+fire；特殊产出: waterbucket + fire | 火边木材产物 | 手动交互/环境产物 | 当前用途弱，设计待定 |
| coin | 铜钱 | production_node | currency, anchor, copper, camp.storable | capability.be_carried, capability.be_stored, capability.currency, capability.transform_input | 是 | 是 | 否 | 特殊: 与 fire → copperBlock | 货币，可熔铜 | 交易/手动交互/收纳 | 货币与普通物资收纳边界待经营区设计 |
| cookmeat | 熟肉 | production_node | consumable, food, food.edible, cooked, commodity, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_traded | 是 | 是 | 否 | 作产出:sheepMeat+fire; rabbitMeat+fire; deerMeat+fire; wolfMeat+fire; humanMeat+fire | 熟食，可直接食用/商品/收纳 | 玩家任务/手动交互/交易 | 食用和出售优先级需明确 |
| copperBlock | 铜块 | production_node | copper, material, camp.storable | capability.be_carried, capability.be_stored, capability.be_used_as_material | 是 | 是 | 否 | 作输入:grassRope→copperCraft；特殊产出: coin + fire | 铜材料，可合铜饰 | 手动交互/收纳 | AI 可达性待补，当前主要手动 |
| copperCraft | 铜饰 | production_node | consumable, craft, copper, novelty, commodity, camp.storable | capability.be_carried, capability.be_stored, capability.be_traded, capability.commodity | 是 | 是 | 否 | 作产出:copperBlock+grassRope | 商品 | 手动交互/交易/收纳 | 经营区陈列后需从普通收纳分流 |
| deer | 鹿 | actor_loop | being, animal, herbivore, wildPrey, largePrey | capability.move, capability.forage, capability.flee, capability.be_hunted | 是 | 否 | 否 | 无直接规则 | 大型野生食草，尸体链 | 生态/玩家任务/手动交互 | 狼饱腹时不应反复猎鹿；肉峰值需控 |
| deerCorpse | 鹿尸体 | structure_service | corpse, organic, organize.locked | capability.be_butchered, capability.sanitation_target | 否 | 否 | 是 | 无直接规则 | 鹿尸体，肉量，结构锁定 | 玩家任务/手动交互/生态 | 与羊尸同类屠宰/清尸规则 |
| deerMeat | 鹿肉 | production_node | consumable, food, food.raw, raw, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_cooked, capability.be_traded | 是 | 是 | 否 | 作输入:fire→cookmeat | 鹿肉生肉，deerMeat+fire→cookmeat | 玩家任务/手动交互/交易 | 纳入现有烹饪链，不单独烤鹿肉卡 |
| dryGrass | 干草 | production_node | dry, fiber, material | capability.be_carried, capability.be_used_as_material, capability.transform_input | 是 | 否 | 否 | 作目标:wood→hut; twig→hut; player砸→grassRope | cover/material，可合草棚/草绳 | 玩家任务/手动交互/狼窝破坏 | 覆盖层材料是否可搬需长期统一 |
| fire | 篝火 | structure_service | heat, environment, camp.anchor, organize.locked | capability.define_domain, capability.protect, capability.transform_input, capability.provide_service | 否 | 否 | 是 | 作目标:wood→charcoal; twig→charcoal; sheepMeat→cookmeat; rabbitMeat→cookmeat; deerMeat→cookmeat; wolfMeat→cookmeat; humanMeat→cookmeat；作产出:wood_like砸shard；特殊: coin → copperBlock; 特殊: waterbucket → charcoal+bucket | 营地锚点，热源，结构锁定 | 玩家任务/手动交互/环境 | 太多系统依赖，不应被普通关系误删 |
| grass | 草皮 | environment_source | grass, foodSource, organize.locked | capability.define_domain, capability.produce_resource, capability.be_used_as_material, capability.shelter | 否 | 否 | 是 | 作目标:wood→hut; twig→hut；特殊产出: waterbucket 浇地 | cover，食物源，结构锁定 | 生态/玩家任务/手动交互 | 既是地表又是合成输入，搬运语义需保持禁止 |
| grassRope | 草绳 | production_node | fiber, craftPart, material | capability.be_carried, capability.be_used_as_material | 是 | 否 | 否 | 作目标:copperBlock→copperCraft；作产出:player砸dryGrass | 材料/部件 | 手动交互 | 铜饰链可用，AI 可达性待补 |
| halfbucket | 半桶水 | manual_resource | water, container | capability.be_carried, capability.container | 是 | 否 | 否 | 无直接规则 | 半桶水 | 当前不可达/设计待定 | 有定义无明确规则，需决定保留或删除 |
| hammer | 锤子 | manual_resource | hard, blunt, tool, commodity | capability.be_carried, capability.use_tool, capability.transform_input | 是 | 否 | 否 | 作输入:砸shard→handler:shape_shard；作产出:stone+wood; stone+twig | 工具/钝器 | 手动交互 | 当前生产链有配方但 AI 可达性待补 |
| hoe | 锄头 | manual_resource | tool, commodity | capability.be_carried, capability.use_tool | 是 | 否 | 否 | 作产出:square+wood; square+twig | 工具 | 手动交互 | 当前生产链有配方但 AI 可达性待补 |
| humanMeat | 人肉 | production_node | consumable, food, food.raw, raw, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_cooked | 是 | 是 | 否 | 作输入:fire→cookmeat | 生肉，可烹饪/收纳 | 玩家任务/手动交互 | 伦理/玩法用途待设计确认 |
| hut | 草棚 | structure_service | shelter, structure, home, camp.fire_bond, organize.locked | capability.bond_to_domain, capability.provide_service, capability.shelter, capability.expand_domain | 否 | 否 | 是 | 作输入:mushroomWood→mushroomGreenhouse；作产出:wood+grass; twig+grass; wood+dryGrass; twig+dryGrass | 结构锁定，营地火绑定 | 玩家任务/手动交互/睡眠/菇棚 | 住所与菇棚合成目标重叠 |
| knife | 石制小刀 | manual_resource | sharp, tool, weapon, commodity | capability.be_carried, capability.use_tool, capability.hunt, capability.butcher | 是 | 否 | 否 | 无直接规则 | 可被 actor 携带为工具 | 玩家任务/手动交互/蘑菇农 | 屠宰/攻击/工具身份重叠 |
| lamb | 羊羔 | actor_loop | being, animal, juvenile | capability.move, capability.follow, capability.grow, capability.be_cared_for, capability.be_hunted | 是 | 否 | 否 | 无直接规则 | 幼体动物 | 生态 | 成长/捕食规则待持续审计 |
| mountain | 山 | environment_source | rooted, environment, stoneSource, source.stone, organize.locked | capability.define_domain, capability.produce_resource | 否 | 否 | 是 | 无直接规则 | rooted，结构锁定 | 环境源/手动交互 | source.stone 产出路径需后续显式化 |
| mushroom | 蘑菇 | camp_resource | consumable, food, food.edible, basic, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_traded | 是 | 是 | 否 | 无直接规则 | 可直接食用/收纳 | 玩家任务/手动交互/蘑菇棚 | 采菇与棚维护不能互相打断 |
| mushroomFarmer | 蘑菇农 | actor_loop | being, worker, autonomous | capability.move, capability.craft, capability.use_tool, capability.work_domain | 是 | 否 | 否 | 作输入:砸shard→handler:farmer_knife | worker，自主 | 生态/交易/手动交互 | 招募、备刀、工作目标需分层 |
| mushroomGreenhouse | 蘑菇棚 | structure_service | shelter, structure, mushroomFarm, organize.locked | capability.define_domain, capability.produce_resource, capability.attract_actor, capability.provide_service | 否 | 否 | 是 | 作产出:hut+mushroomWood | 结构锁定，蘑菇农来源 | 玩家任务/手动交互 | 已有棚时建棚链必须停建转维护 |
| mushroomWood | 长蘑菇的木头 | environment_source | wood, mushroomSource, material | capability.be_carried, capability.produce_resource, capability.be_used_as_material | 是 | 否 | 否 | 作目标:hut→mushroomGreenhouse; *砸→handler:harvest_mushroom | 菇木，可采菇/合菇棚 | 玩家任务/手动交互/环境计时 | 采菇和合棚目标冲突需靠任务上下文区分 |
| player | 玩家 | actor_loop | being, actor | capability.move, capability.hunt, capability.forage, capability.craft, capability.store, capability.trade, capability.use_tool, capability.carry | 是 | 否 | 否 | 作输入:砸dryGrass→grassRope | 手动代管、携带、饥饿、睡眠、眩晕会影响执行 | 玩家任务/手动交互 | 核心 actor，任务目标必须支持恢复 |
| playerCorpse | 遗体 | structure_service | corpse, organic, organize.locked | capability.be_butchered, capability.sanitation_target | 否 | 否 | 是 | 无直接规则 | 尸体，结构锁定 | 手动交互/生态 | 玩家死亡后生态清理规则需谨慎 |
| rabbit | 野兔 | actor_loop | being, animal, smallHerbivore | capability.move, capability.forage, capability.flee, capability.be_hunted | 是 | 否 | 否 | 无直接规则 | 小动物，可挡路/逃跑 | 生态/玩家任务/手动交互 | 狼清路应驱赶而非误杀 |
| rabbitMeat | 兔肉 | production_node | consumable, food, food.raw, raw, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_cooked, capability.be_traded | 是 | 是 | 否 | 作输入:fire→cookmeat | 生肉，可烹饪/收纳 | 玩家任务/手动交互/交易 | 来源和羊肉同类化 |
| shard | 碎石 | production_node | hard, material, material.shard, camp.storable | capability.be_carried, capability.be_stored, capability.be_used_as_material, capability.transform_input, capability.tool_input | 是 | 是 | 否 | 作输入:砸shard→handler:shape_shard；作目标:twig→spear; wood_like砸→fire; hard砸→handler:shape_shard; mushroomFarmer砸→handler:farmer_knife；作产出:stone砸stone | 可作为工具头、火种、临时刀源 | 玩家任务/手动交互/蘑菇农 | 既是材料又接近工具，标签需保持明确 |
| sheep | 羊 | actor_loop | being, animal | capability.move, capability.forage, capability.reproduce, capability.flee, capability.be_hunted | 是 | 否 | 否 | 无直接规则 | 动物，眩晕后可屠宰 | 生态/玩家任务/手动交互 | 攻击、驱赶、屠宰前提要分清 |
| sheepCorpse | 羊尸体 | structure_service | corpse, organic, organize.locked | capability.be_butchered, capability.sanitation_target | 否 | 否 | 是 | 无直接规则 | 尸体，肉量，结构锁定 | 玩家任务/手动交互/生态 | sanitation 不应被肉储备阻断 |
| sheepMeat | 羊肉 | production_node | consumable, food, food.raw, raw, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_cooked, capability.be_traded | 是 | 是 | 否 | 作输入:fire→cookmeat | 生肉，可烹饪/收纳 | 玩家任务/手动交互/交易 | 食物储备和清尸逻辑不能互相阻断 |
| spear | 长矛 | production_node | sharp, tool, weapon, commodity | capability.be_carried, capability.use_tool, capability.hunt, capability.defend | 是 | 否 | 否 | 作产出:twig+shard | 可被 actor 携带为武器 | 玩家任务/手动交互 | 专名规则：长矛只使用 twig |
| square | 正方碎石 | production_node | hard, toolHead | capability.be_carried, capability.be_used_as_material, capability.tool_input | 是 | 否 | 否 | 作输入:wood→hoe; twig→hoe; 砸shard→handler:shape_shard | 碎石塑形产物 | 手动交互 | 当前 hoe 链 AI 可达性待补 |
| stone | 石头 | production_node | hard, blunt, material.stone, camp.storable | capability.be_carried, capability.be_stored, capability.be_used_as_material, capability.transform_input | 是 | 是 | 否 | 作输入:wood→hammer; twig→hammer; 砸stone→shard; 砸shard→handler:shape_shard；作目标:stone砸→shard | 地上/手持都应可参与制碎石 | 玩家任务/手动交互 | 手持状态必须进入规则查询 |
| table | 桌子 | structure_service | businessUnit, structure, camp.fire_bond, organize.locked | capability.bond_to_domain, capability.provide_service, capability.attract_actor, capability.trade | 否 | 否 | 是 | 作产出:woodStruct+wood; woodStruct+twig | 结构锁定，营地火绑定 | 玩家任务/手动交互/交易 | 经营单位，不应被普通整理搬走 |
| traveler | 旅人 | actor_loop | being, customer, autonomous | capability.move, capability.trade, capability.consume_service, capability.attracted_by_service | 是 | 否 | 否 | 无直接规则 | customer，自主、饥饿、需求 | 生态/交易 | 需求生成与商品可达性要对齐 |
| tree | 树林 | environment_source | woodSource, rooted, forest, source.lumber, source.twig, organize.locked | capability.define_domain, capability.produce_resource, capability.regenerate, capability.respond_to_tool | 否 | 否 | 是 | 作目标:axe砸→handler:chop_tree | rooted，生命值、自然掉枝、砍伐掉落 | 环境源/玩家任务/生态 | source.lumber/source.twig 与实际掉落应持续对齐 |
| tri | 三角碎石 | production_node | hard, sharp, toolHead, material.tool_head | capability.be_carried, capability.be_used_as_material, capability.tool_input | 是 | 否 | 否 | 作输入:wood→axe; twig→axe; 砸shard→handler:shape_shard | 碎石塑形产物 | 玩家任务/手动交互 | 工具头专用，避免被当普通石材消耗 |
| twig | 树枝 | production_node | wood, fuel, fuel.fire, material, material.lumber, naturalDrop, camp.storable | capability.be_carried, capability.be_stored, capability.be_used_as_material, capability.transform_input, capability.fuel | 是 | 是 | 否 | 作输入:shard→spear; grass→hut; dryGrass→hut; fire→charcoal; 砸wood_like→woodStruct; 砸shard→fire；作目标:woodStruct→table; tri→axe; square→hoe; stone→hammer; wood_like砸→woodStruct；特殊: 与 waterbucket → wetWood | 地上/手持都应保持木材能力 | 玩家任务/手动交互/自然掉落 | 同时是通用木材和长矛专用输入 |
| waterbucket | 一桶水 | production_node | water, container, container.water, camp.storable | capability.be_carried, capability.be_stored, capability.transform_input, capability.container | 是 | 是 | 否 | 特殊: 与 wood/twig → wetWood; 特殊: 与 fire → charcoal+bucket; 特殊: 浇地 → grass+bucket | 水容器，可做湿木/灭火/浇地 | 玩家任务/手动交互 | 携带状态下任务目标容易错位 |
| wetWood | 湿木头 | production_node | wood, wet, material | capability.be_carried, capability.mature, capability.be_used_as_material | 是 | 否 | 否 | 特殊产出: material.lumber + waterbucket | 成熟等待，湿木变菇木 | 玩家任务/手动交互/环境计时 | 等待状态需要被 trace 识别为进展 |
| wolf | 狼 | actor_loop | being, animal, predator, autonomous | capability.move, capability.hunt, capability.reproduce, capability.care_child, capability.return_home, capability.carry | 是 | 否 | 否 | 无直接规则 | 捕食者，自主、入窝、叼肉 | 生态/手动交互 | 狩猎和回窝路线清障需分离 |
| wolfCorpse | 狼尸体 | structure_service | corpse, organic, organize.locked | capability.be_butchered, capability.sanitation_target | 否 | 否 | 是 | 无直接规则 | 尸体，肉量，结构锁定 | 玩家任务/手动交互/生态 | 清尸和取肉应同一套前提 |
| wolfCub | 幼狼 | actor_loop | being, animal, predator, juvenile, autonomous | capability.move, capability.follow, capability.grow, capability.be_cared_for, capability.return_home | 是 | 否 | 否 | 无直接规则 | 幼体捕食者，可入窝/跟随 | 生态 | 无窝时不能发呆或瞬移 |
| wolfDen | 狼窝 | structure_service | den, shelter, animalHome | capability.define_domain, capability.bond_to_actor, capability.support_reproduce, capability.provide_service | 否 | 否 | 否 | 作目标:*砸→handler:wolf_den | rooted，动物住所 | 生态/手动交互 | 破坏后幼狼/成年狼状态必须恢复 |
| wolfMeat | 狼肉 | production_node | consumable, food, food.raw, raw, camp.storable | capability.be_carried, capability.be_stored, capability.be_consumed, capability.be_cooked, capability.be_traded | 是 | 是 | 否 | 作输入:fire→cookmeat | 生肉，可烹饪/收纳 | 玩家任务/手动交互/交易 | 来源风险高但食物规则同类 |
| wood | 木头 | production_node | wood, fuel, fuel.fire, material, material.lumber, camp.storable | capability.be_carried, capability.be_stored, capability.be_used_as_material, capability.transform_input, capability.fuel | 是 | 是 | 否 | 作输入:grass→hut; dryGrass→hut; fire→charcoal; 砸wood_like→woodStruct; 砸shard→fire；作目标:woodStruct→table; tri→axe; square→hoe; stone→hammer; wood_like砸→woodStruct；特殊: 与 waterbucket → wetWood | 地上/手持都应保持木材能力 | 玩家任务/手动交互/树掉落 | 通用 material.lumber，需避免吞掉 twig 专名规则 |
| woodStruct | 木构件 | production_node | structure, material | capability.be_carried, capability.be_used_as_material, capability.structure_input | 是 | 否 | 否 | 作输入:wood→table; twig→table；作产出:wood_like砸wood_like | 木材敲击产物 | 玩家任务/手动交互 | 桌子链关键中间件 |

## 第一批高风险项

- 手持状态参与规则：`stone`、`wood`、`twig`、`bucket`、`waterbucket` 必须同时支持地上/手持两种状态。
- 手动关系与 AI 可达性：`hoe`、`hammer`、`copperBlock`、`grassRope` 当前偏手动，后续若成为主线必须补任务入口。
- 标签过宽/过窄：`wood` 与 `twig` 共享 `material.lumber`，但 `spear` 仍是 `twig` 专名规则，迁移时必须保留例外说明。
- 中间态等待：`wetWood` 到 `mushroomWood` 是计时状态，trace 必须把等待视为进展。
- 结构锁定：`table`、`hut`、`fire`、`mushroomGreenhouse`、地表 cover 不应进入普通收纳。
