# Current Handoff
- file: AIMemory/handoff_fix-click-kills-hidden.md
- mode: Standard
- status: ready

## 架构计划
左键点击草丛 → 两条攻击路径（detect_drag_smash 视觉重叠 + find_impact_target 格内查找）均不检查 in_cover → 藏匿兔被砸死。修复：apply_smash_hit 拒绝 in_cover + resolve_selection_card 排除 in_cover + detect_drag_smash 跳过 in_cover + cell.overlay 不可拖拽。

## 架构反馈
交互安全层缺失——smash 目标应检查可砸性（visible、not_in_cover、alive）。当前仅检查 hp>0 和 corpse 状态。

## 智能验收
- 点藏兔草丛兔子不死
- 藏匿实体不可选中
- 拖拽不误砸藏匿实体
- 正常砸功能不受影响
- cargo test + cargo build PASS
