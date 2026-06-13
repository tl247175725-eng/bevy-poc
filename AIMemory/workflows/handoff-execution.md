# Handoff 执行工作流

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

## Push 前必须 — cargo check

**改动代码后，必须先本地跑 `cargo check`。零错误才能 push。**

```
cargo check
```

- 只做类型检查，不生成代码，不链接——比 `cargo build` 快 5-10 倍
- 拦住 100% 的编译期错误（缺导入、函数不存在、类型不匹配）
- 首次跑会编译 Bevy 依赖的元数据（需要几分钟），后续增量只查改动文件（秒级）

**如果 cargo check 报错 → 修 → 再 check → 零错误 → 才能 push。绝对不允许推未通过 cargo check 的代码。**

## Push 前自查

1. `cargo check` 零错误（必须，不可跳过）
2. Grep 查所有被删函数/标签的残留引用（`src/` + `tests/`）
3. 检查 `card_audit.rs` 和 `tag_zh.rs` 是否注册了新标签
4. 确保零遗漏再 push

## Push 后

1. GitHub Actions 自动运行 `cargo test` + `smoke test`
2. 主动用 GitHub Token 查 CI 结果并报告用户
3. 不绿不继续下一步
4. 红了 → 读报错 → 修 → push → 循环
5. Handoff 写完后明确告诉用户执行指令

## Cline CLI 命令

```bash
cline -c "E:/桌面/bevy-poc" -p openai -m "deepseek-chat" --thinking high --retries 3 -y "编辑 src/xxx.rs: [具体要改什么]"
```
