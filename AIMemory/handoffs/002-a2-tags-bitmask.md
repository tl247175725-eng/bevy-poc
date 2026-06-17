# Handoff 002-a2 — 创建标签位掩码存储 + 查询 API

## 架构计划

**改什么：** 新建 `src/tags.rs`（1 个文件）
**为什么：** 标签查询从字符串分配+遍历换成位掩码 O(1) AND，消灭性能瓶颈
**依据：** `design-philosophy-v5.md` §1、FACT.md 三柱职能边界、Opus Q4/Q6 性能分析

**文件内容：**

### 1. TagBits — 位掩码存储结构

```rust
/// 512-bit 标签位掩码，内联数组，零堆分配
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagBits {
    bits: [u64; 8],  // 512 bits total
}

impl TagBits {
    pub fn new() -> Self { Self { bits: [0; 8] } }
    
    /// 设置一个 bit
    pub fn set(&mut self, bit: u16) { ... }  // bit ∈ [0, 511]
    
    /// 清除一个 bit
    pub fn unset(&mut self, bit: u16) { ... }
    
    /// 精确匹配：这个标签存在吗？
    pub fn has(&self, bit: u16) -> bool { ... }
    
    /// 子树匹配：entity 有任何属于 parent 子树的标签吗？
    /// parent 的位掩码包含所有子标签的位 → 一次 AND
    pub fn has_descendant_of(&self, parent: &TagMask) -> bool { ... }
    
    /// 取并集
    pub fn union(&self, other: &TagBits) -> TagBits { ... }
}
```

### 2. TagMask — 预计算的父子树掩码

```rust
/// 标签的"全家桶"——该标签自己的位 + 所有子标签的位的 OR
/// 用于 O(1) 子树检查
#[derive(Debug, Clone, Copy)]
pub struct TagMask {
    pub bit: u16,           // 这个标签自己的位
    pub descendants: u64,   // 所有子标签的位的预计算 OR（同组内）
    // 跨组（如位置组+系统组）用 TagBits 的 full_mask
}
```

### 3. Tag 常量枚举（临时手工映射 tags.ron）

```rust
/// 编译期标签常量。当前手工定义，后续 build.rs 从 tags.ron 自动生成。
pub mod tags {
    use super::TagMask;
    
    // === 位置维度 (bits 0-127) ===
    pub const BODY: TagMask = TagMask { bit: 0, descendants: /* 所有 body 子标签 */ };
    pub const HEAD: TagMask = TagMask { bit: 1, descendants: /* 所有 head 子标签 */ };
    pub const SKULL: TagMask = TagMask { bit: 2, descendants: 0 };
    pub const BRAIN: TagMask = TagMask { bit: 3, descendants: 0 };
    pub const EYE: TagMask = TagMask { bit: 4, descendants: (1<<5)|(1<<6) }; // left + right
    pub const EYE_LEFT: TagMask = TagMask { bit: 5, descendants: 0 };
    pub const EYE_RIGHT: TagMask = TagMask { bit: 6, descendants: 0 };
    pub const EAR: TagMask = TagMask { bit: 7, descendants: (1<<8)|(1<<9) };
    pub const EAR_LEFT: TagMask = TagMask { bit: 8, descendants: 0 };
    pub const EAR_RIGHT: TagMask = TagMask { bit: 9, descendants: 0 };
    pub const JAW: TagMask = TagMask { bit: 10, descendants: 0 };
    pub const TORSO: TagMask = TagMask { bit: 11, descendants: /* torso 子标签 */ };
    pub const SPINE: TagMask = TagMask { bit: 12, descendants: 0 };
    pub const RIBCAGE: TagMask = TagMask { bit: 13, descendants: 0 };
    pub const ORGAN_HEART: TagMask = TagMask { bit: 14, descendants: 0 };
    pub const ORGAN_LUNG: TagMask = TagMask { bit: 15, descendants: 0 };
    pub const ORGAN_LIVER: TagMask = TagMask { bit: 16, descendants: 0 };
    pub const ORGAN_KIDNEY: TagMask = TagMask { bit: 17, descendants: 0 };
    pub const ORGAN_STOMACH: TagMask = TagMask { bit: 18, descendants: 0 };
    pub const ORGAN_INTESTINE: TagMask = TagMask { bit: 19, descendants: 0 };
    pub const VESSEL_AORTA: TagMask = TagMask { bit: 20, descendants: 0 };
    pub const LIMB: TagMask = TagMask { bit: 21, descendants: /* arm + leg 子标签 */ };
    pub const ARM: TagMask = TagMask { bit: 22, descendants: /* arm 子标签 */ };
    pub const UPPER_ARM: TagMask = TagMask { bit: 23, descendants: 0 };
    pub const FOREARM: TagMask = TagMask { bit: 24, descendants: 0 };
    pub const HAND: TagMask = TagMask { bit: 25, descendants: 0 };
    pub const FINGER: TagMask = TagMask { bit: 26, descendants: 0 };
    pub const LEG: TagMask = TagMask { bit: 27, descendants: /* leg 子标签 */ };
    pub const THIGH: TagMask = TagMask { bit: 28, descendants: 0 };
    pub const SHIN: TagMask = TagMask { bit: 29, descendants: 0 };
    pub const FOOT: TagMask = TagMask { bit: 30, descendants: 0 };
    pub const TOE: TagMask = TagMask { bit: 31, descendants: 0 };
    pub const VESSEL_FEMORAL: TagMask = TagMask { bit: 32, descendants: 0 };
    
    // === 系统维度 (bits 128-191) ===
    pub const SYSTEM_SKELETAL: TagMask = TagMask { bit: 128, descendants: /* bone */ };
    pub const SYSTEM_MUSCULAR: TagMask = TagMask { bit: 129, descendants: /* muscle */ };
    pub const SYSTEM_CIRCULATORY: TagMask = TagMask { bit: 130, descendants: /* vessel */ };
    pub const SYSTEM_NERVOUS: TagMask = TagMask { bit: 131, descendants: /* nerve */ };
    pub const SYSTEM_RESPIRATORY: TagMask = TagMask { bit: 132, descendants: 0 };
    pub const SYSTEM_DIGESTIVE: TagMask = TagMask { bit: 133, descendants: 0 };
    
    // === 生命关键 (bits 192-223) ===
    pub const VITAL_BRAIN: TagMask = TagMask { bit: 192, descendants: 0 };
    pub const VITAL_HEART: TagMask = TagMask { bit: 193, descendants: 0 };
    pub const VITAL_LUNG: TagMask = TagMask { bit: 194, descendants: 0 };
    pub const VITAL_LIVER: TagMask = TagMask { bit: 195, descendants: 0 };
    pub const VITAL_KIDNEY: TagMask = TagMask { bit: 196, descendants: 0 };
    
    // === 能力维度 (bits 224-287) ===
    pub const CAP_MOVE: TagMask = TagMask { bit: 224, descendants: 0 };
    pub const CAP_FLY: TagMask = TagMask { bit: 225, descendants: 0 };
    pub const CAP_SWIM: TagMask = TagMask { bit: 226, descendants: 0 };
    pub const CAP_CLIMB: TagMask = TagMask { bit: 227, descendants: 0 };
    pub const CAP_GRASP: TagMask = TagMask { bit: 228, descendants: 0 };
    pub const CAP_BITE: TagMask = TagMask { bit: 229, descendants: 0 };
    pub const CAP_SPEAK: TagMask = TagMask { bit: 230, descendants: 0 };
    pub const CAP_CRAFT: TagMask = TagMask { bit: 231, descendants: 0 };
    
    // === 材质维度 (bits 288-351) ===
    pub const MAT_FLESH: TagMask = TagMask { bit: 288, descendants: 0 };
    pub const MAT_BONE: TagMask = TagMask { bit: 289, descendants: 0 };
    pub const MAT_WOOD: TagMask = TagMask { bit: 290, descendants: 0 };
    pub const MAT_STONE: TagMask = TagMask { bit: 291, descendants: 0 };
    pub const MAT_IRON: TagMask = TagMask { bit: 292, descendants: 0 };
    pub const MAT_COPPER: TagMask = TagMask { bit: 293, descendants: 0 };
    pub const MAT_BRONZE: TagMask = TagMask { bit: 294, descendants: 0 };
    pub const MAT_STEEL: TagMask = TagMask { bit: 295, descendants: 0 };
    
    // === 感官维度 (bits 352-383) ===
    pub const SENSE_VISION: TagMask = TagMask { bit: 352, descendants: 0 };
    pub const SENSE_HEARING: TagMask = TagMask { bit: 353, descendants: 0 };
    pub const SENSE_SMELL: TagMask = TagMask { bit: 354, descendants: 0 };
    pub const SENSE_TOUCH: TagMask = TagMask { bit: 355, descendants: 0 };
    
    // === 行为维度 (bits 384-447) ===
    pub const BEH_PREDATOR: TagMask = TagMask { bit: 384, descendants: 0 };
    pub const BEH_HERBIVORE: TagMask = TagMask { bit: 385, descendants: 0 };
    pub const BEH_OMNIVORE: TagMask = TagMask { bit: 386, descendants: 0 };
    pub const BEH_NOCTURNAL: TagMask = TagMask { bit: 387, descendants: 0 };
    
    // === 社会维度 (bits 448-479) ===
    pub const SOC_SOLITARY: TagMask = TagMask { bit: 448, descendants: 0 };
    pub const SOC_PACK: TagMask = TagMask { bit: 449, descendants: 0 };
    pub const SOC_HERD: TagMask = TagMask { bit: 450, descendants: 0 };
    
    // === 损伤状态 (bits 480-495, dynamic 子范围) ===
    pub const INJ_HEALTHY: TagMask = TagMask { bit: 480, descendants: 0 };
    pub const INJ_BRUISED: TagMask = TagMask { bit: 481, descendants: 0 };
    pub const INJ_DAMAGED: TagMask = TagMask { bit: 482, descendants: 0 };
    pub const INJ_FRACTURED: TagMask = TagMask { bit: 483, descendants: 0 };
    pub const INJ_SEVERED: TagMask = TagMask { bit: 484, descendants: 0 };
    pub const INJ_MISSING: TagMask = TagMask { bit: 485, descendants: 0 };
}
```

### 4. TagParams — 参数侧表

```rust
/// 存储参数化标签的数值。位掩码管"有没有"，侧表管"有多强"。
#[derive(Debug, Clone, Default)]
pub struct TagParams {
    pub vision_range: Option<u32>,
    pub hearing_range: Option<u32>,
    pub smell_range: Option<u32>,
    pub metabolism_rate: Option<f32>,
    pub max_age: Option<f32>,
    // 后续扩展...
}

impl TagParams {
    pub fn new() -> Self { Self::default() }
}
```

### 5. 从 tags.ron 加载的引导函数

```rust
/// 从 tags.ron 加载标签定义，初始化位掩码分配表
pub fn load_tag_registry() -> TagRegistry {
    // 读取 assets/tags.ron → 解析 RON → 填充位分配
    // 当前阶段手工映射（Tag 常量已硬编码），此函数为后续 build.rs 自动化预留
    todo!("build.rs 自动化后实现")
}
```

## 架构反馈

**与设计哲学一致性：**
- 标签数据与行为分离 ✅ — TagBits 是纯数据，公理读取它
- 位掩码 O(1) 查询 ✅ — 消灭字符串分配瓶颈（Opus Q6）
- 位包含实现层级 ✅ — has_descendant_of 一次 AND
- 参数侧表分离 ✅ — 位掩码管质性，TagParams 管数值
- 编译期常量 ✅ — TagMask 的 bit/descendants 编译时确定

**当前限制（后续 handoff）：**
- Tag 常量手工定义，不是从 tags.ron 自动生成——build.rs 自动化后续做
- descendants 字段需要预计算所有子标签位——手工更易出错，自动化后彻底安全
- TagParams 当前为 struct 固定字段，未来可能需要 HashMap<String, f32> 更灵活

## 智能验收

- `cargo check` 零错误
- 新增测试：
  - `tagbits_set_and_has` — set(5) → has(5)=true, has(6)=false
  - `tagbits_descendant_check` — BODY descendants 包含 HEAD → has_descendant_of(BODY) 对 head 实体返回 true
  - `tagbits_union` — A|B 同时包含 A 和 B 的标签
  - `tagmask_bit_ranges` — 所有 bit 值在 0-511 范围内
