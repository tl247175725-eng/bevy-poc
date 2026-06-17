# Handoff 004-d — 公理去 type_name 硬编码

## 架构计划

**改什么：** `src/axioms/laws.rs` + `src/axioms/mod.rs`（2 文件）
**为什么：** type_name 匹配违反铁律。Opus 确认这是最轻的改动——公理公式保留，只删两行。

### laws.rs 改动

`laws.rs:38-39`：
```rust
// ❌ 删掉
if incoming.type_name.ends_with("Corpse") {
    return Composition::Allowed { remaining: u8::MAX };
}

// ✅ 改为
if incoming.is_corpse {
    return Composition::Allowed { remaining: u8::MAX };
}
```

EntityProfile 已经有 `is_corpse` 或可通过 `Entity.is_corpse` 判断——检查当前代码中尸体判定方式，用已有的布尔字段替代 type_name 字符串匹配。

### mod.rs 改动

`build_profile` 函数签名：
```rust
// ❌ 删掉 type_name 参数
pub fn build_profile(
    entity_id: EntityId,
    type_name: &str,   // ← 删
    tags: &[String],
    ...
)

// ✅ 改后
pub fn build_profile(
    entity_id: EntityId,
    tags: &[String],
    ...
)
```

函数体内所有 `type_name` 引用：
- `profile::parse_size(tags, type_name)` → `profile::parse_size(tags)`（type_name 只是兜底值，删除不影响）
- `profile::parse_native_medium(tags, type_name)` → `profile::parse_native_medium(tags)`
- `profile::parse_bridges(tags, type_name)` → `profile::parse_bridges(tags)`
- `profile::parse_channels(tags, type_name)` → `profile::parse_channels(tags)`
- `type_name: type_name.to_string()` → `type_name: String::new()`
- 其他所有 `type_name` 参数传递 → 删除

注意：`EntityProfile` struct 仍有 `type_name: String` 字段——这是显示名（"玩家""狼""草"），不是逻辑判断用。保留字段但值可置空或从 tags 提取首个。

### 不做的

- 不修改 compose/traverse/perceive/transform 的纯函数公式
- 不删除 EntityProfile.type_name 字段（显示用）
- 不修改 profile.rs 的 parse_* 函数签名（后续 handoff）

## 架构反馈

**Opus Q1 确认：** "四条公理的数学公式可全部复用，输入契约必须重写。type_name 硬编码只有两处共两行。"
**这步完成后：** 公理引擎彻底与类型名解耦。compose 判断尸体走 `is_corpse` 布尔，任意实体可通过标签获得此状态。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 特别注意：compose 公理的行为不变——尸体仍然被允许进入已有实体的格子
