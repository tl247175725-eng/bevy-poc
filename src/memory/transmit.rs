//! 记忆传输 + 衰减 — Goody 口传 / Assmann 文化记忆 / CMU 2012
//!
//! 依据 Bartlett 1932 记忆重建、Emergent Collective Memory 2025

use crate::memory::store::*;
use crate::spatial_index::EntityId;

const NONE_ID: EntityId = EntityId(0);

/// 记忆衰减速率（每 tick）— ~5000 tick → 归零（约 12 天）
pub const MEMORY_DECAY_PER_TICK: f32 = 0.0002;

/// 强化阈值——重要性低于此值 → 不再强化
pub const MEMORY_REINFORCE_THRESHOLD: f32 = 0.3;

// ===== 衰减与强化 =====

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

// ===== 传播 =====

/// Signal→Receive 传播：A 向 B 讲述一条记忆
///
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
                source: MemorySource::Read {
                    from_object: NONE_ID,
                },
            };
            object.push(inscribed);
        }
    }
}

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_reduces_importance() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("事件A", 0);
        assert_eq!(store.memories[0].importance, 1.0);
        // 1000 ticks of decay
        for _ in 0..1000 {
            tick_memory_decay(&mut store, 0);
        }
        let expected = (1.0 - 1000.0 * MEMORY_DECAY_PER_TICK).max(0.0);
        assert!((store.memories[0].importance - expected).abs() < 0.001);
    }

    #[test]
    fn decay_removes_zero_importance() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("事件B", 0);
        // Decay until removed
        for _ in 0..6000 {
            tick_memory_decay(&mut store, 0);
        }
        assert!(store.memories.is_empty());
    }

    #[test]
    fn reinforce_restores_importance() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("狼群夜袭营地", 100);
        // Decay partially
        for _ in 0..2000 {
            tick_memory_decay(&mut store, 0);
        }
        assert!(store.memories[0].importance < 0.8);
        // Reinforce
        reinforce_memory(&mut store, "狼群", 500);
        assert_eq!(store.memories[0].importance, 1.0);
        assert_eq!(store.memories[0].timestamp, 500);
    }

    #[test]
    fn reinforce_skips_below_threshold() {
        let mut store = MemoryStore::new(10);
        store.store_witnessed("无关事件", 0);
        store.memories[0].importance = 0.1; // below threshold
        reinforce_memory(&mut store, "无关", 100);
        assert_eq!(store.memories[0].importance, 0.1); // unchanged
    }

    #[test]
    fn transmit_creates_heard_memory_with_incremented_hops() {
        let mut alice = MemoryStore::new(10);
        alice.store_witnessed("张三偷了苹果", 0);

        let mut bob = MemoryStore::new(10);
        transmit_memory(&alice, &mut bob, EntityId(1), "张三", 100);

        assert_eq!(bob.memories.len(), 1);
        let heard = &bob.memories[0];
        assert!(heard.event.contains("张三"));
        assert!(matches!(heard.source, MemorySource::Heard { from: EntityId(1), hop_count: 1 }));
        // fidelity = 1.0 (witnessed) × 0.95^1 = 0.95
        assert!((heard.fidelity - 0.95).abs() < 0.001);
    }

    #[test]
    fn transmit_chains_hops() {
        let mut a = MemoryStore::new(10);
        a.store_witnessed("事件", 0);

        // A → B (hop 1)
        let mut b = MemoryStore::new(10);
        transmit_memory(&a, &mut b, EntityId(1), "事件", 1);

        // B → C (hop 2)
        let mut c = MemoryStore::new(10);
        transmit_memory(&b, &mut c, EntityId(2), "事件", 2);

        assert_eq!(c.memories.len(), 1);
        let heard = &c.memories[0];
        assert!(matches!(heard.source, MemorySource::Heard { hop_count: 2, .. }));
        // fidelity = 1.0 × 0.95³ = 0.857375
        let expected = 1.0 * 0.95_f32.powi(3);
        assert!((heard.fidelity - expected).abs() < 0.01);
    }

    #[test]
    fn inscribe_copies_memory_to_object() {
        let mut author = MemoryStore::new(10);
        author.store_witnessed("碑文记录了一场战争", 0);

        let mut object = MemoryStore::new(10);
        inscribe_memory(&author, &mut object, "战争", 500);

        assert_eq!(object.memories.len(), 1);
        assert!(object.memories[0].event.contains("战争"));
        assert!((object.memories[0].fidelity - 1.0).abs() < 0.001);
        assert!(matches!(
            object.memories[0].source,
            MemorySource::Read { .. }
        ));
    }
}
