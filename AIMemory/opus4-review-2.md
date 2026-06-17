# Opus 4.8 — 第二次架构审查

> 上一次你指出了模拟不在 ECS 里、数字矛盾、EntityProfile 技术债、规划缺失四个致命问题。
> 这次审查：我们在你的建议基础上重建了整个项目。看看我们是否走在正确的路上。

## 零、请先读这些文件

```
AIMemory/design-philosophy-v5.md    ← 完整设计哲学（特别关注 §12 标签纯度、§13 抽象深度）
AIMemory/tech-stack.md              ← 技术栈和关键 API
memory/FACT.md                      ← 铁律
AIMemory/current.md                 ← 当前状态

assets/tags.ron                     ← 17 维 ~184 标签的完整定义
assets/card_defs.ron                ← 36 张卡（7 地形 + 11 植物 + 16 动物 + 2 水生）

src/tags.rs                         ← TagRegistry + TagBits 实现
src/card_def.rs                     ← CardDef 加载 + 标签位掩码转换
src/initial_spawn.rs                ← 世界生成（同心环 + 动物散落）
src/systems/main_tick.rs            ← tick 六阶段管线
src/meta_values.rs                  ← A/B 层元数值常量
src/meta_actions.rs                 ← 25 元动作枚举

src/need_match/                     ← 需求匹配引擎（data/activation/search/execution/engine）
src/perception/                     ← 四通道感知系统
src/memory/                         ← 流动记忆系统
src/render/                         ← 渲染层（skybox/terrain/vegetation/scatter/animation/card_base）
```

## 一、上次你的四个致命发现——我们怎么处理的

**1. 模拟不在 Bevy ECS 里：** ✅ 留在 WorldState 串行循环。没迁进 ECS。main_tick 六阶段管线明确。

**2. 数字矛盾：** ✅ 1 day = 420 tick 统一。meta_values.rs 修正。工作流补了数值一致性检查。

**3. EntityProfile 技术债：** ✅ 标注了待删除。新建了 TagRegistry + TagBits 位掩码系统替代字符串匹配。

**4. 规划缺失：** ✅ 行为惯性 + 选择阈值 1.2× + 知识链分解。中间步骤受保护不被贪心匹配器丢弃。

## 二、本次审查重点

### A. 设计层面

**A1. 抽象深度标准（design-philosophy-v5.md §13）是否自洽？**

我们基于 BFO 四原则（Adequatism/Perspectivalism/Realism/Fallibilism）制定了一套抽象深度统一规则：
- 同类内所有成员共用同一标签模板
- 不同类别可用不同维度（动物用 diet，植物用 nutrition）
- 粒度上限=器官级，下限=不模拟细胞/分子

请审阅 card_defs.ron 中动物卡和植物卡的标签集——它们是否严格遵循了这套标准？有无遗漏或越界？

**A2. 标签纯度（§12.2）——五条标准是否有效？**

我们定义的五条纯度标准：不可含物种名、可传递、可组合、趋同验证、行为闭包。请检查 card_defs.ron 中随机抽取几张动物卡——这些标签能否通过五条测试？有没有"低纯度"标签混入？

**A3. 生态自洽性**

我们现在有 16 种动物，分布在七环地形上。从生态学角度——这个食物网能否闭环？捕食者-猎物比例是否合理？有没有物种会必然灭绝或泛滥？

### B. 代码层面

**B1. TagRegistry + 位掩码实现**

当前 `src/tags.rs` 手工定义 ~300 个 TagInfo 常量 + `default_registry()`。`assets/tags.ron` 是设计文档但不参与编译。这种"手工常量 + RON 文档"的双轨方案是否可行？还是应该立即做 build.rs 自动化生成？

**B2. need_match 引擎与标签的对接**

动物的 Need/Perceive/Decide 通过 `init_animal_needs()` 和 `init_animal_knowledge()` 从标签派生。这个"标签→需求/知识"的映射逻辑是否正确？有没有遗漏的标签组合会导致动物行为异常？

**B3. 性能预判**

当前：36 张卡 × 32×32 格 = 环境+动物约 200-300 个实体。need_match 管线已接入每个 tick。
- 这个量级在 420 tick/day 预算内吗？
- 未来扩展到 5000 实体时，最大的瓶颈在哪里（感知？Decide？渲染？）
- 分批错峰调度（DF 模式）是否已应用？

### C. 架构风险

**C1. 我们是否在向"远离车间"的设计目标前进？**

设计的核心承诺：玩家不是车间主任——NPC 自主行动，玩家通过手牌干预。当前代码中——动物的行为决策是否真的由标签驱动（无 if-else 物种判断），还是存在隐式的物种硬编码？

**C2. 叠加域是否在代码中实现了？**

设计说"标签不调用行为，标签调制行为参数"。当前 need_match 引擎的 `init_animal_knowledge()` 里——`if has_tag("diet:herbivore") { add("吃草") }` ——这算"标签调制行为参数"还是"标签调用行为"？边界在哪？

**C3. 最大的未实现缺口是什么？**

36 张卡 + 渲染 + 需求引擎都有了——但生态真的能跑起来吗？有没有我们以为已连接但实际上断开的管线？有没有关键系统被遗漏？

## 三、输出要求

写入 `AIMemory/opus4-review-2.md` 的**响应段**（在文件末尾追加 `## Opus 响应`，不要覆盖原文）。

对每个 A/B/C 问题给出判断。关注"我们现在最该停下来修什么"——而不是"你们做得很好"。

**也请自由分享你对这个项目的整体看法。** 一个标签驱动的卡牌生态模拟——它在你见过的大量项目中处于什么位置？你现在对这个项目的直觉判断是什么？不必局限于我们列的问题。

先读文件再回答。

---

## Opus 响应

> 第二次审查。我把整条 tick 管线、需求引擎、感知层、初始生成、标签系统、36 张卡逐文件读完了。
> 先说结论，再逐题。

### 总判断（先读这一段）

**上一次的四个致命问题，你们修对了"形状"，但没修"接通"。**

- ECS：✅ 真的没迁，串行循环清晰。这条是真解决了。
- 数字矛盾：✅ `TICKS_PER_DAY=420` 全仓统一，`meta_values.rs` 干净。真解决了。
- EntityProfile：⚠️ 标了待删，建了 `TagRegistry/TagBits`。方向对，但旧 `profile.rs` 还在（`SmallVec<[NeedState;4]>` 还挂在那），双轨并存。
- 规划缺失：❌ **这条是假修复。** 行为惯性、1.2× 阈值、知识链分解的代码都写了、单测都绿，但它们在真实管线里**一次都没真正跑通过**——见下。

**最重的发现（C3）：生态现在是冻结的。一只动物都不会动、不会吃、不会繁殖、不会死。**

我把整条因果链追到底，每一个"接缝"都是 stub：

1. `main_tick.rs` Phase 4 / Phase 5：`let env: Vec<...> = Vec::new(); // TODO` —— 环境向量**永远是空的**。执行引擎拿到的世界是空的。
2. `search.rs::count_prerequisites_met()` —— stub，**永远返回 0** → achievability 永远 = 0 → arbitrate 的 score 永远 = 0 → 永远过不了 `> 当前分 × 1.2` → **带前提的需求永远建不出计划**。
3. `init_animal_knowledge()` 里"吃草"/"捕猎"的 `decomposition: vec![]` —— **空的**。即使建出计划，零步骤 → `tick_execution` 返回 None → `Pause`。
4. `execution.rs` 的 Act 步骤 → `Strike { target: EntityId(0) }` / `Consume { target: EntityId(0) }` —— **目标硬编码成幽灵实体 0**，和感知到的真实目标没有任何绑定。
5. Acquire 成功 → `Move { dx:0, dy:0 }` —— **原地踏步**。整个系统里没有任何"朝目标走一步"的元动作派生。
6. `perception/senses.rs` 视觉：捕食者/猎物判定都写成 `target_tags.has(0)`（bit 0 是 BODY，占位符）→ **永不触发**；只剩无条件的 Curiosity 0.1。
7. `main_tick.rs` 嗅觉：对每个邻居硬编码 `odor_type = "food"` → 所有邻居都"闻起来像食物"，给所有实体灌 Nutrition 急迫度（虚假信号）。
8. `tick_starvation.rs` = `pub fn tick_starvation(_world){}` —— **空函数，没人会饿死**。需求衰减到 1.0 后永远停在 1.0。
9. `tick_reproduction.rs::parse_repro_params` 只认旧词表（`repro_spawn:`、`require_grass`、`pop_cap`）。新卡用的是 `repro:few_offspring` / `repro:egg_layer`——**一个都不被解析** → 对 36 张新卡返回 None → **零繁殖**。
10. **桥接词表缺失：** 知识前提查询属性 `"edible"` / `"prey"`（`MaterialProperties.satisfies` 里 `tags.contains("edible")`），但 `tags.ron` 和 `card_defs.ron` 里**没有任何一张卡带 `edible` 或 `prey` 标签**。知识层说的语言和卡层说的语言是**不相交的**。哪怕 env 填满了，`satisfies` 也永远匹配不上。

结论：**单测全绿，是因为每个模块的测试都用手搓的 fixture 喂了真实管线永远喂不进去的数据**（engine 测试注入了带非空 decomposition 的知识图；execution 测试注入了填好的 environment）。绿色测试制造了"已集成"的错觉。**没有一个集成测试会生成真实世界、然后断言"某只鹿走到草边把草吃了"。** 那个本该是 smoke test——而 smoke test 现在是瞎的（见下）。

按 `memory/FACT.md` 的工作流铁律"0 移动 / 0 捕猎 → smoke 视为失败"，**这个项目当前就处于失败态**，只是 smoke 用了死词表，看不见而已。

---

### A. 设计层面

**A1. 抽象深度标准是否自洽？——基本自洽，但有三处实质偏差。**

- **personality 维度不一致。** 所有哺乳类 + 爬行类（鳄、蟒）都带 `personality:`，但两条鱼（鲤、鲶）**没有**。`§13.3` 的模板表里压根没列 `personality` 这一维。所以它是个"未登记却被不一致使用"的维度。要么它是"动物通用"（那鱼漏了 = 违规），要么是"哺乳/爬行专属"（那得写进 §13.3）。二选一，写死。
- **器官层级是"声明了但全卡未用"。** `§13.3` 把粒度上限定在器官级（"肝/心/肺可被砸伤"），`tags.rs` 也老老实实实现了整棵器官树（bit 0–49：skull/brain/heart/lung/liver…）。但 **36 张卡里没有任何一张带任何器官/body 标签**。于是"能被操作→必须建模"这条自己的标准被违反了：器官在引擎词表里存在，却没有任何实体拥有它们，"砸断一条腿"永远不可能发生。模板承诺器官粒度，卡只交付整体粒度。要么把器官树移出当前范围，要么让 `body_plan` 派生出器官——现在这是悬空的脚手架。
- **小瑕疵：** `sloth_bear` 带 `diet:frugivore` 但 foraging 只有 `:graze`，没有匹配食果的采食方式；属性集不闭合。

除此之外，同类内模板一致性是好的——所有有蹄类共用一套，捕食者共用一套，这部分做到了。

**A2. 五条纯度标准是否有效？——有效，而且这是整个重建里最强的一块。**

随机抽 `diet:herbivore` / `defense:flee` / `social:pack` / `body_plan:quadruped` / `personality:bold`：五条全过——无物种名、可传递、可组合、趋同（食草性/逃跑/集群在多谱系独立演化）、有行为闭包意图。`tags.ron` 引经据典（PanTHERIA / EltonTraits / Winemiller / Díaz），干净、无物种名混入。**没找到低纯度标签。** 唯一的类别模糊：`state:` / `injury:` / `personality:` 是"运行时状态/非性状"维度，混在同一个 registry 里——不违规，但"趋同验证"对 `state:starving` 这种根本不适用，概念上 trait 和 state 该分箱。这是小事。A2 判定：通过。

**A3. 生态能否闭环？——纸面角色齐全，但比例严重失衡；且代码上当前是零边图。**

- **代码层面：食物网当前没有任何一条边。** 因为上面 C3 的 #10（无 edible/prey 桥接）+ 零繁殖 + 零死亡，种群是**静止画**：生成一次，永不变动。捕食者-猎物比例在代码里是个无意义的问题。
- **纸面层面（假设接通后）：捕食者太多。** 大型食草 ≈ 54 只（水鹿 8 + 斑鹿 20 + 麂 15 + 印度野牛 5 + 犀 3 + 象 3），陆地顶级/中级捕食者 ≈ 17 只（虎 4 + 豹 3 + 豺 6 + 懒熊 2 + 蟒 2）外加鳄 3。捕食者:猎物 ≈ 1:2.7。真实系统按生物量是 1:100+。鹿是 `few_offspring` + `growth:medium/slow`，而繁殖根本没接（见 #9）。一旦接通，捕食者会在几周内吃光食草动物然后集体饿死——经典的"顶级捕食者过载"崩溃。建议 v1 把陆地捕食者砍到 1–2 种、或把食草动物数量 ×5，让比例进到 1:10 以上再说。

---

### B. 代码层面

**B1. 手工常量 + RON 文档双轨——当前这套是"最坏的两全"，建议现在就上 build.rs。**

`tags.ron` 文件头白纸黑字写着："build.rs 编译期生成 src/generated_tags.rs / 手写 Rust 标签代码 = 违反铁律"。但仓库里**没有 build.rs**（Glob 0 命中），而 `tags.rs` 是 **928 行手写常量**——也就是说，**项目正在违反自己写在 tags.ron 里的铁律**，而且那个 RON 文件**谎称自己是唯一真相来源**。证据：`tags.ron` 184 标签 vs `tags.rs` ~300 常量，**两者已经对不上了**。

判定：双轨**不可行**，不是因为双轨本身，而是因为文档假装自己有权威。两条诚实的路：
- (a) **现在就把 build.rs 做真**——184 个标签时这是半天工作；等长到 500 个再做就是一周的漂移调试。你们正要大量加卡，债务时钟在走。
- (b) 删掉 RON 头部的谎言，宣布 `tags.rs` 为唯一真相，反过来从 `tags.rs` 生成 `tags.ron` 供人阅读。

我倾向 (a)。不是"停下来一切"级别，但越拖越贵。

**B2. 标签→需求/知识 映射——需求侧 OK，知识侧有大洞。**

- `init_animal_needs`：尚可。Nutrition decay 从 `metab:` 派生（这是**正确的"调制"**）；Safety 恒在；Social 来自 pack/herd；Curiosity 来自 cognition。
  - **洞 1：Hydration 和 Rest 这两个 NeedKind 从来没被创建过。** `baseline_hydration()` 写了、枚举里有，但 `init_animal_needs` 永远不 push 它们。6 种需求里 2 种是"定义了但永不出生"。
- `init_animal_knowledge`：**只覆盖 13 种 diet 里的 3 种**（herbivore/carnivore/omnivore）。`piscivore`（水獭）、`insectivore`+`frugivore`（懒熊）、`detritivore`/`scavenger`（鲤/鲶）、`filter_feeder`、`nectar_feeder`… → **知识图为空 → 永远建不出任何计划 → 永久 Pause**。水獭、懒熊、鲤鱼、鲶鱼这 4 种**从出生起就是行为惰性**的。这正是你问的"有没有遗漏的标签组合导致行为异常"——有，而且是结构性的。

**B3. 性能——当前能扛，但 5000 实体会死在感知层，且 DF 错峰没做。**

- 当前 ~200–300 实体：每 tick 每个实体按 `MAX_SENSE_RANGE = 20`（切比雪夫）查邻居。**32×32 的棋盘上半径 20 几乎覆盖全图** → 空间索引基本不剪枝 → 实际是全配对 O(N²) ≈ 6–9 万次/tick，每次还做 2 个 `card_defs.get(&type_name)` **字符串键 HashMap 查找** + 2 次 perceive。300 实体大概 <15ms，420 tick/day 预算内，能跑。
- **5000 实体：感知层是瓶颈，没有悬念。** O(N²) = 2500 万次/tick × 420/天。Decide/Execute 是 O(N)、便宜；渲染走 Bevy 另算。放大因素有二：(1) `MAX_SENSE_RANGE=20` 让网格索引形同虚设；(2) 热循环里用 String 键查 card_defs。
- **DF 错峰调度：没做。** 每个实体每 tick 都感知 + Decide。`§11.6` 的"非当前层概率积分"只在设计文档里。三个动作项：① 把 `MAX_SENSE_RANGE` 收到真实单体上限（~6–8），让索引真的剪枝；② 把 `tag_bits` 缓存到 Entity 上，别每个邻居都按字符串回查 card_defs；③ Decide 错峰到每 N tick 一次（急迫度变化很慢，不需要每 tick 重算）。**这三件在扩规模前做，不是之后。**

---

### C. 架构风险

**C1. 在向"远离车间"前进吗？——方向对，但现在无法证伪，因为没有行为可被驱动。**

好的一面：决策路径里**没有** `type_name == "wolf"` 这种硬编码，路径是标签形状的。但：
- 行为是按 `diet:herbivore` / `diet:carnivore` 分派的——这是**更粗的"物种类"代理**，而且知识是按这个类硬塞的字面量，不是从标签组合涌现。
- `spawn_concentric_world` 里 `"abyss_pool"` / `"lotus"` / `"miscanthus"` 等 type_name 字符串硬编码——对种子世界可接受，但它**是** world-gen 里的 type_name 耦合。
- smoke_test 硬编码死掉的物种类字符串。

更根本的：**这个问题现在无法回答，因为根本没有行为。** 动物永久 Pause。当自治层只发 no-op 时，你没法宣称"标签驱动的自治"。车还没装引擎，问方向盘是机械连杆还是写死的，为时过早。

**C2. 叠加域实现了吗？——没有。`if has_tag("diet:herbivore"){ add("吃草") }` 是"标签调用行为"，不是"调制参数"。**

边界划清楚：
- **调制（正确，§1.2/§4）：** 标签给一个**通用机制**上的标量调权。本仓库里有正确的例子——`nutrition_decay = if metab:high {0.7} else if metab:low {0.2} else {0.4}`。吃这个机制对所有动物通用，`metab:` 只是缩放速率。
- **调用（`init_animal_knowledge` 干的事）：** 标签的**存在**本身是一个开关，注入一个离散的、命名的行为包（"吃草"，带硬编码的前提+效果）。这**字面上就是 §5.4 禁止的 `if-else 行为链`，只是拼成了 `if has_tag`**。

设计承诺的是：`predator + nocturnal + pack_hunter → 涌现`，无需额外规则。而这里，行为**不从标签组合涌现**，是按单个标签查表。`{graze + ground + herbivore}` 不会组合出一个参数化的吃草计划——`diet:herbivore` 一个标签就触发了罐头条目，`foraging:` / `foraging_stratum:` / `body_size:` 全被忽略。**所以叠加域没实现，实现的是一张按单标签分派的查表。**

干净的修法：知识应当是**派生**的——例如"任何实体，其 diet 标签的可食集 与 被感知实体的提供集 有交集，即可吃之"，magnitude 从 `metab` / `body_size` 派生。一条通用规则 + 标签参数化数据，**零 per-diet 分支**。这条在加更多 diet 之前修，否则你在手写一张随物种增长的行为表，把重建好不容易消灭的物种耦合又请回来了。

**C3. 最大的未实现缺口？——见总判断：生态是冻结的，几乎每条管线都是"以为接了实际断了"。**

"以为已连接但实际断开"的清单：上面总判断的 #1–#10，基本是全部接缝。被遗漏的关键系统：
1. **感知实体 → 元动作目标 的绑定**（最重要的缺件——没有它，Strike/Consume/Move 永远指向不了真实实体）。
2. **环境装配**（把感知到的邻居转成引擎吃的 `[(id, MaterialProperties)]`）。
3. **卡标签 → MaterialProperties / edible / prey 的投影**（桥接词表）。
4. **朝目标移动**（从目标 (x,y) 派生出 `Move` 单步）。
5. **新词表上的繁殖 / 饥饿死亡**。
6. **一个有意义的 smoke 闸门**（现在的 smoke 查 `"herbivore"` / `"predator"` / `"mesopredator"` / `"omnivore.small"`——新词表用的是 `diet:herbivore`，字符串对不上，**无论真实状态如何，smoke 必然报"全面崩溃"**。这意味着你们现在**没有任何回归信号**）。

---

### 现在最该停下来修什么（按优先级）

1. **打通一条完整闭环，给一种动物。** 选食草→吃草。让：感知发出真实的"(x,y) 处有食物 + 目标 id"；environment 向量从感知邻居填充；"吃草"知识有真实 decomposition `[朝目标走, Consume(目标)]`；Consume 绑定真实 id；`tick_starvation` 扣 hp 让"不吃"有后果。**用一个新的集成测试（外加修好的 smoke 闸门）证明一只鹿走到草边吃了草。** 在这条纵切跑通之前，16 张卡、器官树、社会、记忆，全是停尸房里的装饰。
2. **把 smoke 闸门修到新词表**（`diet:herbivore` 等），让"0 移动 / 0 捕猎 = FAIL"重新成真。现在这条铁律闸门被绕过了，你看不见自己在退步。
3. **用派生规则替换 per-diet 知识分支（C2）**，在加更多 diet 之前。
4. 然后：build.rs（B1）、捕食者:猎物再平衡（A3）、Hydration/Rest 需求 + 补齐其余 diet（B2）、扩规模前的性能（收 sense range、缓存 tag_bits）。

---

### 整体看法（你问的"直觉判断"）

这是个**哲学上异常认真**的卡牌生态模拟。BFO + 功能性状标签学 + 25 元动作那套本体工程，比我见过的 95% 的独立模拟和大多数学术玩具世界都更严谨。`tags.ron` 是真功夫，需求激活的数学（sigmoid 急迫度、安全阻断、metab 派生衰减）是干净且对的，元动作枚举利落。**架构的"形状"是对的，能撑起一个好游戏。**

但你们撞上了这个品类的经典死法——也是 Dwarf Fortress / Songs of Syx 一路走来的同一个坑：**本体在狂奔，运行的循环在原地。** 你们建了一座定义的大教堂，中殿却是空的。这次重建正确地修好了上次的**结构性**批评（管线形状、数字、标签系统），但**把"接好了线"误当成"跑起来了"**——接缝全是 TODO。

我的直白直觉：**这个项目的风险已经不是架构了，是执行纪律。** 是那种"再加一层就好了"的引力——器官！记忆！社会！——而不是逼一只动物把一棵草吃掉。你们现在大概在"它能跑"的 15%，却在"它被定义清楚了"的 80%。

最有价值的一条规矩，如果我只能给一条：**在 smoke test 于一条真实闭环上变绿之前，不许新增任何一张卡、一个标签、一个元动作。** 架构能成。能不能成为一个游戏，全看你们能不能抵抗本体论的引力，把集成的接缝一条条焊死。

