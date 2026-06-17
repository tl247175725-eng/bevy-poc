# Handoff 执行工作流

## Handoff 生命周期

```
1. 自然语言对齐 → 用户确认方向
2. Claude 写 handoff（三段式）→ 用户审核三段
3. 通过 → DeepSeek Code (dsc) 执行
4. 编译器自动验证（clippy + test）→ 不绿不推
5. 回第 1 步
```

**关键原则：handoff 是我们共识的翻译，不是 Claude 的独白。**
用户不需要看 Rust 代码——编译器替用户看了。用户只需要审 handoff 的三段描述。

## 实施前强制分类（铁律——动手前第一问）

**"这个改动是修 bug 还是加机制？"**

| 类型 | 定义 | 流程 |
|---|---|---|
| **修 bug** | 代码行为和已确认的设计意图不一致（如：鹿应该吃草但管线不通、标签注册了但 has_descendant_of 返回 false） | 直接修，修完汇报 |
| **加机制** | 引入了新的游戏行为、新的数值体系、新的卡牌效果、新的生态规则——哪怕只有一行代码（如：狼没有繁殖逻辑，我加了繁殖 → 这是加机制） | **必须停下来讨论，用户确认后再写 handoff** |
| **边界模糊** | 不确定属于哪类 | **按加机制处理——停下来讨论** |

**红线：永远不要因为"跑通了再说"而私自加游戏机制。** 一个未经讨论的机制会变成后续所有设计的隐式前提——改它的成本是加它的一百倍。

## 三根柱子强制检查（铁律——任何任务前第一件事）

**元本质 = 标签 + 元数值 + 元动作。三根柱子，不需要第四根。**
每次接收任务、写 handoff、讨论实现方案前，**必须**逐条过以下检查。不通过 = 方案有架构问题，不能继续。

### 检查清单

对当前任务涉及的每一个行为（吃、移动、战斗、繁殖……），回答四个问题：

| # | 问题 | 验证方式 | 不通过的症状 |
|---|---|---|---|
| 1 | **标签**：行为的判断条件来自哪个实体上的哪个标签？ | 标签在 `tags.ron` 中已定义；判断用 `TagBits::has()` / `has_descendant_of()`；行为前提是**目标**的标签还是**执行者**的标签？ | diet→edible 查表、"has_tag: 幽灵字符串"不存在于任何卡牌、type_name 字符串匹配 |
| 2 | **元数值**：行为的强度/阈值/速率引用了 `meta_values.rs` 的哪个常量或函数？ | Grep `meta_values.rs` 确认常量存在；裸数字 = 违规 | `consumed = true`、`hp -= 1`、`decay_rate: 0.7`（裸数字） |
| 3 | **元动作**：行为对应 25 个元动作中的哪一个？ | `MetaAction` 枚举中已定义；Execute 步骤引用了该变体 | "吃"用 `consumed = true` 而不是 `MetaAction::Consume` 的完整执行 |
| 4 | **公理**：元动作执行前通过了哪个公理的验证？ | `src/axioms/` 中的 compose/traverse/perceive/transform 之一被调用 | 公理目录存在但无人调用；动作跳过公理直接改 WorldState |

### 检查时机

- **接收任务时**：先过四问，判断方案可行性。发现三柱子没接上 → 先修柱子，再做事。
- **写 handoff 时**：架构计划中必须写清楚"标签用哪个、元数值用哪个、元动作用哪个、公理用哪个"。
- **审查 dsc 产出时**：四问逐条验证代码。任何一问不通过 → 修正 handoff。

### 当前已知的三柱断点

| 行为 | 现状 | 断在哪 |
|---|---|---|
| 吃（Consume） | `diet_to_edible_tags()` 查表 + `target.consumed = true` | 标签（用幽灵字符串不是真实标签）、元数值（未调用 `baseline_energy()`）、元动作（Consume 未完整实现）、公理（transform 未参与） |
| 战斗（Strike） | `target.hp -= 1` | 元数值（裸数字 1 不是 meta_values 常量）、公理（未验证武器/材质） |
| 移动（Move） | 直接改 x,y | 公理（traverse 未验证曼哈顿约束、碰撞） |

**修一个新功能之前，先把对应行的三柱断点修通。**

## 写 Handoff

每个 handoff 必须含三段：
1. **架构计划** — 改什么，为什么，涉改文件列表
2. **架构反馈** — 暴露了什么架构问题，和设计哲学是否一致
3. **智能验收** — 写成可执行断言，能直接转为测试

前置检查：
- 这次任务如何复用/扩展公理/标签/元动作？
- 有没有违反设计哲学的地方？
- 新增数字能否追溯到 `src/meta_values.rs`？
- 必须引用设计文档（`AIMemory/design_*`）的具体行
- **涉及标签新增/修改 → 必须对照 `AIMemory/design-philosophy-v5.md` §13 抽象深度统一规则**（同类一致、五条纯度验证）

**架构讨论前强制阅读（铁律）：**
任何涉及 ECS 架构、元动作执行模型、需求匹配引擎数据结构、性能瓶颈的讨论前，必须先读以下文件理解执行链路：
- `src/main.rs` — App 入口
- `src/lib.rs` — 模块导出和调用关系
- `src/systems/main_tick.rs` — 模拟主循环

模拟不在 Bevy ECS 里。模拟是一个串行的 `main_tick(&mut WorldState, delta)` 调用的 for 循环。Bevy ECS 只管渲染和输入。任何"元动作该是 Component 还是 System"之类的问题必须先在执行链路的事实基础上验证。

**技术前提验证（铁律——每次 handoff 前必做）：**
任何 handoff 写之前，必须验证技术方案和项目现状兼容：
- 涉及新 crate → 确认 crate 能否完成这个具体任务（不是"大概能"）
- 涉及解析/序列化 → 确认格式和库的已知限制
- 涉及已有结构体 → 确认当前字段签名，避免编译后才报错
- 涉及文件格式 → 确认 Rust 端能否反序列化该格式的所有特性
不确认就写 handoff = 浪费 token 和时间。
验证方式：Cargo.toml 查已有依赖 → crate 文档查能力边界 → 已有代码 Grep 查签名。

**低多边形/渲染/模型生成 handoff 特殊规则（铁律）：**
凡是涉及以下内容的 handoff——不写概念描述，先找已有实现：
- Bevy 程序化 Mesh 生成
- 顶点色/顶点操作/GPU instancing
- 天空盒/粒子/光照/雾
步骤：搜索 Bevy crate/example → 找到和我们需求最接近的已有代码 → 适配我们的参数 → 写成 handoff。DeepSeek 改现有代码比从零写快 5-10 倍。

**外部 crate 版本兼容检查（铁律）：**
引入任何新 crate 前，必须检查其 Bevy 版本要求与项目当前 Bevy 版本是否一致。
- 查 crate 的 `Cargo.toml` 或 crates.io 页面上的 "Bevy support table"
- 版本不匹配 → 不引入。改为参考其源码自己实现
- 绝不为了用新 crate 而升级 Bevy 版本——Bevy 只是渲染壳子

**数值一致性检查（铁律）：**
凡涉及以下数值的讨论和修改，必须**同时检查并保持一致**：
- `src/meta_values.rs` — 代码常量定义（唯一数值来源）
- `memory/FACT.md` — 铁律中的数值
- `AIMemory/design-philosophy-v5.md` — 设计文档中的数值
三处出现同一数值时，必须一致。不一致时以 FACT.md 铁律为准，同步修正其余两处。
每次修改 meta_values.rs 的常量后，Grep 该常量的旧值在所有 `AIMemory/` 文件中的引用并更新。

## 改代码

### 安全规则
- **每次只改 1-2 个文件**。一个 handoff 只做一件事，CI 绿了再推下一个
- **禁止一次改 3 个以上文件**
- **不改已通过的代码**

### 新建 vs 编辑
- 新文件：DeepSeek (CherryStudio) 直接 Write
- 已有文件：DeepSeek Edit 或 Cline CLI
- Cline CLI 只编辑已有文件，不建新文件

### 改标签前
- **Grep 该标签字符串在 `src/` + `tests/` 中的所有引用**，确认全部已知后再改
- 改完后检查 `src/card_audit.rs` 和 `src/tag_zh.rs` 是否注册了新标签

## Push 前验证（本地，限 4 核，不卡电脑）

**改动代码后，本地跑两步。全部零错误才能 push。**

```
cargo check    # 类型检查，秒级，拦住编译错误
cargo test     # 全量测试，2 分钟，拦住行为回归
```

每步报错 → 修 → 重跑 → 零错误 → push。绝对不允许推未通过本地验证的代码。

## Push 前自查

1. `cargo check` 零错误
2. `cargo test` 全 PASS
3. Grep 查残留引用（`src/` + `tests/`）
4. 检查 `card_audit.rs` 和 `tag_zh.rs` 新标签注册
5. 确保零遗漏再 push

## GitHub 同步

GitHub 是代码备份 + 可选 CI。每次 handoff 完成后 push 到 GitHub 保持同步。

## Push 后

1. Handoff 完成时 push 到 GitHub 同步代码
2. GitHub Actions 自动验证（可选，本地已过）
3. 不绿不继续下一步

## Rust / Bevy 代码规范

以下规范写入每个 handoff 的架构计划，dsc 实现时必须遵守。

### Bevy ECS 惯用模式

```
Component（数据）: #[derive(Component)] struct, 纯数据无行为
  - ZST（零大小类型）用于标签类 Component，如 struct Predator;

Bundle（打包）: #[derive(Bundle)] struct, 组合相关 Component
  - 生成实体时使用 Bundle 而非逐个 insert

System（逻辑）: fn system_name(query: Query<&Component>) {}
  - 命名: 动词_名词，如 tick_hunger_decay、apply_move_resolution
  - 只读用 &，写入用 &mut
  - 运行条件: .run_if(in_state(GameState::Playing)) 限状态

Plugin（模块）: impl Plugin for XxxPlugin { fn build(&self, app: &mut App) {} }
  - 一个逻辑模块 = 一个 Plugin
  - Plugin 内注册该模块的所有 systems、resources、events
  - 大 Plugin 可组合子 Plugin
```

### 标签系统规范

```
- 标签必须是编译期注册（非运行时字符串匹配）
- 层级标签用位掩码存储：[u64; N] 内联数组，禁止堆分配
- 子标签包含父标签的位 → 子树检查 = 一次 bitmask AND，O(1)
- 标签定义统一来源：assets/tags.ron（RON 格式），构建时生成 Rust 代码
- 标签查询 API:
    entity.has_tag(Tag::Movement)        // 精确匹配
    entity.has_descendant_of(Tag::Body)  // 子树匹配（body → limb → bone → ...）
```

### 文件组织

```
src/
├── main.rs              # App 构建，插件注册
├── meta_values.rs       # 所有元数值常量定义（铁律：数字的唯一来源）
├── meta_actions.rs      # 25 元动作枚举 + 执行 trait
├── tags.rs              # 标签系统（位掩码、层级、注册）
├── axioms/              # 四条公理（compose/traverse/perceive/transform）
│   ├── mod.rs
│   ├── compose.rs
│   ├── traverse.rs
│   ├── perceive.rs
│   └── transform.rs
├── need_match/          # 需求匹配引擎
│   ├── mod.rs           # Plugin
│   ├── activation.rs    # 需求激活 + 基线计算
│   ├── search.rs        # 双方向搜索
│   ├── filter.rs        # 三层过滤（功能→可行性→风险）
│   └── arbitrate.rs     # 冲突仲裁
├── knowledge/           # 知识图
│   ├── mod.rs
│   ├── graph.rs         # 知识条目存储 + 查询
│   └── transactive.rs   # 社会交互记忆目录
├── body/                # 身体因果链
│   ├── mod.rs
│   └── causal_tree.rs
├── world/               # 世界规则
│   ├── mod.rs
│   └── rules.rs
└── card/                # 卡牌定义
    ├── mod.rs
    └── def.rs
```

### Rust 铁律

```
- 禁止 unsafe 代码
- 禁止 unwrap() / expect() 在生产代码中（用 ? 或 match）
- 禁止裸数字字面量——所有常量引用 meta_values.rs
- 禁止 type_name 字符串硬编码匹配
- 新增类型在独立模块文件，不在已有文件中追加无关类型
- 单元测试与代码同文件（#[cfg(test)] mod tests {}）
- 集成测试放 tests/ 目录
```

## dsc 执行 handoff

**前置：** 项目根目录已有 `.deepseek/config.toml`（API key）。

### 执行命令

**Claude 直接调用 dsc（不由用户操作）。**

```bash
# 标准 handoff 执行
dsc --debug --effort low -p "读 AIMemory/handoffs/xxx.md，按 handoff 改代码。改完跑 cargo check + cargo test，必须全 PASS。"

# 复杂多文件 handoff
dsc --debug --effort high -p "读 AIMemory/handoffs/xxx.md，按 handoff 改代码。改完跑 cargo check + cargo test。"

# 继续上次中断的会话
dsc -c
```

### 思考模式选择

| effort | 适用场景 | 费用 |
|---|---|---|
| low | 单文件、纯数据、简单迁移 | ¥0.005 |
| high | 多文件联动、跨模块 | ¥0.01 |
| max | 不用 | — |

**规则：** Claude 写 handoff 时标注 effort。大部分用 low。每次发送前确认。

### 预估时间（每个 handoff 必标注）
| 规模 | 时间 |
|---|---|
| 1 文件 | 1-3 分钟 |
| 1-2 文件逻辑 | 3-8 分钟 |
| 2-3 文件连锁 | 8-15 分钟 |
| 3+ 文件 | 15-25 分钟 |

超过预估 2× → 停止检查。

### dsc 实现后审查（铁律——cargo test 通过后必做）

**dsc 写代码时只看 handoff，不看铁律。** Claude 必须在 cargo test 通过后**读关键改动文件的 diff**，逐条检查：

1. **有无 if-else 按标签分支？** → 铁律禁止。应用派生规则替代。
2. **有无 type_name 字符串匹配？** → 铁律禁止。
3. **有无魔法数字？** → 所有常量必须引用 meta_values.rs。
4. **有无硬编码 EntityId(0) 占位符？** → 禁止。

任何一条不通过 → 立即写修正 handoff，不等用户发现。

### dsc 任务后清理（铁律——必须执行，先清理再汇报）
每次 dsc 任务完成后（无论成功/失败），Claude **必须先执行清理，再向用户汇报结果**：
1. `taskkill //F //IM cargo.exe 2>/dev/null` — 清理残留 cargo 进程
2. `taskkill //F //IM rustc.exe 2>/dev/null` — 清理残留 rustc
3. `taskkill //F //IM dsc.exe 2>/dev/null`  — 清理残留 dsc
4. `sleep 2` — 等待文件锁释放
5. 确认无残留后再跑 cargo check + cargo test

### dsc 超时规则（铁律）
- dsc 命令必须加 **timeout 上限**：`timeout 600 dsc ...`（10 分钟硬上限）
- 超过 10 分钟 → 自动终止，分析原因，决定是否重试或换方案
- **不要在超过预估 2× 后继续等——主动终止。**

### 进度报告（铁律）
- **每 5 分钟** Claude 必须主动检查 dsc 任务状态，向用户简报当前进展。
- 简报只需一句话："[任务名] — [X分钟]，当前在[做什么]，正常/异常"
- 连续 10 分钟无产出 → 报告用户，评估是否卡住或需要修改方向
- 检查方式：读 dsc 输出文件 + 检查目标文件是否生成/更新 + cargo check 快速验证
