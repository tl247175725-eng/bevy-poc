# Opus 4.8 — 架构审判

**你不是来写代码的。你是来审判这个设计的。**
我们花了一周时间用多个 AI 交叉验证了这个体系的理论完备性。现在需要你：
用你最强的推理能力，**找出这个体系落地时会死在哪**。

## 零、请先读这些文件

按优先级排序。读得越多，判断越准。

```
AIMemory/design-philosophy-v5.md          ← 完整设计哲学（全部读完）
AIMemory/workflows/handoff-execution.md    ← 代码规范和工作流
memory/FACT.md                            ← 铁律
src/meta_actions.rs                       ← 当前元动作枚举（仅 8 个变体，不是最终的 25）
src/meta_values.rs                        ← 当前元数值常量（shell，尚未扩展）
src/axioms/
  ├── mod.rs                              ← 公理引擎（关键：build_profile 函数）
  ├── laws.rs                             ← 四条公理实现（compose 有 type_name 硬编码！）
  ├── profile.rs                          ← EntityProfile 数据结构（遗留设计）
  ├── composition.rs                      ← 格子占用管理
  └── causality.rs                        ← 因果追踪
src/world_state.rs                        ← Entity 结构体定义（仍然用 hp: i32！）
src/world_rules.rs                        ← 标签查询原语（仍然用字符串 start_with 匹配）
src/card_def.rs                           ← 卡牌定义结构（CardDef 仍然有 hp 字段）
assets/card_defs.ron                      ← 5 张占位卡
src/systems/
  ├── movement.rs                         ← 碰撞/路径保留，AI 决策被 stub
  ├── tick_reactive.rs                    ← 从 1220 行纯化到 130 行骨架
  └── main_tick.rs                        ← 主 tick 调度
src/tag_zh.rs                             ← 标签中文化（417 行，字符串匹配）
src/sim_clock.rs                          ← 模拟时钟
src/spatial_index.rs                      ← 空间索引
src/smoke_test.rs                         ← 冒烟测试
src/lib.rs                                ← 模块导出
```

## 一、项目是什么

**方寸商国：桃花源记** — 标签驱动的卡牌生态模拟。
Rust + Bevy 0.15 ECS。同构现实 + 卡牌媒介兼容。

**核心赌注**：世界只需要三根柱子——标签、元数值、元动作。所有行为从这三者的交互中涌现，不需要 if-else 行为链，不需要配方表，不需要 type_name 匹配。

引擎不认识"刀"——只认识 {hard≥4, shape:blade, edge:present}。
引擎不认识"篝火"——只认识 {flammability>0.5, spark_source=true}。

## 二、已确定的体系（设计哲学 v5 完整记录）

### 2.1 元动作 — 25+2 个，7 层

物理(11)：Move, Strike, Consume, Combine, Separate, Constrain, Release, Pause, Hide, Emerge, Alter
信息(4)：Signal, Receive, Process, Store
生物(3)：Reproduce, Inherit, Transform
心智(3)：Decide, Teach, Bond
社会(2)：Assign, Commit
超自然(2)：Create, Destroy
底层(2)：Elapse, Chance

### 2.2 元数值 — A/B/C/D 四层

A 层（世界无法闭环）：tick/cell/mass/temperature/hardness/pH/感官参数等 ~50 个物理测量基元
B 层（涌现依赖）：energy/metabolism/heart_rate/need/attention/trust/norm_strength 等 22 个——经文献交叉验证
C 层（可选）：电磁/超自然参数
D 层（派生）：speed/damage/price/phase_state 等——不进元数值表

### 2.3 铁律

- 标签描述形态，元数值描述强度，元动作描述变化
- 无 HP——损伤作用在材质属性 + vital 标签上，标签即后果
- 身体因果树：父节点失效覆盖所有子节点
- 无配方：Combine 属性代数合并，知识图不写死材料
- Strike 统一（战斗=加工），材料真实物理值
- 1 day = 420 tick
- 所有数字追溯到 meta_values.rs
- 感知四通道（视觉/听觉/嗅觉/触觉），返回模糊度而非二元

### 2.4 需求匹配引擎（刚设计完，一行代码没写）

```
感知环境 → 需求激活 → 双方向搜索(A:需求→满足物 + B:环境→需求) 
→ 三层过滤(功能→可行性→风险，人格标签调制)
→ 冲突仲裁(紧迫度×可达成度，1.2×阈值)
→ 可行性失败时查社会记忆目录(Transactive Memory)
→ Decide → 元动作序列执行
```

## 三、审判问题（请逐条回答）

### 🔴 致命级

**Q1. 公理引擎的架构兼容性**
现在 `axioms/mod.rs:28-96` 的 `build_profile` 函数使用 `type_name: &str` 作为参数，通过类型名字符串匹配来解析属性。`laws.rs:38-39` 用 `incoming.type_name.ends_with("Corpse")` 来判断是否为尸体。

这是铁律禁止的 type_name 硬编码。如果我们把一切改为标签驱动——Entity 身上只有标签，没有 type_name 字符串——现有的公理引擎哪些能直接复用，哪些必须重写？`EntityProfile` 结构体（profile.rs）是否是遗留设计需要废弃？替代方案是什么？

**Q2. Entity 结构体的去 HP 化**
`world_state.rs:36-50` 的 Entity struct 有 `hp: i32`、`fed: bool`、`fed_today: bool`、`is_corpse: bool`、`ecology_state: EcologyState`。

我们的设计说"无 HP，标签即后果"。但当前代码 784 行的 world_state.rs 大量依赖这些字段。在 Bevy ECS 里，如何从"Entity struct 有 hp 字段"迁移到"Entity 只有标签+元数值，损伤由公理从材质属性计算"？`EcologyState` 枚举这个行为状态机还能存在吗？还是应该被需求匹配引擎完全替代？

**Q3. 元动作在 ECS 中的执行模型**
当前 `meta_actions.rs` 只定义了 8 个枚举变体（最终是 25），`ActionResult` 枚举表示结果。

核心问题：元动作在 Bevy ECS 中怎么执行？是：
(a) 每个元动作是一个 System，由 System ordering 编排？
(b) 元动作是 Component（例如 `MoveAction { dx, dy }` 作为 Component 挂在 Entity 上，由统一的 `execute_meta_actions` System 消费）？
(c) 自定义状态机，完全脱离 Bevy 的 System 调度？
(d) 其他？

注意：Elapse 和 Chance 这两个底层机制必须融入 tick 循环——它们不是 Entity 发起的，是世界的被动推进。

**Q4. 标签系统的最优实现**
我们需要：层级标签（body → limb → bone）、O(1) 子树检查、编译期注册、可动态添加（学习新知识）。

位掩码方案：`[u64; N]` 内联数组，父标签位包含所有子标签位 → 子树检查 = 一次 AND。
但学习（运行时新增标签）和编译期位掩码有矛盾——怎么解决？

替代方案：bevy-tag crate（GID u128, 8 层深度, TOML 编译期生成）vs 自建位掩码 vs 其他？
字符串匹配是当前实现（`world_rules.rs:25` 用 `t.starts_with`），我们知道这必须替换。但换成位掩码后，调试和序列化（存档）怎么处理？

### 🟡 严重级

**Q5. 需求匹配引擎的数据结构设计**
我们设计了一个完整的匹配链路，但从未讨论过它在 ECS 里的数据结构形态。具体：

- "需求"（need）是 Entity 上的 Component？全局 Resource？每个 need 一个 Component 还是一个 Needs container？
- "知识图"——每个实体有自己的一份知识条目。是`HashMap<Tag, KnowledgeEntry>` 作为 Component 挂在 Entity 上？还是全局 KnowledgeGraph Resource？
- 三层过滤（功能→可行性→风险）——是独立的 System 还是同一个 System 内的阶段？
- 需求基线的计算（baseline_nutrition = metabolism_rate × mass × species_factor）——这是 System 里动态算还是启动时预计算存进 Component？

**Q6. 性能瓶颈预判**
200 实体场景。找出真正会卡的地方：

- 感知系统：四通道 × 200 实体 × 15 邻居范围内的距离/遮挡计算——这是 O(n²) 吗？还是空间索引救了？
- 知识图匹配：触发频率低（Decide 只在需求跨阈值时触发），但 50 条目 × 5 需求 = 250 次标签比较——可以忽略
- 路径寻路：曼哈顿网格的简单寻路——A* 200 实体同时跑会怎样？
- 你认为最大的瓶颈是哪个？我们应该提前准备什么优化？

**Q7. 知识图的"玩"发现机制会不会失控**
B 方向搜索（环境→需求）：实体感知到一个红果子，查知识图不知道能不能吃，尝试 Combine → Consume → 中毒。这会产生一个新知识点"红果子=有毒-半可食用"。

问题：如果 200 个实体各自这样试错，会产生大量"垃圾知识"（试了 100 种不可吃的草，每种都存了一个失败记录）。知识图会膨胀吗？需要知识衰减/遗忘机制吗？

**Q8. 当前被掏空的代码——哪些该删，哪些该留**
大量系统被 stub 了：`tick_reactive.rs`（原 1220→130 行）、`movement.rs` 行为函数、所有 `tick_*.rs`。

在新的标签驱动架构下：
- `EcologyState` 枚举（Idle/SeekingFood/Fleeing/...）应该被需求匹配引擎完全替代——同意吗？
- `tick_reactive.rs` 保留了哪些基础设施（事件调度？时间推进？）还有价值？
- `world_rules.rs` 的标签查询原语（`card_has_tag`、`parse_tag_u32`）——换成位掩码后，这些函数怎么变？
- `card_audit.rs` 和 `tag_zh.rs`——标签系统换位掩码后，审计和中文显示怎么处理？

### 🟢 重要级

**Q9. 实现顺序推荐**
我们的约束：
- 每步必须 cargo check + cargo test 全 PASS
- 每次改动 1-2 个文件
- 冒烟测试必须通过（0 移动或 0 捕猎 = 失败）

推荐从哪开始？选项：
(a) 先标签系统（位掩码 + 层级），替换字符串匹配
(b) 先元数值体系（meta_values.rs 扩展为完整 A/B 层常量）
(c) 先 25 元动作枚举定义
(d) 先公理引擎去 type_name 化
(e) 先需求匹配引擎新建模块

**Q10. 超自然 Create/Destroy 和物理公理的兼容**
Create = 无前因产出新实体。Destroy = 从因果链彻底移除。
我们的原则是"超自然 ≠ 规则豁免，超自然 = 极端物理参数 + 公理覆盖"。

但 Create 在严格意义上就是豁免了物质守恒。在架构上，Create 应该被 compose 公理拦截吗？一个被 Create 出来的物体，compose 是否允许它进入已被占据的格子？Destroy 后的物体，因果链中引用它的条目怎么处理？

### 🔵 检查你的直觉

**Q11. 你认为这个体系最大的一个逻辑漏洞是什么？**
不是实现细节——是设计层面的。"这个假设不成立因为它忘了 X"，或者"这个过程中有一个跳步，Y 到 Z 之间的路径没有被元动作覆盖"。

如果找不到漏洞——那你认为这个体系**最弱**的一个环节是什么？最可能在什么时候崩塌？

**Q12. 一句话：这个架构能落地吗？**
如果答案是"能"——说一下前提条件。
如果答案是"悬"——说一下致命伤。
如果答案是"不能"——说一下为什么。

## 四、输出要求

写入文件 `AIMemory/opus4-guidance.md`。

**不要写代码。不要写伪代码。不要写实现示例。**
输出格式：对每个 Q 给出你的分析判断。判断比建议重要——告诉我们你看到了什么，而不是你应该做什么。

先读文件再回答。你的价值在于你能看到我们七个 AI 都看不到的东西。
