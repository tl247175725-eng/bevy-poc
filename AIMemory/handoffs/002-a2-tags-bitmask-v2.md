# Handoff 002-a2 v2 — 创建标签位掩码存储 + TagRegistry

> ⚠️ 替换 v1。v1 在手工计算 70+ 个 descendants 位掩码——这是错误的方法。
> 如果之前已部分创建了 src/tags.rs，先删除，从零开始。

## 架构计划

**改什么：** 新建 `src/tags.rs`（1 个文件）
**为什么：** 标签查询从字符串分配换成位掩码 O(1)。tags.ron 已有完整树结构——所有 descendants 从树自动计算，禁止手工算。

**核心原则：tags.ron 是唯一真相来源。代码从它读树，自动生成一切。**

### 文件内容（三个组件）

**组件 1: TagBits — 位掩码存储**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagBits {
    bits: [u64; 8],  // 512 bits total
}

impl TagBits {
    pub fn new() -> Self { Self { bits: [0; 8] } }
    pub fn set(&mut self, bit: u16) { self.bits[(bit/64) as usize] |= 1 << (bit % 64); }
    pub fn unset(&mut self, bit: u16) { self.bits[(bit/64) as usize] &= !(1 << (bit % 64)); }
    pub fn has(&self, bit: u16) -> bool { (self.bits[(bit/64) as usize] >> (bit % 64)) & 1 == 1 }
    pub fn union(&self, other: &TagBits) -> TagBits {
        let mut result = TagBits::new();
        for i in 0..8 { result.bits[i] = self.bits[i] | other.bits[i]; }
        result
    }
}
```

**组件 2: TagRegistry — 从 tags.ron 自动生成**
```rust
use std::collections::HashMap;

pub struct TagRegistry {
    /// 标签名 → bit 位置
    name_to_bit: HashMap<String, u16>,
    /// bit 位置 → 标签名
    bit_to_name: HashMap<u16, String>,
    /// 标签 bit → 所有后代 bit 的位掩码 (用于 has_descendant_of)
    descendants: HashMap<u16, TagBits>,
    /// 下一个可分配的 bit
    next_bit: u16,
}

impl TagRegistry {
    pub fn from_tags_ron() -> Self {
        // 读取 assets/tags.ron
        // 遍历 positional/systemic 树
        // 对每个节点：
        //   1. 分配 bit = next_bit++
        //   2. 递归收集所有子节点 → 计算 descendants 位掩码（自动 OR）
        //   3. 存入 name_to_bit / bit_to_name / descendants
        // 返回注册表
        todo!("实现 load 逻辑")
    }
    
    /// 子树检查：child_bit 是 parent_bit 的后代吗？
    pub fn is_descendant_of(&self, child_bit: u16, parent_bit: u16) -> bool {
        self.descendants
            .get(&parent_bit)
            .map(|desc_bits| desc_bits.has(child_bit))
            .unwrap_or(false)
    }
}
```

**组件 3: 便捷查询 trait**
```rust
pub trait TagQuery {
    fn has_tag(&self, registry: &TagRegistry, name: &str) -> bool;
    fn has_descendant_of(&self, registry: &TagRegistry, parent_bit: u16) -> bool;
}

impl TagQuery for TagBits {
    fn has_tag(&self, registry: &TagRegistry, name: &str) -> bool {
        registry.name_to_bit.get(name)
            .map(|&bit| self.has(bit))
            .unwrap_or(false)
    }
    
    fn has_descendant_of(&self, registry: &TagRegistry, parent_bit: u16) -> bool {
        registry.descendants.get(&parent_bit)
            .map(|desc_bits| {
                // 实体的 bits AND parent 所有后代 bits ≠ 0 → 有后代标签
                let mut result = false;
                for i in 0..8 {
                    if self.bits[i] & desc_bits.bits[i] != 0 {
                        result = true;
                        break;
                    }
                }
                result
            })
            .unwrap_or(false)
    }
}
```

### 不需要做的事

- ❌ 不手工定义 TagMask 常量
- ❌ 不手工计算 descendants 位掩码
- ❌ 不在 tags.rs 里硬编码任何标签位值
- ❌ 不定义 tags 模块（tags.ron 是唯一来源，代码从它读）

### 为什么这个方案正确

1. tags.ron 已有完整树结构（`body → head → skull`）
2. 代码遍历这棵树 → 自动分配 bit → 自动计算 descendants = 子节点所有 bit 的 OR
3. 加新标签只需改 tags.ron → 代码自动重新分配 bit
4. 零手工位运算 → 零出错机会

## 架构反馈

**一致性与简化：**
- tags.ron 是唯一真相来源 ✅
- descendants 由代码从树自动计算 ✅
- 无硬编码标签常量 ✅
- 运行时动态标签通过 next_bit 扩展即可 ✅

**当前限制：**
- `from_tags_ron()` 的 RON 解析用 `todo!()`——因为解析 RON 结构需要额外的类型定义，留给后续 handoff
- 当前 handoff 目标：TagBits + TagRegistry 数据结构就位，`from_tags_ron` 骨架写对，测试验证数据结构本身

## 智能验收

- `cargo check` 零错误
- 删除之前任何部分创建的 src/tags.rs，从零新建
- 新增测试：
  - `tagbits_set_and_has` — set(5) → has(5)=true, has(6)=false
  - `tagbits_multiple_words` — set(70) 正确（跨 u64 边界）
  - `tagbits_union` — 两个 TagBits 的 union 包含各自的标签
  - `registry_allocate_bits` — 创建 TagRegistry，手工插入几个标签，验证 bit 分配和 descendants 计算正确
