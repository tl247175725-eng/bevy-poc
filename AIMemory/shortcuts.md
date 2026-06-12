# 工作流与规范 v6

> **三权**: 制作人（策划拍板）/ DeepSeek（设计+执行）/ GitHub Actions（编译测试）
> **工具链**: CherryStudio(DeepSeek) → Cline CLI(DeepSeek) → GitHub CI

---

## 核心原则

### 你的电脑只做一件事：跑游戏看效果
- ❌ 不在本地编译
- ❌ 不在本地跑测试
- ✅ 只跑 `cargo run` 启动游戏
- ✅ 编译测试全走 GitHub Actions 云端

### DeepSeek(CherryStudio) = 设计师 + 执行者
- 写 handoff + 建新文件 + 编辑代码 + 查 CI + git push
- Cline CLI = 辅助编辑器（只编辑已有文件，不建新文件）

### 所有改动必须走 CI
- git push → GitHub Actions 自动 cargo test + cargo build + smoke test
- 绿了 = 继续
- 红了 = DeepSeek 读报错 → 修 → push → 再查 CI

---

## 完整工作流（v6 优化版）

```
你（制作人）
    │ 讨论设计
    ▼
DeepSeek (CherryStudio)
    │ 设计 → 写 handoff → 改代码 → git push
    │ （新建文件 = DeepSeek Write，编辑已有 = Cline CLI 或 DeepSeek Edit）
    ▼
GitHub Actions（云端免费）
    │ cargo test --release
    │ cargo run --release -- --smoke-test
    │ 全绿 = ✅  红了 = ❌
    ▼
DeepSeek 查 CI 结果
    │ 红了 → 读报错 → 修 → push → 循环
    │ 绿了 → 告诉你
    ▼
你：本地 git pull → cargo run → 看游戏效果
```

---

## 日常指令

### 设计讨论
- `把这个思路记下来`
- `查一下业界做法`
- `帮我分析这段代码为什么 [现象]`

### 写代码
- `写 handoff` — 含架构计划 + 架构反馈 + 智能验收
- `直接改` — 简单改动不走 handoff，直接编辑文件 + push

### 验证
- `查 CI` — 查 GitHub Actions 最新运行结果
- `修 CI` — CI 红了，读报错并修复

---

## Cline CLI 使用规则

### 什么时候用 Cline
- ✅ 编辑已有文件（改函数、修 bug）
- ✅ 编译错误自修复循环

### 什么时候不用 Cline
- ❌ 创建新文件（Windows JSON 转义问题）
- ❌ 需要多文件协调的复杂重构

### 命令模板（终端中执行）
```bash
cline -c "E:/桌面/bevy-poc" -p openai -m "deepseek-chat" --thinking high --retries 3 -y "编辑 src/xxx.rs: [具体要改什么]"
```

---

## Handoff 规范

每个 handoff 必须包含三部分：
1. **架构计划** — 改什么，为什么，涉改文件列表
2. **架构反馈** — 暴露了什么架构问题，和设计哲学是否一致
3. **智能验收** — 写成可执行断言，能直接转为测试

### 前置检查
1. 这次任务如何复用/扩展公理/标签/元动作？
2. 有没有违反设计哲学的地方？
3. 新增数字能否追溯到 meta_values.rs？

---

## 禁止事项

- ❌ 本地跑 cargo test / cargo build（CPU 扛不住，走 GitHub CI）
- ❌ type_name 硬编码
- ❌ 魔法数字（一切数值追溯至 meta_values.rs）
- ❌ if-else 行为链
- ❌ Cline 建新文件（Windows 兼容问题）
