# Current Handoff
- file: AIMemory/handoff_purification-phase4-meta-actions-impl.md
- mode: Standard
- status: executing

## 架构计划
新建 `src/meta_actions_exec.rs` — 8 个元动作的执行函数。exec_move/exec_strike/exec_consume 有完整实现 + 测试，其余暂返回 Invalid。lib.rs 注册新模块。

## 架构反馈
2 个文件改动（1 新建 + 1 编辑）。Combine/Release/Wait/Hide/Emerge 骨架留后续填充。

## 智能验收
cargo check + cargo test 全 PASS
