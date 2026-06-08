# 工程效率工具：卡牌审计 + 会话报告 + 实时面板

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-07
**Priority**: P1（加内容前的基础设施）

---

## 一、卡牌审计测试 `card_audit_tests.rs`

### 目标

加新卡 → `cargo test` 自动检查六维归类、能力完整性、颜色合法性。不依赖人肉 checklist。

### 检查项

| # | 检查 | 规则 |
|----|------|------|
| 1 | 每张卡至少 1 个 tag | card_defs.ron 加载后逐张检查 |
| 2 | 每张卡有 color 字段，且 RGB 非全零 | |
| 3 | 每张卡的 tags 全部在已知维度归类表中（identity/material_form/capability/relation_domain/action/rule_modifier） | 引用 `card_rule_audit.gd` 的六维表 |
| 4 | CARD_CAPABILITIES 中每张卡至少 1 个 capability | |
| 5 | 所有 capability 值在已知 CAP_ZH 中有中文映射 | |
| 6 | 所有 tag 值在已知 TAG_ZH 中有中文映射 | |
| 7 | display_name 非空 | |
| 8 | icon 非空 | |

### 实现

`tests/card_audit_tests.rs`，纯数据检查，不启动 Bevy App。读 `assets/card_defs.ron` + `src/capabilities.rs` + `src/tag_zh.rs`。

---

## 二、会话报告 `session_report.txt`

### 目标

关游戏时自动生成 `session_report.txt` 到项目根目录。内容：

```
方寸商国 会话报告
─────────────────
运行时间：XX 秒
游戏时间：第 X 日 HH:MM
总 tick 数：X
实体数：X（峰值 X）
生态事件：捕猎 X 次 / 繁殖 X 次 / 死亡 X 次
平均 tick 耗时：X.XX ms
最大 tick 耗时：X.XX ms
```

### 实现

`WorldState` 加 `SessionReport` 结构体，`main_tick` 末尾推统计数据。Bevy `AppExit` 事件或窗口关闭时写文件。

---

## 三、egui 实时状态面板

### 目标

在右侧面板日志之下新增一个折叠区"运行状态"：

```
▼ 运行状态
  实体数：187
  tick 耗时：0.34 ms
  活跃事件：5
  捕猎/繁殖/死亡：3/0/0
```

每秒刷新一次（不需要每帧）。数据直接从 `WorldState` 和 `SimClock` 读取。

### 实现

`game_ui_panel.rs` 的 egui 区加一个 `CollapsingHeader`，读 `TickStats` Resource。

---

## 四、涉及文件

| 文件 | 改动 |
|------|------|
| `tests/card_audit_tests.rs` | **新建**，8 条检查 |
| `src/session_report.rs` | **新建**，报告结构体 + 写文件 |
| `src/main_tick.rs` | tick 末尾推统计 |
| `src/game_ui_panel.rs` | egui 实时状态折叠区 |
| `src/main.rs` | 关窗口时调 `session_report.write()`

## 五、验收

- `cargo test`：167 + 8 = 175 PASS
- `cargo run --release`：egui 面板日志下出现运行状态
- 关游戏后在 `bevy-poc/session_report.txt` 看到报告

## 约束

- 不碰 M1-M4 逻辑层
- 不碰 Observer/Rete
- 记 FIX_LOG
