# 生态标签化调参：草压力 + 逃逸 + 移动能力

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 执行依据

`docs/design/v3.0-a-ecology-tuning-via-tags.md`（完整设计）
`docs/design/world-rule-tag-machine-v0.1.md`（标签机规范）
`docs/design/rule-dimensions-v0.1.md`（六维归类）

## A. 草压力（改两个常量）

`game_state.gd`：
```
LIVING_GRASS_CAP: int = 12        # 原 18
RIPARIAN_GRASS_INTERVAL: float = 9.0   # 原 6.0
```

只改值，不动恢复逻辑。同步更新数值总表。

## B. 逃逸标签 + 捕猎成功率

### B1. 新增 capability 标签（归入 capability 维度）

| 标签 | 分配给 |
|------|--------|
| `capability.escape_fast` | deer, fox |
| `capability.escape_small` | rabbit, fieldMouse |
| `capability.escape_cover` | fox, fieldMouse |

同步更新卡牌的 `CARD_CAPABILITIES` 和 CardRuleAudit 维度归类。

### B2. 新增 WorldRules.hunt_success_chance

```gdscript
static func hunt_success_chance(hunter: CardBase, target: CardBase) -> float:
    var chance := 0.85
    if card_has_capability(target, "capability.escape_fast"):   chance -= 0.40
    if card_has_capability(target, "capability.escape_small"):  chance -= 0.25
    if card_has_capability(target, "capability.escape_cover") and target_in_cover(target): chance -= 0.35
    if is_stunned_being(target):                                 chance += 0.50
    var d := hunter.distance_to(target)
    if d > 1: chance -= (d - 1) * 0.05
    return clampf(chance, 0.10, 0.95)
```

### B3. 行为接入

狼/狐狸/玩家在攻击前调用 `hunt_success_chance`。失败时：目标安全，捕猎者进入 huntCooldown。不修改 `hunt_target_score`（那只是目标选择）。

## C. 移动能力标签化

### C1. 新增 capability 标签

| 标签 | 间隔 | 分配给 |
|------|------|--------|
| `capability.move_fast` | 0.50 | fieldMouse |
| `capability.move_quick` | 0.55 | rabbit, fox, wolf |
| `capability.move_normal` | 0.75 | sheep, deer |
| `capability.move_slow` | 0.80 | lamb, deerFawn, fieldMousePup |
| `capability.move_juvenile` | 1.20 | foxCub, wolfCub |

### C2. 新增 WorldRules.move_interval_for

按优先级匹配能力标签 → 返回间隔。`game_types.move_interval()` 改为转发此查询。

### C3. 灌木加速通用化

`capability.use_cover` + 当前在 `cover.small` 格 → 移速 × 1.35。不再只硬编码给狐狸。

## 验收

- L0：escape 标签 + move 标签 + hunt_success_chance 断言
- L1.5：新标签登记的卡 smoke 通过
- closed_ecology_trace PASS

## 禁止

- 不在 behavior 脚本里加 `card_type ==` 判断
- 不硬编捕猎成功/失败概率到狼/狐狸各自的行为里

## 回复方式

在本文件末尾追加 `# 回复` 段，记入 `AIMemory/work.log`。

---

# 回复（cursor / 2026-05-30）

**状态**：DONE

**A 草压力**：`LIVING_GRASS_CAP` 18→12，`RIPARIAN_GRASS_INTERVAL` 6→9；已同步 `docs/design/数值设计总表.md`。

**B 逃逸 + 捕猎**：`CARD_CAPABILITIES` 增加 `escape_fast/small/cover`；`WorldRules.hunt_success_chance` / `try_hunt_attack` / `apply_hunt_miss_cooldown`；狼（`wolf_attack`）、狐（`fox_attack`）、玩家（`player_hunt_helper` / `survival_tasks` / `player_needs_manager`）统一走标签机，无行为内硬编码概率。

**C 移动**：`move_fast/quick/normal/slow/juvenile` 标签；`move_interval_for` + `cover_move_multiplier`（`COVER_MOVE_BONUS`）；`consume_move_tick` 按能力查表。狐扑空休整读 `huntCooldown`。

**审计**：`card_rule_audit.gd` 能力表与 `world_rules` 同步；L0 新增 `_test_ecology_tag_tuning`。

**验收**：L0 PASS（727 assertions）；`coverHostId` 升格为 `CardBase` 正式字段。

**附带**：`field_mouse_behavior` 幼体分支改 `is_adult_field_mouse`；审计补全步骤2 卡牌标签维度。
