# 玩家 AI headless + 湿地扩大 + 藏触发

**Priority**: P0

## 修复 1：玩家在 headless 模式不动
`ActionRunner::tick` 在 `tick_player_world`（headless）中只做"思考"——评估需求、选择意图——但 `interaction` 和 `events` 为空导致不执行移动。让 ActionRunner 在 headless 模式下直接 walk/wander/flee 而不依赖交互输入。

## 修复 2：湿地面积扩大 3 倍
`terrain_ecology.rs` 中 `WETLAND_OUTER_RINGS` 常量 ×3。

## 修复 3：藏触发条件
当前 `drive:hide` 要求 predator 在范围内才触发。但实际行为应该更自然——兔子本能地待在草里，不只是"有狼才藏"。改为：草/灌木上的动物若无更高优先级 drive，优先 stay/idle（留在掩护中）。

## 验收
- headless 模式玩家能走路觅食
- 湿地范围扩大 3×
- 兔子和鼠倾向于停留在草丛/灌木中
