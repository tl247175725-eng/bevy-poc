# Capability Migration Slices v0.1

本文件定义首批能力 / 行为规则迁移候选。目标是把 taxonomy 变成可落地的小切片，而不是一次性重写运行时。

当前候选只考虑两个方向：

- 世界规则结构：`camp domain + fire bond`
- 生物行为结构：`hunt + care_child`

两者都必须服务同一个目标：减少散落硬编码，让卡牌通过能力、域、羁绊和行为规则自然进入系统。

## 1. 候选 A：camp domain + fire bond

### 实施状态（2026-05-27）

第一段运行时迁移已完成，范围控制为“查询层 + 稳定调用点替换”，不重写任务 FSM。

已落地：

- `WorldRules` 新增 `domain.camp`、`domain.mushroom_farm`、`bond.fire`、`service.*` 查询。
- `WorldRules` 能回答：哪张卡定义营地域、哪张卡能吸附营地域、哪张卡提供烹饪 / 睡眠 / 经营 / 收纳 / 菇棚服务。
- `CampHelpers` 的 `holds_domain`、`has_fire_bond`、`bond_active`、`active_base_cells`、`bonded_hut` 改为消费 `WorldRules` 查询结果。
- 菇棚提供 `service.greenhouse`，但本轮不吸附进 `domain.camp`，避免把农业域误并入营地域。
- L0 增加域 / 羁绊 / 服务断言。

验证：

- L0：PASS，323 assertions
- L1 intent_trace：PASS，tick=1264 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=1725 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

剩余：

- 睡眠、交易、蘑菇棚维护还没有全部改成“服务查询驱动”；本轮只完成第一条运行时样板。
- `FIRE_BOND_DOMAIN_TYPES` 仍保留给旧规划/搬运分支作兼容，后续应继续收口为规则枚举。

### 目标

把篝火、草棚、桌子、蘑菇棚这组营地结构，从散落判断收束成“域 + 羁绊 + 服务”的结构。

### 抽象对象

- `domain.camp`
- `bond.fire`
- `service.sleep`
- `service.commerce`
- `service.cook`
- `service.storage`
- `service.greenhouse`

### 首批卡牌

- `fire`
- `hut`
- `table`
- `mushroomGreenhouse`
- `wood`
- `twig`
- `bucket`
- `waterbucket`
- `mushroomWood`

### 应收束的现有逻辑

- 篝火作为营地锚点
- 草棚 / 桌子与篝火的并列羁绊
- 睡眠必须依赖住所 / 安全域
- 交易桌必须处于营地域
- 蘑菇棚建造与维护应选择营地结构附近的合理目标
- 收纳和整理应读取营地域，而不是散落范围判断

### 不在本切片处理

- 不重写完整 path / movement
- 不重写完整 storage craft 链
- 不改蘑菇棚产出玩法
- 不新增建筑内容

### 成功标准

- 能从规则查询中回答：
  - 哪张卡定义营地域
  - 哪些结构绑定到营地域
  - 某张卡提供什么服务
  - 某个任务为什么选择这个结构作为目标
- L0 增加域 / 羁绊单测
- L1 / L2a 仍能建成蘑菇棚
- L2b 若仍失败，报告必须能说明是 mission 恢复问题，而不是域结构缺失

### 价值

这是“非动物卡牌也进入世界规则标签机”的第一条样板。它直接服务玩家主线：生火、建棚、睡觉、交易、收纳、蘑菇棚。

## 2. 候选 B：hunt + care_child

### 实施状态（2026-05-27）

第一段运行时迁移已完成，范围控制为“规则查询层 + 低风险调用点替换”，不重写完整生态 AI。

已落地：

- `WorldRules` 新增 `domain.wolf_pack`、`action.hunt`、`action.feed`、`action.follow`、`action.return_home` 常量。
- `WorldRules` 能回答：谁能捕猎、谁能被谁捕猎、谁能照顾幼体、幼体应跟随谁、狼窝是否是狼 / 幼狼的 home。
- 狼选择猎物从散落卡名判断改为 `WorldRules.is_hunt_target_for(wolf, card)`。
- 玩家普通狩猎目标选择改为 `WorldRules.is_hunt_target_for(player, card)`；本轮仍保持玩家不自动猎羊羔。
- 幼狼无窝跟随成年狼改为 `WorldRules.nearest_care_actor(cub)`。
- L0 增加 hunt / care_child / wolf_pack 断言。

验证：

- L0：PASS，339 assertions
- L1 intent_trace：PASS，tick=1454 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=1252 建成蘑菇棚，卡滞段 0
- L2 behavior_trace_live：PASS，tick=1443 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

剩余：

- 还没有把狼喂幼狼、回窝、建窝、玩家伏击草丛全部改成 action 查询驱动。
- `is_hunt_target_for` 内仍保留物种差异表；这是本轮允许的“专名边界”，后续应逐步抽到猎物偏好 / 风险规则。

### 目标

把狼捕猎、玩家捕猎、幼狼跟随 / 入窝，从散落行为收束为“捕猎能力 + 抚养能力 + 回家行为”的结构。

### 抽象对象

- `capability.hunt`
- `capability.care_child`
- `capability.return_home`
- `action.hunt`
- `action.feed`
- `action.follow`
- `action.return_home`
- `domain.wolf_pack`

### 首批卡牌

- `wolf`
- `wolfCub`
- `wolfDen`
- `rabbit`
- `sheep`
- `sheepMeat`
- `rabbitMeat`
- `player`
- `knife`
- `spear`

### 应收束的现有逻辑

- 狼有食物需求时才捕猎
- 狼有肉时优先回窝 / 喂养，而不是继续杀生
- 狼叼肉遇阻时清路而不是误杀
- 幼狼无窝时跟随成年狼
- 有窝后幼狼走向窝，不瞬移
- 玩家捕猎和狼捕猎共享“捕猎动作”的前提结构，但执行细节不同

### 不在本切片处理

- 不做完整 GOAP
- 不改所有动物生态
- 不新增繁殖系统
- 不把玩家全部 AI 改成通用 planner

### 成功标准

- 能从规则查询中回答：
  - 谁能捕猎
  - 谁需要照顾幼体
  - 谁可以作为捕猎目标
  - 为什么此时捕猎 / 不捕猎
  - 幼体为什么跟随或回窝
- L0 增加 hunt / care_child 单测
- 狼叼肉遇兔堵路专项 trace 仍通过
- 幼狼无窝跟随、建窝后步行入窝专项 trace 仍通过

### 价值

这是“行为不属于单张卡，而属于能力和行动模板”的第一条样板。它能直接降低狼与玩家捕猎逻辑的重复和分叉。

## 3. 推荐顺序

建议优先做候选 A：`camp domain + fire bond`。

理由：

- 它覆盖非动物卡牌，能验证“世界规则、所有卡牌”这个边界。
- 它更接近玩家主线：生火、草棚、桌子、蘑菇棚、睡眠、收纳、交易。
- 它能为 L2b live 失败提供更清楚的诊断结构：如果域和羁绊清楚，剩下的问题更可能是 mission 恢复 / 优先级恢复。
- 风险低于直接迁移捕猎和幼体行为，因为它主要先统一查询和解释，不必立刻改变生态动作。

候选 B 应作为第二个样板紧跟推进，用来验证同一套能力 / 行为结构也能覆盖动物与玩家行为。

## 4. 实施纪律

- 每次只迁移一个小切片。
- 不删除旧逻辑，先增加规则查询和断言，再逐步替换调用点。
- 每次迁移必须更新 `card-rule-audit.md`。
- 每次迁移必须说明减少了哪些硬编码。
- 每次迁移必须跑 L0；涉及玩家主线时跑 L1 / L2a / L2b；涉及生态时跑专项 trace 和 L3。
