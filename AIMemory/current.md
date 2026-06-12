# Current Handoff
- file: AIMemory/handoff_purification-phase3-ruleindex-tags.md
- mode: Standard
- status: ready

## 架构计划
Phase 2 激活了 RuleIndex。Phase 3 清理其内部 type_name 硬编码：behavior_rule_matches 纯标签化、删除 def_tags type_name 注入、删除所有 legacy/dual_track 函数。

## 架构反馈
def_tags 注入 type_name 是最隐蔽的硬编码——type_name 以标签身份混入。删掉后缺标签的卡会暴露——补标签而非补 type_name。

## 智能验收
- behavior_rule_matches 无 type_name
- legacy/dual_track 全部删除
- ecosystem_behavior_key_legacy 删除
- cargo test + smoke test 全 PASS
- 相关卡定义标签完整
