# 十、实现规则

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

### 自审五问（每次 push 前默读）

```
1. 这次改动中，有没有只对特定物种有效的逻辑？
2. 加一种全新的动物，需不需要改代码？
3. 所有数字能不能追溯到 meta_values 或真实数据？
4. 有没有绕过公理直接改 WorldState 的地方？
5. 这次改动的"为什么"和设计哲学一致吗？
```

### 不变量

```yaml
invariants:
  - "每个动物必须有 body_size 标签（恰好一个）"
  - "diet:* 标签只出现在 animal 实体上"
  - "nutrition:autotroph 只出现在 plant 实体上"
  - "所有裸数字必须追溯到 meta_values.rs"
  - "所有标签名必须在 TagRegistry 注册"
  - "library/ 对应章节和代码同步变更——同一 PR/commit"
  - "加新标签维度不能引入新的抽象深度级别"
```

### 图书馆维护纪律（铁律——每次设计确认后立即执行）

**用户说"确认"/"对"/"ok"/"认同"后，必须立即做三件事：**

1. **更新对应章节**——找到 `library/` 中对应的章节文件，加入新确认的设计内容
2. **更新术语表**——如有项目特有的新概念 → 加入 `_GLOSSARY.md`
3. **更新目录描述**——如涉及新系统/新维度 → 更新 `_INDEX.md` 中该章节的一句话描述

**不是"等 handoff 的时候再记"——是"讨论确认的那一刻就记"。**

违反后果：下次会话读到过时信息 → 基于错误前提做设计 → 浪费时间。

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
    - "library/ 对应章节和代码在同一个 commit"
```
