# Layer 0 意向词典与验收（判断层）

## 发展阶段（dev）

| ID | 含义 |
|----|------|
| `dev_survive` | 求生 |
| `dev_camp_stable` | 营地持家 |
| `dev_greenhouse` | 蘑菇棚闭环 |
| `dev_idle` | 无明确发展 |

## 意向 ID（Intent）

| ID | 显示 goal | 绑定 need |
|----|-----------|-----------|
| `survive_hunger` | 先吃饱再发展 | hunger |
| `prepare_wood` | 备木（制斧/砍树） | build_greenhouse |
| `gh_fetch_water` | 建菇棚·打水 | build_greenhouse |
| `gh_make_wet` | 建菇棚·做湿木 | build_greenhouse |
| `gh_wait_mature` | 建菇棚·等发菇 | build_greenhouse |
| `gh_assemble` | 建菇棚·合棚 | build_greenhouse |
| `camp_organize` | 整理营地 | cleanliness |
| `operate_greenhouse` | 打理蘑菇棚 | idle_scan |
| `idle_scan` | 观察周遭 | idle_scan |

## Layer 0 合格 Intent 序列（建菇棚）

```text
prepare_wood → gh_fetch_water → gh_make_wet → gh_wait_mature → gh_assemble
```

允许分支：

- 地图已有 `twig`：`prepare_wood` 可极短或跳过
- `eval=hunger` 且饥饿 ≥ SEEK：插入 `survive_hunger`，之后回到发展链

## 验收标准（过程 > 结果）

1. **能动性**：序列中出现 ≥2 次意向转换，且与 `progress_key` 前进一致  
2. **发展性**：`dev_greenhouse` 意向曾激活，且 `progress_key` 至少前进 2 档  
3. **一致性**：`goal` 与 `active_intent.id` 对应，不再长期「规划 / 敲山」错位  
4. **结果**：菇棚建成（次要，执行层修完后应满足）

## 测试

- 单测：`scenes/unit_test.tscn`（IntentPlanner + 态势）  
- 追踪：`scenes/intent_trace.tscn`（1 tick/帧，看 Intent 序列）  
- 报告：`user://intent_trace_report.txt`

## 问题记录原则

见 `layer0-intent-backlog.md`：只记录、归类，默认不修执行层。
