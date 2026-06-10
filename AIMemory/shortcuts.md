# 工作流与规范

> **治理体系**: 见 `AIMemory/governance.md`
> **三权**: 制作人（策划拍板）/ DeepSeek（设计+接洽）/ Cursor（执行）

### Cursor 改完代码必须做的事
1. `cargo run` — 编译并启动游戏确认效果。**没编译等于没改。**
2. 确认后 `git commit` + `git push`

## 让 DeepSeek 读代码查问题
- `系统性检查代码，一次性找出所有问题，给清单和优先级`
- `对比 Godot 和 Bevy 两套代码，列出不一致的地方`
- `读代码找出 [具体现象] 的根因`

## 让 DeepSeek 写执行单
- `写 handoff` — 每个 handoff 必须含"架构计划"和"架构反馈"两段
- `发 Standard`（发 handoff 给 Cursor，Standard 模式关 Max）
- `发 Max`（发 handoff 给 Cursor，Max Mode）

### Handoff 前置检查
1. 这次任务如何复用公理/标签/引擎？
2. 有没有暴露出架构底层缺陷？

## 问问进度
- `Cursor 在干嘛`
- `整体进度`

## 设计讨论
- `把这个思路记下来`
- `查一下业界做法 / 论文 / 有没有人做过类似的东西`

## 完整工作流

```
DeepSeek 写 handoff → push GitHub
    │
你切到 Cursor → 说 "读 current.md"
    │
Cursor 读 current.md → 找 handoff 文件 → 本地执行
    │
你立刻启动游戏看效果（本地改完了，不用等同步）
    │
确认 OK → Cursor git push 到 GitHub
    │
GitHub CI 自动编译 + cargo test + smoke test
    │
去 Actions 页面看结果：全绿 = 通过
```

### 关键规则
- **改代码在 Cursor 本地**——改完马上能跑游戏
- **编译测试在 GitHub**——你本地不编译，不吃 CPU
- **auto-pull 后台跑着**——保持文件同步
- **不需要 Cursor Automation**——已关闭
