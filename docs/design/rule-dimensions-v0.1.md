# Rule Dimensions v0.1

本文件定义“世界规则标签机”的维度约束。目标不是增加术语，而是防止标签失控：新增标签、能力、动作、域、羁绊、服务时，必须先能归入一个维度。

## 1. 核心原则

标签可以描述很多东西，但标签之上必须有维度。维度就是世界语法。

如果一个新标签无法归入现有维度，不应直接加进 `CardDB`。这通常意味着两种情况之一：

- 新标签其实可以用现有维度中的标签组合表达。
- 现有维度不够，需要先扩展世界语法，再加标签。

## 2. 六个维度

| dimension | 说明 | 例子 |
|---|---|---|
| `identity` | 卡牌稳定身份：它是什么 | `being`、`animal`、`structure`、`tool`、`weapon` |
| `material_form` | 材料、形态、物理性质 | `material.lumber`、`sharp`、`blunt`、`container.water` |
| `capability` | 能进入哪个行为或规则系统 | `capability.hunt`、`capability.care_child`、`capability.provide_service` |
| `relation_domain` | 如何连接世界结构 | `domain.camp`、`bond.fire`、`service.sleep`、`source.lumber` |
| `action` | 当前可执行动作模板 | `action.hunt`、`action.feed`、`action.follow`、`action.return_home` |
| `rule_modifier` | 规则修饰、经济/生命周期/临时倾向 | `commodity`、`food.raw`、`organize.locked`、`juvenile` |

## 3. 标签组合优先于专名继承

新卡不应“继承狼”或“继承长矛”。它应由标签组合进入世界规则。

例如另一个偏好兔子的食肉动物，不应该写成狼的特例，而应表达为：

```text
being + animal + predator + capability.hunt + capability.move + preference.rabbit（后续偏好维度扩展）
```

长矛也不是特殊武器父类，而是：

```text
sharp + tool + weapon + capability.hunt + reach.1（后续规则修饰扩展）
```

当前系统还没有 `preference.*` 和 `reach.*` 的正式维度词汇，因此本轮仍把这些差异保留在 `WorldRules.is_hunt_target_for` 与 `hunt_target_score` 中，并在审计中标为后续迁移目标。

## 4. 专名规则的处理

专名规则允许存在，但必须说明原因，且不能替代标签组合。

当前保留的专名边界：

- 长矛只用 `twig`，不是所有 `material.lumber` 都能做长矛。
- 玩家普通自动狩猎只选 `sheep` / `rabbit`，本轮不自动猎 `lamb`。
- 狼捕猎偏好兔子，当前仍由 `hunt_target_score` 表达。

这些规则后续应逐步迁移为规则修饰维度，例如 `requires.twig`、`preference.rabbit`、`risk.near_fire`。

## 5. 行为结构

行为不只拆成“前提 / 目标 / 执行”，还必须有恢复段：

| stage | 说明 | 例子 |
|---|---|---|
| `availability` | 这个动作此刻能不能做 | 狼是否需要肉，玩家是否有武器 |
| `target` | 对谁做 | 找最近可猎目标，或按偏好排序 |
| `execution` | 怎么做 | 走近、伏击、攻击、取肉 |
| `recovery` | 被打断后怎么续上 | 手上有肉时回窝，手上有石头时继续制碎石 |

玩家卡显得犹豫，常见原因不是缺少动作，而是缺少 `recovery`：它不知道“手上的东西是否仍属于刚才的任务”。

### 5.1 已落地样板：捕猎 target 阶段

v2.37 已把捕猎行为的 `target` 阶段收口到 `WorldRules.best_hunt_target(actor)`：

- `availability`：仍由旧需求 / 生态逻辑判断，例如玩家是否需要肉、是否有武器，狼是否需要捕猎。
- `target`：由 `WorldRules.is_hunt_target_for`、`hunt_target_visible_to`、`hunt_target_score` 共同决定。
- `execution`：仍保留在玩家任务和狼行为中，例如走近、攻击、取肉。
- `recovery`：本轮暂不迁移，后续可从“狼叼肉回窝”或“玩家手持材料恢复任务”切入。

这一步的重点是证明：玩家和狼不是互相继承，而是共同消费 `capability.hunt` 与规则层 target 查询。新增捕猎者时应优先进入这套查询，而不是复制狼或玩家的专名代码。

### 5.2 已落地样板：回家 target/recovery 入口

v2.38 已把回家动作的目标选择收口到 `WorldRules.home_for_actor(actor)`：

- `availability`：actor 必须具备 `capability.return_home`。
- `target`：从 `domain.wolf_pack` 中寻找可作为 home 的卡牌。
- `execution`：仍由狼行为负责走向狼窝、入窝、潜伏。
- `recovery`：`homeDenId` 绑定优先于最近狼窝，用于恢复“这只幼狼 / 成狼原本属于哪个窝”。

这一步说明 home 不应是狼的专名查找函数，而应是 actor 和 domain 的关系查询。后续新增其它“有家”的生物时，应优先扩展 `is_home_for_actor` 和 `home_for_actor`。

### 5.3 已落地样板：玩家携带 recovery

v2.39 已把玩家手上物品的 recovery 收口到 `ActionRecovery`：

- `carry_intent(player)`：用 `WorldRules.bond_to_domain`、`is_camp_storable`、`is_organize_locked` 分类为 `bond_fire` / `store_camp` / `evict` / `none`。
- `can_resume_carry(player, craft)`：availability 查询，判断 craft 非冲突且存在可达 recovery 目标。
- `resume_carry(player, craft)`：统一 recovery 入口，顺序仍为 bond 搬桌/搬棚/搬火 → campOrganize 归置/清出。
- `execution`：仍由 `CampHelpers.resume_interrupted_bond_move` / `resume_interrupted_organize` 与既有 craft FSM 执行。
- **未迁移**：狼叼肉清路、storage orphan `on_arrive`、movement 单主人仍留待下一切片。

这一步说明 recovery 不应散落在 `CampHelpers` 与 `PlayerExecutionRecovery` 的专名分支里，而应成为标签机四段式中的独立查询层。后续狼 recovery 应作为第二 consumer 接入同一入口。

## 6. 工程约束

已落地：

- `CardRuleAudit.dimension_for_token()` 给标签和规则词汇归维度。
- L0 会检查 `CardDB` 标签、`CARD_CAPABILITIES`、首批运行时 `domain/bond/service/action` 是否全部能归维度。
- 自动事实表：`docs/design/generated/rule-dimension-facts.md`。
- 捕猎目标选择：`WorldRules.best_hunt_target()` 已作为行为 `target` 阶段的首个运行时样板。
- 回家目标选择：`WorldRules.home_for_actor()` 已作为 `target/recovery` 边界的运行时样板。
- 玩家携带 recovery：`ActionRecovery.carry_intent()` / `resume_carry()` 已作为行为 `recovery` 阶段的首个玩家侧样板。

新增卡牌或标签时的流程：

1. 先判断新词属于哪个维度。
2. 如果没有维度，优先尝试用现有标签组合表达。
3. 如果确实表达不了，先扩展本文件和 `CardRuleAudit` 的维度注册。
4. 再改 `CardDB` / `WorldRules`。
5. 运行 `tools/export_card_rule_facts.ps1` 和 L0。
