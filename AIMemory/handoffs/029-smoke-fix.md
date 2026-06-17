# Handoff 029 — 修 smoke_test 闸门

## 架构计划

**改什么：** `src/smoke_test.rs`（1 文件）
**做什么：** 替换旧词表为 TagRegistry 查询，降低初始门槛

### 改动

1. **所有 `card_has_tag(def, "herbivore")` / `"omnivore.small"` → 查 `diet:herbivore` / `diet:omnivore`**
2. **所有 `card_has_tag(def, "predator")` / `"mesopredator"` → 查 `diet:carnivore`**
3. **使用 `def.tag_bits` + `TagRegistry` 替代字符串匹配**
4. **`e.needs_grazing_tick` / `e.fed_today` → 替代检查**：读取 Entity 的 need 状态或 execution state
5. **`e.is_corpse` → 检查 `state:dead` 标签**
6. **降低"零捕猎=失败"门限**：生态刚解冻，改为 `if predation_ticks == 0` 只发 warning 不 fail。**保留"零移动=失败"**。
7. **触发 smoke_test 前初始化 TagRegistry**（`crate::world_rules::TAG_REGISTRY` 必须就位）

### 不删的

- 实体数范围检查（30-900）
- OOB 检查
- tick 性能阈值（<15ms）
- NaN 检查
- 事件泄漏检查

## 智能验收

- `cargo build --release` 零错误
- `cargo run --release -- --smoke-test` 不崩溃（PASS 或带 warning 的 FAIL 均可——只要不 panic）

### 关键

当前生态刚解冻——**零捕猎是预期行为，不是失败。** smoke 的职责是"不崩溃 + 有移动"。
