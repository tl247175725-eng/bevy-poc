# 方寸商国：桃花源记 — 项目规则

你是 Cline，正在为"方寸商国：桃花源记"项目编写代码。这是一个 Rust/Bevy ECS 标签驱动卡牌生态模拟游戏。

## 铁律

1. **禁止 type_name 硬编码** — 用 `card_has_tag(def, "tag_name")` 代替
2. **禁止魔法数字** — 所有数值必须能追溯到 `game_constants.rs` 或标签定义
3. **禁止 if-else 行为链** — 行为必须通过标签驱动的规则引擎分发
4. **一格一卡** — compose 规则不可绕过
5. **曼哈顿移动** — 不允许对角线移动（单轴，dx 或 dy 一个非零）
6. **标签是存在** — 行为从标签涌现，不从 type_name

## 编译与测试

```bash
# 编译
cargo build 2>&1

# 测试（必须先全部通过）
cargo test -- --nocapture 2>&1

# 如果编译或测试失败：
# 1. 读报错信息
# 2. 修复问题
# 3. 重新编译/测试
# 4. 最多重试 3 轮
# 5. 3 轮后仍未通过 → 报告失败原因
```

## 关键文件速查

| 文件 | 作用 |
|---|---|
| `src/axioms/laws.rs` | compose/traverse/perceive/transform 四条公理 |
| `src/axioms/profile.rs` | 标签→属性解析 |
| `src/axioms/composition.rs` | CellComposition 格子占用 |
| `src/world_state.rs` | Entity 数据模型、WorldState |
| `src/systems/tick_reactive.rs` | 行为决策引擎 |
| `src/systems/movement.rs` | 移动系统 |
| `src/card_def.rs` | 卡定义数据结构 |
| `assets/card_defs.ron` | 所有卡的数据定义 |
| `src/game_constants.rs` | 全局常量 |
| `AIMemory/design-philosophy-v5.md` | 设计哲学（改动前必读） |
| `AIMemory/design_human-readable_v4.md` | 人读设计文档 |
| `AIMemory/design_machine-readable_v4.md` | 机读设计断言 |
| `AIMemory/tag-dictionary.md` | 标签字典（单点真相） |

## Git 提交规范

```bash
git add <具体文件>
git commit -m "<类型>: <简短描述>"
git push
```

类型: fix / feat / refactor / handoff / test
