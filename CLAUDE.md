# 方寸商国：桃花源记

Bevy 0.15 标签驱动卡牌生态模拟。模拟在 WorldState 内自主运行，Bevy 只管渲染。

## 铁律（违反 = commit 被拒）

### 每次任务前必读
1. `AIMemory/current.md` — 当前状态
2. `AIMemory/shortcuts.md` — 快捷索引
3. `memory/FACT.md` — 铁律 + 三柱断点表
4. `AIMemory/tech-stack.md` — Bevy 0.15 API
5. `AIMemory/workflows/handoff-execution.md` — 三柱强制检查

### 代码铁律
- **禁止裸数字** — 所有数值引用 `src/meta_values.rs` 常量
- **禁止 type_name 匹配** — 用 `card_has_tag(def, "tag:name")` 替代
- **禁止按标签 if-else 链** — 用派生规则表或公理函数
- **禁止 EntityId(0)** — 用 `NONE_ID` 常量
- **禁止 unsafe / unwrap() / expect()**

### 三根柱子
- 标签描述形态 → `src/tags.rs` (TagBits + TagRegistry)
- 元数值描述强度 → `src/meta_values.rs` (唯一定义源)
- 元动作描述变化 → `src/meta_actions.rs` (25 元动作)
- 公理验证动作 → `src/axioms/`

### 工作流
- 设计→对齐→handoff(三段)→Agent 执行→验证
- 每次只改 1-3 个文件
- `cargo check + cargo test` 全 PASS 才能 commit
- push 前自动运行 `scripts/check-iron-law.sh`

### 关键常量
- Bevy 0.15（不是 0.16/0.17）
- TICKS_PER_DAY = 420（定义在 meta_values.rs）
- GRID: 32×32
- 1 tick = 0.5s 渲染帧
