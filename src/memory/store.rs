//! 流动记忆存储 — Bartlett 1932 记忆重建理论
//!
//! 依据 Goody 口传 / Assmann 文化记忆 / CMU AAAI 2012 衰减模型

use crate::spatial_index::EntityId;

// ===== 记忆条目 =====

/// 单条记忆
#[derive(Debug, Clone)]
pub struct Memory {
    pub event: String,       // "张三被李四用刀捅了"
    pub fidelity: f32,       // 1.0=亲眼所见 → 传一次×0.95 → 传十次=0.60
    pub importance: f32,     // 对拥有者的重要性 (0-1)
    pub timestamp: u64,      // 记忆记录时的 tick
    pub source: MemorySource,
}

#[derive(Debug, Clone)]
pub enum MemorySource {
    /// 亲眼所见 — fidelity=1.0
    Witnessed,
    /// 听说的 — fidelity×0.95^hop
    Heard { from: EntityId, hop_count: u32 },
    /// 从物体读到的
    Read { from_object: EntityId },
}

// ===== 记忆存储容器 =====

#[derive(Debug, Clone)]
pub struct MemoryStore {
    pub memories: Vec<Memory>,
    /// 最大记忆容量（物种/个体差异）
    pub capacity: usize,
}

impl MemoryStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            memories: Vec::with_capacity(capacity),
            capacity,
        }
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
    pub fn store_heard(
        &mut self,
        event: impl Into<String>,
        from: EntityId,
        source_fidelity: f32,
        hop_count: u32,
        tick: u64,
    ) {
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
    pub fn store_read(
        &mut self,
        event: impl Into<String>,
        from_object: EntityId,
        source_fidelity: f32,
        tick: u64,
    ) {
        let memory = Memory {
            event: event.into(),
            fidelity: source_fidelity * 0.98, // 阅读比听说保真度高
            importance: 1.0,
            timestamp: tick,
            source: MemorySource::Read { from_object },
        };
        self.push(memory);
    }

    pub(crate) fn push(&mut self, memory: Memory) {
        if self.memories.len() >= self.capacity {
            // 踢掉最不重要的记忆
            self.memories
                .sort_by(|a, b| a.importance.partial_cmp(&b.importance).unwrap());
            self.memories.remove(0);
        }
        self.memories.push(memory);
    }

    /// 查询某类事件（用于复仇/追溯）
    pub fn recall_by_keyword(&self, keyword: &str) -> Vec<&Memory> {
        self.memories
            .iter()
            .filter(|m| m.event.contains(keyword))
            .collect()
    }
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_witnessed_has_full_fidelity() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("狼袭击了羊群", 100);
        let mem = &store.memories[0];
        assert_eq!(mem.fidelity, 1.0);
        assert_eq!(mem.importance, 1.0);
        assert!(matches!(mem.source, MemorySource::Witnessed));
    }

    #[test]
    fn store_heard_fidelity_decays_with_hops() {
        let mut store = MemoryStore::new(10);
        // 3 跳：fidelity = 1.0 × 0.95³ ≈ 0.857
        store.store_heard("狼袭击了羊群", EntityId(1), 1.0, 3, 100);
        let mem = &store.memories[0];
        let expected = 1.0 * 0.95_f32.powi(3);
        assert!((mem.fidelity - expected).abs() < 0.001);
        assert!(matches!(
            mem.source,
            MemorySource::Heard {
                from: EntityId(1),
                hop_count: 3
            }
        ));
    }

    #[test]
    fn store_read_fidelity_slower_decay() {
        let mut store = MemoryStore::new(10);
        store.store_read("碑文记载洪水", EntityId(42), 0.9, 500);
        let mem = &store.memories[0];
        let expected = 0.9 * 0.98;
        assert!((mem.fidelity - expected).abs() < 0.001);
        assert!(matches!(
            mem.source,
            MemorySource::Read {
                from_object: EntityId(42)
            }
        ));
    }

    #[test]
    fn capacity_evicts_least_important() {
        let mut store = MemoryStore::new(3);
        store.store_witnessed("A", 1);
        store.store_witnessed("B", 2);
        store.store_witnessed("C", 3);
        // Manually lower importance of the first memory
        store.memories[0].importance = 0.1;
        // Push a 4th — should evict the 0.1 importance one
        store.store_witnessed("D", 4);
        // "A" should be gone
        let events: Vec<&str> = store.memories.iter().map(|m| m.event.as_str()).collect();
        assert!(!events.contains(&"A"));
        assert!(events.contains(&"D"));
    }

    #[test]
    fn recall_by_keyword_finds_matches() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("张三被李四用刀捅了", 10);
        store.store_witnessed("王五在河边钓鱼", 20);
        let results = store.recall_by_keyword("张三");
        assert_eq!(results.len(), 1);
        assert!(results[0].event.contains("张三"));
    }

    #[test]
    fn recall_returns_empty_when_no_match() {
        let store = MemoryStore::new(10);
        let results = store.recall_by_keyword("不存在");
        assert!(results.is_empty());
    }
}
