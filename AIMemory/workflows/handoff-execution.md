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

## Cline CLI 命令

```bash
cline -c "E:/桌面/bevy-poc" -p openai -m "deepseek-chat" --thinking high --retries 3 -y "编辑 src/xxx.rs: [具体要改什么]"
```
