# 修复 landBug 尸体崩溃

**Priority**: P0（闪退）

## 根因
`landBug` 无 `corpse_type:` 标签，`corpse_type_for` 返回默认 "sheepCorpse"。但某处直接拼了 "landBug" + "Corpse" → 生成 `landBugCorpse` → card_defs 中不存在 → panic。

## 修复
在 `finalize_prey_kill` 中：昆虫（type_name == "landBug" \|\| "waterBug"）不生成尸体，直接 remove。昆虫无实卡，死亡即消失。

或者：给 landBug 加 `corpse_type: none` 标签，在 `finalize_prey_kill` 中识别 "none" → 不生成尸体。
