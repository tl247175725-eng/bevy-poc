//! 初始生成——七环同心世界。

use crate::card_def::CardDef;
use crate::meta_values::{
    CURIOSITY_BASELINE, CURIOSITY_DECAY, NUTRITION_BASELINE, NUTRITION_DECAY_HIGH,
    NUTRITION_DECAY_LOW, NUTRITION_DECAY_MEDIUM, SAFETY_BASELINE, SOCIAL_BASELINE, SOCIAL_DECAY,
};
use crate::need_match::data::{
    CompareOp, DecompositionStep, EffectDescriptor, KnowledgeEntry, KnowledgeGraph, KnowledgeId,
    KnowledgeSource, NeedKind, NeedState, PropertyRequirement,
};
use crate::tags::TagQuery;
use crate::world_rules::{card_has_tag, is_animal, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

/// 环索引常量。
const RING_ABYSS: usize = 0;
const RING_SHALLOW: usize = 1;
const RING_WETLAND: usize = 2;
const RING_GRASSLAND: usize = 3;
const RING_FOREST: usize = 4;
const RING_FOOTHILLS: usize = 5;
const RING_CLIFF: usize = 6;

/// Habitat 标签 → 环索引（允许多环，如 aquatic 跨深潭+浅水）。
const HABITAT_RING_MAP: &[(&str, &[usize])] = &[
    ("habitat:aquatic", &[RING_ABYSS, RING_SHALLOW]),
    ("habitat:wetland", &[RING_WETLAND]),
    ("habitat:grassland", &[RING_GRASSLAND]),
    ("habitat:forest", &[RING_FOREST]),
    ("habitat:mountain", &[RING_FOOTHILLS]),
];

/// 返回 `dist` 在 `RING_ABYSS..=RING_CLIFF` 的环索引
fn ring_for_cell(x: u8, y: u8) -> usize {
    let center = (16.0f32, 16.0f32);
    let dx = x as f32 - center.0;
    let dy = y as f32 - center.1;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 1.5 {
        RING_ABYSS
    } else if dist <= 3.5 {
        RING_SHALLOW
    } else if dist <= 7.0 {
        RING_WETLAND
    } else if dist <= 12.0 {
        RING_GRASSLAND
    } else if dist <= 19.0 {
        RING_FOREST
    } else if dist <= 25.0 {
        RING_FOOTHILLS
    } else {
        RING_CLIFF
    }
}

// ===== 动物 Need + 知识图初始化（标签驱动） =====

/// metab: 标签 → 营养 decay rate（集中管理裸数字）。
fn metab_decay_rate(def: &CardDef) -> f32 {
    match () {
        _ if card_has_tag(def, "metab:high") => NUTRITION_DECAY_HIGH,
        _ if card_has_tag(def, "metab:low")  => NUTRITION_DECAY_LOW,
        _ => NUTRITION_DECAY_MEDIUM,
    }
}

/// 根据 CardDef 标签生成初始 Needs。
fn init_animal_needs(def: &CardDef) -> Vec<NeedState> {
    let mut needs = Vec::new();

    // 所有动物都需要营养：decay_rate 从 metab: 标签推导
    let nutrition_decay = metab_decay_rate(def);
    needs.push(NeedState {
        kind: NeedKind::Nutrition,
        current: 0.0,
        baseline: NUTRITION_BASELINE,
        decay_rate: nutrition_decay,
        blocked: false,
        urgency: 0.0,
    });

    // 所有动物都需要安全（不自然衰减）
    needs.push(NeedState {
        kind: NeedKind::Safety,
        current: 0.0,
        baseline: SAFETY_BASELINE,
        decay_rate: 0.0,
        blocked: false,
        urgency: 0.0,
    });

    // social:pack / social:herd → 社交需求
    if card_has_tag(def, "social:pack") || card_has_tag(def, "social:herd") {
        needs.push(NeedState {
            kind: NeedKind::Social,
            current: 0.0,
            baseline: SOCIAL_BASELINE,
            decay_rate: SOCIAL_DECAY,
            blocked: false,
            urgency: 0.0,
        });
    }

    // cognition:basic_learning 以上 → 好奇心
    if card_has_tag(def, "cognition:basic_learning") {
        needs.push(NeedState {
            kind: NeedKind::Curiosity,
            current: 0.0,
            baseline: CURIOSITY_BASELINE,
            decay_rate: CURIOSITY_DECAY,
            blocked: false,
            urgency: 0.0,
        });
    }

    needs
}

/// 公开包装——供集成测试使用
pub fn init_animal_needs_public(def: &CardDef) -> Vec<NeedState> {
    init_animal_needs(def)
}
/// 公开包装——供集成测试使用
pub fn init_animal_knowledge_public(def: &CardDef) -> KnowledgeGraph {
    init_animal_knowledge(def)
}

/// 从 diet 标签派生可食标签集（一条通用规则，不分物种）
fn diet_to_edible_tags(tags: &impl TagQuery, registry: &crate::tags::TagRegistry) -> Vec<String> {
    let mut edible = Vec::new();
    // carnivore → 可吃 animal
    if tags.has_tag(registry, "diet:carnivore") { edible.push("animal".into()); }
    // herbivore → 可吃 plant
    if tags.has_tag(registry, "diet:herbivore") { edible.push("plant".into()); }
    // piscivore → 可吃 body_plan:fish
    if tags.has_tag(registry, "diet:piscivore") { edible.push("body_plan:fish".into()); }
    // insectivore → 可吃 body_plan:insectoid
    if tags.has_tag(registry, "diet:insectivore") { edible.push("body_plan:insectoid".into()); }
    // frugivore → 可吃 plant（果实尚未作为独立卡牌类型存在）
    if tags.has_tag(registry, "diet:frugivore") { edible.push("plant".into()); }
    // omnivore → 可吃 animal + plant
    if tags.has_tag(registry, "diet:omnivore") { edible.push("animal".into()); edible.push("plant".into()); }
    // detritivore/scavenger → 尸体通过 is_corpse 判断，不走 has_tag
    // 暂时保留 "animal" 作为标记——后续 handoff 改 MaterialProperties.satisfies() 支持 is_corpse
    if tags.has_tag(registry, "diet:scavenger")
        || tags.has_tag(registry, "diet:detritivore") { edible.push("animal".into()); } // 先吃尸体=死动物
    edible
}

/// 从可食标签集构建**一条**通用觅食知识条目（不分分支）
fn build_foraging_knowledge(edible_tags: &[String]) -> KnowledgeEntry {
    KnowledgeEntry {
        id: KnowledgeId(0), // 由 kg.add() 覆盖
        name: "觅食".into(),
        functional_prerequisites: edible_tags.iter().map(|t| PropertyRequirement {
            property: "has_tag".into(),
            operator: CompareOp::Present,
            threshold: 0.0,
            quantity_needed: 1.0,
            tag_value: Some(t.clone()),
        }).collect(),
        decomposition: vec![
            DecompositionStep::Acquire { requirements: vec![] },
            DecompositionStep::Act { action: "Consume".into(), target: Some("acquired_0".into()) },
        ],
        effects: vec![EffectDescriptor { satisfies: NeedKind::Nutrition, magnitude: 0.5 }],
        source: KnowledgeSource::CommonSense,
    }
}

/// 根据 CardDef 标签生成初始常识知识图（派生规则，无 per-diet 分支）。
fn init_animal_knowledge(def: &CardDef) -> KnowledgeGraph {
    let registry = match crate::world_rules::TAG_REGISTRY.get() {
        Some(r) => r,
        None => return KnowledgeGraph::new(),
    };
    let edible = diet_to_edible_tags(&def.tag_bits, registry);
    if edible.is_empty() { return KnowledgeGraph::new(); }
    let mut kg = KnowledgeGraph::new();
    kg.add(build_foraging_knowledge(&edible));
    kg
}

/// 按栖息环面积比例散落动物。
/// 每物种每格最多 1 只；水生动物的 z 坐标设为水下。
pub fn spawn_animals(world: &mut WorldState, card_defs: &[CardDef]) {
    let animal_cards: Vec<&CardDef> = card_defs.iter().filter(|d| is_animal(d)).collect();
    if animal_cards.is_empty() {
        return;
    }

    // 预计算各环格点列表
    let mut ring_cells: [Vec<(u8, u8)>; 7] =
        [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            ring_cells[ring_for_cell(x, y)].push((x, y));
        }
    }
    let ring_areas: [usize; 7] = [
        ring_cells[0].len(),
        ring_cells[1].len(),
        ring_cells[2].len(),
        ring_cells[3].len(),
        ring_cells[4].len(),
        ring_cells[5].len(),
        ring_cells[6].len(),
    ];

    for animal_def in &animal_cards {
        // 提取 habitat 标签 → 匹配环（去重）
        let mut matching_rings: Vec<usize> = Vec::new();
        for &(habitat, rings) in HABITAT_RING_MAP {
            if card_has_tag(animal_def, habitat) {
                for &r in rings {
                    if !matching_rings.contains(&r) {
                        matching_rings.push(r);
                    }
                }
            }
        }
        if matching_rings.is_empty() {
            continue;
        }

        let total = animal_def.quantity as usize;
        if total == 0 {
            continue;
        }

        // 按面积比例分配
        let total_area: usize = matching_rings.iter().map(|&r| ring_areas[r]).sum();
        if total_area == 0 {
            continue;
        }
        let mut remaining = total;
        let is_aquatic = card_has_tag(animal_def, "habitat:aquatic");

        for (idx, &r) in matching_rings.iter().enumerate() {
            let count = if idx == matching_rings.len() - 1 {
                remaining
            } else {
                let c = total * ring_areas[r] / total_area;
                remaining = remaining.saturating_sub(c);
                c
            };
            let count = count.min(ring_areas[r]);
            if count == 0 {
                continue;
            }

            // 确定性伪随机排列环内格点（按 type_name + 坐标哈希排序）
            let mut sorted: Vec<(u8, u8)> = ring_cells[r].clone();
            let type_seed: u64 = animal_def
                .type_name
                .as_bytes()
                .iter()
                .fold(0u64, |acc, &b| acc.wrapping_mul(127).wrapping_add(b as u64));
            sorted.sort_by_key(|&(x, y)| {
                type_seed
                    .wrapping_mul(1_000_003)
                    .wrapping_add(x as u64 * 1_000_033)
                    .wrapping_add(y as u64 * 1_000_037)
            });

            for &(x, y) in sorted.iter().take(count) {
                let id = world.spawn(&animal_def.type_name, x, y);
                if let Some(entity) = world.entities.get_mut(&id) {
                    if is_aquatic {
                        entity.z = -1;
                    }
                    // 初始化需求 + 知识图
                    entity.needs = init_animal_needs(animal_def);
                    entity.knowledge = init_animal_knowledge(animal_def);
                }
            }
        }
    }
}

/// 按七环同心圆结构填充 32×32 棋盘。
pub fn spawn_concentric_world(world: &mut WorldState, card_defs: &[CardDef]) {
    let center = (16.0, 16.0); // 地图正中心

    // ── Phase 1: 基底地形 ──
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let dx = x as f32 - center.0;
            let dy = y as f32 - center.1;
            let dist = (dx * dx + dy * dy).sqrt(); // 欧几里得距离

            let card_type = match dist {
                d if d <= 1.5 => "abyss_pool",
                d if d <= 3.5 => "shallow_water",
                d if d <= 7.0 => "wetland",
                d if d <= 12.0 => "grassland",
                d if d <= 19.0 => "broadleaf_forest",
                d if d <= 25.0 => "foothills",
                _ => "cliff",
            };

            // 只在 card_defs 中存在时生成
            if let Some(_def) = card_defs.iter().find(|d| d.type_name == card_type) {
                world.spawn(card_type, x, y);
            }
        }
    }

    // ── Phase 2: 散点植物 ──
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let dx = x as f32 - center.0;
            let dy = y as f32 - center.1;
            let dist = (dx * dx + dy * dy).sqrt();

            // seed = x*32+y 确保可复现性
            let seed = (x as u64 * GRID_WIDTH as u64 + y as u64).wrapping_mul(1103515245).wrapping_add(12345);
            let r = (seed >> 16) as u32 % 100; // 0..99

            match () {
                // 浅水区: lotus 1/6 ≈ 0.167 → r < 17; waterweed 1/3 → r < 50
                _ if dist <= 3.5 => {
                    if r < 17 {
                        world.spawn("lotus", x, y);
                    }
                    if r < 50 {
                        world.spawn("waterweed", x, y);
                    }
                }
                // 湿地: reed 1/2 → r < 50; cattail 1/3 → r < 33
                _ if dist <= 7.0 => {
                    if r < 50 {
                        world.spawn("reed", x, y);
                    }
                    if r < 33 {
                        world.spawn("cattail", x, y);
                    }
                }
                // 草原: miscanthus 1/2 → r < 50
                _ if dist <= 12.0 => {
                    if r < 50 {
                        world.spawn("miscanthus", x, y);
                    }
                }
                // 森林: nanmu 1/3 → r < 33; camphor 1/4 → r < 25; bamboo 1/5 → r < 20
                _ if dist <= 19.0 => {
                    if r < 33 {
                        world.spawn("nanmu_tree", x, y);
                    }
                    if r < 25 {
                        world.spawn("camphor_tree", x, y);
                    }
                    if r < 20 {
                        world.spawn("bamboo", x, y);
                    }
                }
                // 山麓: pine 1/2 → r < 50; azalea 1/5 → r < 20
                _ if dist <= 25.0 => {
                    if r < 50 {
                        world.spawn("pine_forest", x, y);
                    }
                    if r < 20 {
                        world.spawn("azalea", x, y);
                    }
                }
                // cliff 区不散点（只保留基底地形）
                _ => {}
            }
        }
    }
}

/// 生成初始世界（地形 → 植物 → 动物）。
pub fn spawn_initial_world() -> WorldState {
    let mut world = WorldState::from_card_defs_file(crate::assets_util::card_defs_path());
    let defs = world.card_defs.values().cloned().collect::<Vec<_>>();
    spawn_concentric_world(&mut world, &defs);
    spawn_animals(&mut world, &defs);
    world
}

/// 初始卡牌数——生成后由 spawn_initial_world 决定。
pub fn initial_card_count() -> usize {
    0
}
