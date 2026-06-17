# Handoff 007 — 流动记忆系统（Store + 传播 + 衰减）

## 架构计划

**改什么：** 新建 `src/memory/mod.rs` + `src/memory/store.rs` + `src/memory/transmit.rs`（3 文件）
**依据：** Bartlett 1932 记忆重建理论、流动记忆模型、Goody 口传/Assmann 文化记忆、CMU AAAI 2012 衰减模型

### store.rs

```rust
use crate::spatial_index::EntityId;

/// 单条记忆
#[derive(Debug, Clone)]
pub struct Memory {
    pub event: String,              // "张三被李四用刀捅了"
    pub fidelity: f32,              // 1.0=亲眼所见 → 传一次×0.95 → 传十次=0.60
    pub importance: f32,            // 对拥有者的重要性 (0-1)
    pub timestamp: u64,             // 记忆记录时的 tick
    pub source: MemorySource,       // 来源
}

#[derive(Debug, Clone)]
pub enum MemorySource {
    Witnessed,                          // 亲眼所见 — fidelity=1.0
    Heard { from: EntityId, hop_count: u32 }, // 听说的 — fidelity×0.95^hop
    Read { from_object: EntityId },      // 从物体读到的
}

/// 记忆存储容器
#[derive(Debug, Clone)]
pub struct MemoryStore {
    pub memories: Vec<Memory>,
    pub capacity: usize,                // 最大记忆容量（物种/个体差异）
}

impl MemoryStore {
    pub fn new(capacity: usize) -> Self {
        Self { memories: Vec::with_capacity(capacity), capacity }
    }

    /// 存入记忆——亲眼所见
    pub fn store_witnessed(&mut self, event: impl Into<String>, tick: u64) {
        let memory = Memory {
            event: event.into(),
            fidelity: 1.0,
            importance: 1.0,
            timestamp: tick,
            source: MemorySource::Witnessed,
        };
        self.push(memory);
    }

    /// 存入记忆——听说的
    pub fn store_heard(&mut self, event: impl Into<String>, from: EntityId, source_fidelity: f32, hop_count: u32, tick: u64) {
        let fidelity = source_fidelity * 0.95_f32.powi(hop_count as i32);
        let memory = Memory {
            event: event.into(),
            fidelity,
            importance: 1.0,
            timestamp: tick,
            source: MemorySource::Heard { from, hop_count },
        };
        self.push(memory);
    }

    /// 存入记忆——从物体读到的
    pub fn store_read(&mut self, event: impl Into<String>, from_object: EntityId, source_fidelity: f32, tick: u64) {
        let memory = Memory {
            event: event.into(),
            fidelity: source_fidelity * 0.98, // 阅读比听说保真度高
            importance: 1.0,
            timestamp: tick,
            source: MemorySource::Read { from_object },
        };
        self.push(memory);
    }

    fn push(&mut self, memory: Memory) {
        if self.memories.len() >= self.capacity {
            // 踢掉最不重要的记忆
            self.memories.sort_by(|a, b| a.importance.partial_cmp(&b.importance).unwrap());
            self.memories.remove(0);
        }
        self.memories.push(memory);
    }

    /// 查询某类事件（用于复仇/追溯）
    pub fn recall_by_keyword(&self, keyword: &str) -> Vec<&Memory> {
        self.memories.iter()
            .filter(|m| m.event.contains(keyword))
            .collect()
    }
}
```

### transmit.rs

```rust
use crate::memory::store::*;
use crate::spatial_index::EntityId;

/// 记忆衰减速率（每 tick）
pub const MEMORY_DECAY_PER_TICK: f32 = 0.0002;  // ~5000 tick → 归零（约12天）

/// 强化阈值——重要性低于此值 → 不再强化
pub const MEMORY_REINFORCE_THRESHOLD: f32 = 0.3;

/// tick 衰减：每条记忆重要性随时间下降
pub fn tick_memory_decay(store: &mut MemoryStore, _current_tick: u64) {
    for memory in &mut store.memories {
        memory.importance = (memory.importance - MEMORY_DECAY_PER_TICK).max(0.0);
    }
    // 清理归零记忆
    store.memories.retain(|m| m.importance > 0.0);
}

/// 强化记忆——回忆/讨论时，重要性重置
pub fn reinforce_memory(store: &mut MemoryStore, keyword: &str, current_tick: u64) {
    for memory in &mut store.memories {
        if memory.event.contains(keyword) && memory.importance > MEMORY_REINFORCE_THRESHOLD {
            memory.importance = 1.0;
            memory.timestamp = current_tick;
        }
    }
}

/// Signal→Receive 传播：A 向 B 讲述一条记忆
/// 返回 B 收到的新记忆（fidelity 经 A 的版本再衰减）
pub fn transmit_memory(
    teller: &MemoryStore,
    listener: &mut MemoryStore,
    teller_id: EntityId,
    keyword: &str,
    current_tick: u64,
) {
    for memory in &teller.memories {
        if memory.event.contains(keyword) {
            let hop_count = match &memory.source {
                MemorySource::Heard { hop_count: h, .. } => *h + 1,
                _ => 1, // 亲眼所见 → 第一次传播
            };
            listener.store_heard(
                memory.event.clone(),
                teller_id,
                memory.fidelity,
                hop_count,
                current_tick,
            );
        }
    }
}

/// 物体记忆——向物体上 Store（刻写/记录）
pub fn inscribe_memory(
    author: &MemoryStore,
    object: &mut MemoryStore,
    keyword: &str,
    current_tick: u64,
) {
    for memory in &author.memories {
        if memory.event.contains(keyword) {
            let inscribed = Memory {
                event: memory.event.clone(),
                fidelity: memory.fidelity, // 刻写保真度 = 作者的记忆版本
                importance: 1.0,
                timestamp: current_tick,
                source: MemorySource::Read { from_object: EntityId(0) }, // 占位
            };
            object.push(inscribed);
        }
    }
}
```

### mod.rs

```rust
pub mod store;
pub mod transmit;

pub use store::*;
pub use transmit::*;
```

### lib.rs

```rust
pub mod memory;
```

## 架构反馈

**文献一致性：**
- Bartlett 1932: fidelity × 0.95^hop 对应每次传输丢失细节 ✅
- CMU 2012: MEMORY_DECAY_PER_TICK 对应内部衰减 ✅
- 2025 Emergent Collective Memory: 临界密度——衰减快→需更多人口维持记忆 ✅
- 物体刻写: 阅读 fidelity 衰减 0.98（比口头 0.95 慢）—Assmann 文化记忆理论 ✅

**与已有系统的连接：**
- Store 元动作（信息层）→ 调用 store_witnessed
- Signal/Receive → transmit_memory
- 死亡 → MemoryStore 随 Entity 销毁
- 物体刻写 → inscribe_memory
- 复仇系统 → recall_by_keyword("张三")

**性能：**
- 每条记忆 ~100 字节，200 实体×50 条 = 1MB
- 衰减 0.01ms/tick
- 传输偶发事件，不计

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：store_witnessed → fidelity=1.0，importance=1.0
- 测试：store_heard 3 跳 → fidelity=0.95³≈0.86
- 测试：decay 后 → recall 空结果
- 测试：reinforce 后 → importance 恢复
- 测试：超容量 → 踢出最不重要记忆
