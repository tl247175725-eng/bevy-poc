# 修复 CI：2 个测试失败

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（CI 连续 3 次红）

## 架构计划
测试失败源于审计清理的标签驱动重构——收获/火焰逻辑改为标签后，测试中的断言未同步。纯测试修复，不碰核心逻辑。

## 架构反馈
标签驱动的收获/生产系统重构后，测试未能及时更新——需要建立"改代码→改测试"的强制检查点。

---

## 失败测试

### 1. `godot_harvest_pool_lotus` — lot harvest panic
文件: `tests/godot_parity.rs:50`
根因: 审计清理将 `tick_harvest.rs` 的物种硬编码改为标签驱动后，测试中的 lotus 卡牌可能缺少 `harvest_product:` 标签。

### 2. `godot_fire_on_water_becomes_charcoal` — charcoal 实体不存在
文件: `tests/godot_parity.rs:83`
根因: 火→木炭的转化可能受标签驱动改造影响，charcoal 未生成。

## 修复

1. 检查 `card_defs.ron` 中 lotus、charcoal 相关标签是否完整
2. 在测试中追踪标签链验证转化路径
3. 修复后跑 `cargo test --release` 确认全绿

## 验收
- `cargo test --release` 全部 PASS
- `cargo run --release -- --smoke-test` SMOKE: PASS
