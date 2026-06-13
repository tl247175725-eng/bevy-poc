# 设计讨论工作流

## 讨论规则

- 新增或修改前必须读取 `AIMemory/design_*` 文件，避免前后矛盾
- 禁止 type_name 硬编码、魔法数字、if-else 行为链
- 所有数值必须能追溯到 `src/meta_values.rs`

## 结论分类

设计讨论收束时，向用户确认归属：

- **铁律** → 记入 `memory/FACT.md`（不可违反，6 个月后仍然有效）
- **暂定** → 记入对应 `AIMemory/design_*` 文件（当前阶段沿用，未来可改）
- **放着** → 不记录（知道有这个方向，但暂时不做）

## 记录位置

- 每次设计讨论的结论 → 写入对应 `AIMemory/design_*` 文件
- 铁律 → 写入 `memory/FACT.md`
- 哲学讨论 → 写入 `AIMemory/design-philosophy-v5.md`

## 每次讨论结束后

检查现有设计文档是否需要更新以保持一致。
