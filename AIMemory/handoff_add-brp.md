# 添加 Bevy Remote Protocol — 游戏实时监控

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P1（工具——让 AI 能读游戏运行状态，减少人工验证循环）

## 架构计划
Bevy 内置 RemotePlugin，不引入外部依赖。游戏启动时开放 BRP HTTP 端口。纯诊断工具不影响游戏逻辑。

## 架构反馈
无。

## 智能验收
- `cargo check` 0 错误
- 游戏启动后在浏览器打开 `http://localhost:15702` 返回 BRP 信息
---

## 实现

### 1. 注册 RemotePlugin

`src/plugins/mod.rs` 中 `AppPlugin` 的 build 函数：

```rust
use bevy::remote::RemotePlugin;

// 在 .add_plugins() 链中加
.add_plugins(RemotePlugin::default())
```

如果 RemotePlugin 不在 bevy 0.14，使用：
```rust
use bevy::remote::RemoteHttpPlugin;
.add_plugins(RemoteHttpPlugin::default())
```

### 2. Cargo.toml 不需要改

RemotePlugin 是 bevy 内置功能，不需要额外 crate。

## 验收

- `cargo run` 启动游戏
- 另开浏览器或 curl `http://localhost:15702` → 能看到 BRP API 信息
