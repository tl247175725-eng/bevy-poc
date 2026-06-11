# 紧急修复：草皮堆叠 + 高度系统

**Priority**: P0（一格一卡被打破）

## 根因
草皮有 `capability.incorporeal` → compose 永远放行 → 无限堆叠。
其他 `incorporeal` 实体（粪便、腐殖土）同理。

## 修复

### 1. 草皮移除 incorporeal，改 height:flat

`assets/card_defs.ron` 中 grass：
```
移除 capability.incorporeal
添加 height:flat
```
同理 dung、humus 也改 height:flat。

### 2. compose 加高度判

```rust
fn compose(cell: &CellSlot, incoming: &EntityProfile) -> Composition {
    if incoming.incorporeal { return Allowed; }
    if incoming.is_corpse { return Allowed; }
    if cell.has_only_corpses() { return Allowed; }
    
    if cell.living_count > 0 {
        // height:flat 或 low 不阻挡 medium+
        if incoming.height <= Height::Low && cell.occupant_height <= Height::Low {
            // flat 和 flat 之间仍拒绝（两个草皮不能同格）
            return Denied;
        }
        if cell.occupant_height > Height::Low {
            // 已占格的是 medium+ → 拒绝
            return Denied;
        }
        // 已占格的是 flat/low，来者是 medium+ → 允许（踩上去）
        // 但已占格仍有 living_count 计数
    }
    Allowed
}
```

### 3. 生成者 spawn 前检查

`tick_producer_spawn` 中：草皮生成前检查该格是否已有 height:flat 实体。

## 验收
- 草皮不堆叠
- 粪便不堆积
- 动物仍能走上草皮（height:flat 不阻 medium）
