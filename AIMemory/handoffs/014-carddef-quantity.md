# Handoff 014 — CardDef 加 quantity 字段

## 架构计划

**改什么：** `src/card_def.rs`（1 文件）
**做什么：** CardDef struct 新增 `quantity: u32` 字段

### 改动

```rust
pub struct CardDef {
    pub type_name: String,
    pub display_name: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub color: (u8, u8, u8, u8),
    pub hp: i32,
    pub is_rooted: bool,
    pub quantity: u32,  // 新增。卡的量。默认 1。
}
```

### 波及

- `card_defs.ron` 中每张卡加 `quantity: 1` 或适当值
- Grep 所有 `CardDef {` 构造处，补 quantity 字段
- `card_visual.rs` 中 `fallback_def` 补 `quantity: 1`

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
