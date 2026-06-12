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
| day | 天 | 1 day = 24 hours（即 480 tick @ 2 tick/s） |
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

## 六、未来方向：从哲学僵尸到涌现世界

当前阶段的目标是:
1. 框架提纯: 确保所有逻辑走标签→公理→需求匹配→元动作的管线
2. 元动作+元数值: 建立不可分解的语法基础
3. 定义完备: 在语法基础上定义所有现有卡的行为

后续方向（不在当前范围）:
- 记忆系统: 实体记住"上次在这被狼追"→影响后续行为
- 社会层: pack 协作、flock 信息传递、territory 标记
- 经济层: 玩家商业行为作为"楔子"插入自转的生态系统
