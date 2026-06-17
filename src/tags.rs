//! 标签位掩码存储 + TagRegistry
//!
//! 所有标签常量编译期定义（不解析 tags.ron）。
//! tags.ron 保留作为设计文档（人类阅读用）。
//! 禁止手工定义 TagMask 常量或手工计算 descendants。

use std::collections::HashMap;

// ===== TagInfo — 编译期标签常量 =====

pub struct TagInfo {
    pub name: &'static str,
    pub bit: u16,
    pub parent_bit: Option<u16>,
}

/// 标签常量——编译期定义，与 assets/tags.ron 树结构一致。
/// 加新标签 = 加一行常量 + 在 default_registry() 的注册列表中追加引用。
pub mod tag {
    use super::TagInfo;

    // ── positional (bit 0–38) ──
    pub const BODY: TagInfo            = TagInfo { name: "body",            bit: 0,  parent_bit: None };
    pub const HEAD: TagInfo            = TagInfo { name: "head",            bit: 1,  parent_bit: Some(0) };
    pub const SKULL: TagInfo           = TagInfo { name: "skull",           bit: 2,  parent_bit: Some(1) };
    pub const BRAIN: TagInfo           = TagInfo { name: "brain",           bit: 3,  parent_bit: Some(1) };
    pub const EYE: TagInfo             = TagInfo { name: "eye",             bit: 4,  parent_bit: Some(1) };
    pub const EYE_LEFT: TagInfo        = TagInfo { name: "left",            bit: 5,  parent_bit: Some(4) };
    pub const EYE_RIGHT: TagInfo       = TagInfo { name: "right",           bit: 6,  parent_bit: Some(4) };
    pub const EAR: TagInfo             = TagInfo { name: "ear",             bit: 7,  parent_bit: Some(1) };
    pub const EAR_LEFT: TagInfo        = TagInfo { name: "left",            bit: 8,  parent_bit: Some(7) };
    pub const EAR_RIGHT: TagInfo       = TagInfo { name: "right",           bit: 9,  parent_bit: Some(7) };
    pub const JAW: TagInfo             = TagInfo { name: "jaw",             bit: 10, parent_bit: Some(1) };
    pub const TORSO: TagInfo           = TagInfo { name: "torso",           bit: 11, parent_bit: Some(0) };
    pub const SPINE: TagInfo           = TagInfo { name: "spine",           bit: 12, parent_bit: Some(11) };
    pub const RIBCAGE: TagInfo         = TagInfo { name: "ribcage",         bit: 13, parent_bit: Some(11) };
    pub const ORGAN_HEART: TagInfo     = TagInfo { name: "organ_heart",     bit: 14, parent_bit: Some(11) };
    pub const ORGAN_LUNG: TagInfo      = TagInfo { name: "organ_lung",      bit: 15, parent_bit: Some(11) };
    pub const ORGAN_LIVER: TagInfo     = TagInfo { name: "organ_liver",     bit: 16, parent_bit: Some(11) };
    pub const ORGAN_KIDNEY: TagInfo    = TagInfo { name: "organ_kidney",    bit: 17, parent_bit: Some(11) };
    pub const ORGAN_STOMACH: TagInfo   = TagInfo { name: "organ_stomach",   bit: 18, parent_bit: Some(11) };
    pub const ORGAN_INTESTINE: TagInfo = TagInfo { name: "organ_intestine", bit: 19, parent_bit: Some(11) };
    pub const VESSEL_AORTA: TagInfo    = TagInfo { name: "vessel_aorta",    bit: 20, parent_bit: Some(11) };
    pub const LIMB: TagInfo            = TagInfo { name: "limb",            bit: 21, parent_bit: Some(0) };
    pub const ARM: TagInfo             = TagInfo { name: "arm",             bit: 22, parent_bit: Some(21) };
    pub const UPPER_ARM: TagInfo       = TagInfo { name: "upper_arm",       bit: 23, parent_bit: Some(22) };
    pub const BONE_HUMERUS: TagInfo    = TagInfo { name: "bone_humerus",    bit: 24, parent_bit: Some(23) };
    pub const FOREARM: TagInfo         = TagInfo { name: "forearm",         bit: 25, parent_bit: Some(22) };
    pub const BONE_RADIUS: TagInfo     = TagInfo { name: "bone_radius",     bit: 26, parent_bit: Some(25) };
    pub const BONE_ULNA: TagInfo       = TagInfo { name: "bone_ulna",       bit: 27, parent_bit: Some(25) };
    pub const HAND: TagInfo            = TagInfo { name: "hand",            bit: 28, parent_bit: Some(22) };
    pub const FINGER: TagInfo          = TagInfo { name: "finger",          bit: 29, parent_bit: Some(28) };
    pub const LEG: TagInfo             = TagInfo { name: "leg",             bit: 30, parent_bit: Some(21) };
    pub const THIGH: TagInfo           = TagInfo { name: "thigh",           bit: 31, parent_bit: Some(30) };
    pub const BONE_FEMUR: TagInfo      = TagInfo { name: "bone_femur",      bit: 32, parent_bit: Some(31) };
    pub const VESSEL_FEMORAL: TagInfo  = TagInfo { name: "vessel_femoral",  bit: 33, parent_bit: Some(31) };
    pub const SHIN: TagInfo            = TagInfo { name: "shin",            bit: 34, parent_bit: Some(30) };
    pub const BONE_TIBIA: TagInfo      = TagInfo { name: "bone_tibia",      bit: 35, parent_bit: Some(34) };
    pub const BONE_FIBULA: TagInfo     = TagInfo { name: "bone_fibula",     bit: 36, parent_bit: Some(34) };
    pub const FOOT: TagInfo            = TagInfo { name: "foot",            bit: 37, parent_bit: Some(30) };
    pub const TOE: TagInfo             = TagInfo { name: "toe",             bit: 38, parent_bit: Some(37) };

    // ── systemic (bit 39–49) ──
    pub const SKELETAL: TagInfo        = TagInfo { name: "skeletal",         bit: 39, parent_bit: None };
    pub const BONE: TagInfo            = TagInfo { name: "bone",             bit: 40, parent_bit: Some(39) };
    pub const MUSCULAR: TagInfo        = TagInfo { name: "muscular",         bit: 41, parent_bit: None };
    pub const MUSCLE_SKELETAL: TagInfo = TagInfo { name: "muscle_skeletal",  bit: 42, parent_bit: Some(41) };
    pub const MUSCLE_CARDIAC: TagInfo  = TagInfo { name: "muscle_cardiac",   bit: 43, parent_bit: Some(41) };
    pub const CIRCULATORY: TagInfo     = TagInfo { name: "circulatory",      bit: 44, parent_bit: None };
    pub const VESSEL: TagInfo          = TagInfo { name: "vessel",           bit: 45, parent_bit: Some(44) };
    pub const NERVOUS: TagInfo         = TagInfo { name: "nervous",          bit: 46, parent_bit: None };
    pub const NERVE: TagInfo           = TagInfo { name: "nerve",            bit: 47, parent_bit: Some(46) };
    pub const RESPIRATORY: TagInfo     = TagInfo { name: "respiratory",      bit: 48, parent_bit: None };
    pub const DIGESTIVE: TagInfo       = TagInfo { name: "digestive",        bit: 49, parent_bit: None };

    // ── capability (bit 50–57) ──
    pub const MOVE: TagInfo  = TagInfo { name: "move",  bit: 50, parent_bit: None };
    pub const FLY: TagInfo   = TagInfo { name: "fly",   bit: 51, parent_bit: None };
    pub const SWIM: TagInfo  = TagInfo { name: "swim",  bit: 52, parent_bit: None };
    pub const CLIMB: TagInfo = TagInfo { name: "climb", bit: 53, parent_bit: None };
    pub const GRASP: TagInfo = TagInfo { name: "grasp", bit: 54, parent_bit: None };
    pub const BITE: TagInfo  = TagInfo { name: "bite",  bit: 55, parent_bit: None };
    pub const SPEAK: TagInfo = TagInfo { name: "speak", bit: 56, parent_bit: None };
    pub const CRAFT: TagInfo = TagInfo { name: "craft", bit: 57, parent_bit: None };

    // ── material (bit 58–72) ──
    pub const FLESH: TagInfo   = TagInfo { name: "flesh",   bit: 58, parent_bit: None };
    pub const WOOD: TagInfo    = TagInfo { name: "wood",    bit: 59, parent_bit: None };
    pub const STONE: TagInfo   = TagInfo { name: "stone",   bit: 60, parent_bit: None };
    pub const IRON: TagInfo    = TagInfo { name: "iron",    bit: 61, parent_bit: None };
    pub const COPPER: TagInfo  = TagInfo { name: "copper",  bit: 62, parent_bit: None };
    pub const BRONZE: TagInfo  = TagInfo { name: "bronze",  bit: 63, parent_bit: None };
    pub const STEEL: TagInfo   = TagInfo { name: "steel",   bit: 64, parent_bit: None };
    pub const GOLD: TagInfo    = TagInfo { name: "gold",    bit: 65, parent_bit: None };
    pub const SILVER: TagInfo  = TagInfo { name: "silver",  bit: 66, parent_bit: None };
    pub const LEATHER: TagInfo = TagInfo { name: "leather", bit: 67, parent_bit: None };
    pub const GLASS: TagInfo   = TagInfo { name: "glass",   bit: 68, parent_bit: None };
    pub const CLAY: TagInfo    = TagInfo { name: "clay",    bit: 69, parent_bit: None };
    pub const ICE: TagInfo     = TagInfo { name: "ice",     bit: 70, parent_bit: None };
    pub const WATER: TagInfo   = TagInfo { name: "water",   bit: 71, parent_bit: None };
    // bone is already defined in systemic (bit 40); material "bone" shares same bit

    // ── sense (bit 72–75) ──
    pub const VISION: TagInfo  = TagInfo { name: "vision",  bit: 72, parent_bit: None };
    pub const HEARING: TagInfo = TagInfo { name: "hearing", bit: 73, parent_bit: None };
    pub const SMELL: TagInfo   = TagInfo { name: "smell",   bit: 74, parent_bit: None };
    pub const TOUCH: TagInfo   = TagInfo { name: "touch",   bit: 75, parent_bit: None };

    // ── behavior (bit 76–83) ──
    pub const PREDATOR: TagInfo   = TagInfo { name: "predator",   bit: 76, parent_bit: None };
    pub const HERBIVORE: TagInfo  = TagInfo { name: "herbivore",  bit: 77, parent_bit: None };
    pub const OMNIVORE: TagInfo   = TagInfo { name: "omnivore",   bit: 78, parent_bit: None };
    pub const SCAVENGER: TagInfo  = TagInfo { name: "scavenger",  bit: 79, parent_bit: None };
    pub const NOCTURNAL: TagInfo  = TagInfo { name: "nocturnal",  bit: 80, parent_bit: None };
    pub const DIURNAL: TagInfo    = TagInfo { name: "diurnal",    bit: 81, parent_bit: None };
    pub const TERRITORIAL: TagInfo = TagInfo { name: "territorial", bit: 82, parent_bit: None };
    pub const MIGRATORY: TagInfo  = TagInfo { name: "migratory",  bit: 83, parent_bit: None };

    // ── social (bit 84–88) ──
    pub const SOLITARY: TagInfo = TagInfo { name: "solitary", bit: 84, parent_bit: None };
    pub const PACK: TagInfo     = TagInfo { name: "pack",     bit: 85, parent_bit: None };
    pub const HERD: TagInfo     = TagInfo { name: "herd",     bit: 86, parent_bit: None };
    pub const FLOCK: TagInfo    = TagInfo { name: "flock",    bit: 87, parent_bit: None };
    pub const COLONY: TagInfo   = TagInfo { name: "colony",   bit: 88, parent_bit: None };

    // ── personality (bit 89–93) ──
    pub const PERSONALITY_BOLD: TagInfo = TagInfo { name: "personality:bold", bit: 89, parent_bit: None };
    pub const CAUTIOUS: TagInfo  = TagInfo { name: "cautious",  bit: 90, parent_bit: None };
    pub const CURIOUS: TagInfo   = TagInfo { name: "curious",   bit: 91, parent_bit: None };
    pub const AGGRESSIVE: TagInfo = TagInfo { name: "aggressive", bit: 92, parent_bit: None };
    pub const PERSONALITY_SOCIAL: TagInfo = TagInfo { name: "personality:social", bit: 93, parent_bit: None };

    // ── injury (bit 94–99) ──
    pub const HEALTHY: TagInfo  = TagInfo { name: "healthy",  bit: 94, parent_bit: None };
    pub const BRUISED: TagInfo  = TagInfo { name: "bruised",  bit: 95, parent_bit: None };
    pub const DAMAGED: TagInfo  = TagInfo { name: "damaged",  bit: 96, parent_bit: None };
    pub const FRACTURED: TagInfo = TagInfo { name: "fractured", bit: 97, parent_bit: None };
    pub const SEVERED: TagInfo  = TagInfo { name: "severed",  bit: 98, parent_bit: None };
    pub const MISSING: TagInfo  = TagInfo { name: "missing",  bit: 99, parent_bit: None };

    // ── trait: habitat (bit 100–106) ──
    pub const HAB_AQUATIC: TagInfo          = TagInfo { name: "habitat:aquatic",          bit: 100, parent_bit: None };
    pub const HAB_WETLAND: TagInfo          = TagInfo { name: "habitat:wetland",          bit: 101, parent_bit: None };
    pub const HAB_GRASSLAND: TagInfo        = TagInfo { name: "habitat:grassland",        bit: 102, parent_bit: None };
    pub const HAB_FOREST: TagInfo           = TagInfo { name: "habitat:forest",           bit: 103, parent_bit: None };
    pub const HAB_MOUNTAIN: TagInfo         = TagInfo { name: "habitat:mountain",         bit: 104, parent_bit: None };
    pub const HAB_SUBTERRANEAN: TagInfo     = TagInfo { name: "habitat:subterranean",     bit: 105, parent_bit: None };
    pub const HAB_AERIAL: TagInfo           = TagInfo { name: "habitat:aerial",           bit: 106, parent_bit: None };

    // ── trait: diet (bit 107–119) ──
    pub const DIET_CARNIVORE: TagInfo       = TagInfo { name: "diet:carnivore",           bit: 107, parent_bit: None };
    pub const DIET_HERBIVORE: TagInfo       = TagInfo { name: "diet:herbivore",           bit: 108, parent_bit: None };
    pub const DIET_OMNIVORE: TagInfo        = TagInfo { name: "diet:omnivore",            bit: 109, parent_bit: None };
    pub const DIET_PISCIVORE: TagInfo       = TagInfo { name: "diet:piscivore",           bit: 110, parent_bit: None };
    pub const DIET_INSECTIVORE: TagInfo     = TagInfo { name: "diet:insectivore",         bit: 111, parent_bit: None };
    pub const DIET_FRUGIVORE: TagInfo       = TagInfo { name: "diet:frugivore",           bit: 112, parent_bit: None };
    pub const DIET_GRANIVORE: TagInfo       = TagInfo { name: "diet:granivore",           bit: 113, parent_bit: None };
    pub const DIET_DETRITIVORE: TagInfo     = TagInfo { name: "diet:detritivore",         bit: 114, parent_bit: None };
    pub const DIET_SCAVENGER: TagInfo       = TagInfo { name: "diet:scavenger",           bit: 115, parent_bit: None };
    pub const DIET_NECTAR: TagInfo          = TagInfo { name: "diet:nectar_feeder",       bit: 116, parent_bit: None };
    pub const DIET_FILTER: TagInfo          = TagInfo { name: "diet:filter_feeder",       bit: 117, parent_bit: None };
    pub const DIET_WOOD: TagInfo            = TagInfo { name: "diet:wood_eater",          bit: 118, parent_bit: None };
    pub const DIET_SANGUIVORE: TagInfo      = TagInfo { name: "diet:sanguivore",          bit: 119, parent_bit: None };

    // ── trait: foraging (bit 120–128) ──
    pub const FORAGE_AMBUSH: TagInfo        = TagInfo { name: "foraging:ambush",          bit: 120, parent_bit: None };
    pub const FORAGE_PURSUIT: TagInfo       = TagInfo { name: "foraging:pursuit",         bit: 121, parent_bit: None };
    pub const FORAGE_GRAZE: TagInfo         = TagInfo { name: "foraging:graze",           bit: 122, parent_bit: None };
    pub const FORAGE_BROWSE: TagInfo        = TagInfo { name: "foraging:browse",          bit: 123, parent_bit: None };
    pub const FORAGE_SCAVENGE: TagInfo      = TagInfo { name: "foraging:scavenge",        bit: 124, parent_bit: None };
    pub const FORAGE_FILTER: TagInfo        = TagInfo { name: "foraging:filter",          bit: 125, parent_bit: None };
    pub const FORAGE_DRILL: TagInfo         = TagInfo { name: "foraging:drill",           bit: 126, parent_bit: None };
    pub const FORAGE_TRAP: TagInfo          = TagInfo { name: "foraging:trap",            bit: 127, parent_bit: None };
    pub const FORAGE_COOP_HUNT: TagInfo     = TagInfo { name: "foraging:cooperative_hunt",bit: 128, parent_bit: None };

    // ── trait: foraging_stratum (bit 129–135) ──
    pub const STRATUM_GROUND: TagInfo            = TagInfo { name: "foraging_stratum:ground",            bit: 129, parent_bit: None };
    pub const STRATUM_UNDERSTORY: TagInfo        = TagInfo { name: "foraging_stratum:understory",        bit: 130, parent_bit: None };
    pub const STRATUM_CANOPY: TagInfo            = TagInfo { name: "foraging_stratum:canopy",            bit: 131, parent_bit: None };
    pub const STRATUM_AERIAL: TagInfo            = TagInfo { name: "foraging_stratum:aerial",            bit: 132, parent_bit: None };
    pub const STRATUM_AQUATIC_SURFACE: TagInfo   = TagInfo { name: "foraging_stratum:aquatic_surface",   bit: 133, parent_bit: None };
    pub const STRATUM_AQUATIC_SUBMERGED: TagInfo = TagInfo { name: "foraging_stratum:aquatic_submerged", bit: 134, parent_bit: None };
    pub const STRATUM_BARK: TagInfo              = TagInfo { name: "foraging_stratum:bark",              bit: 135, parent_bit: None };

    // ── trait: defense (bit 136–144) ──
    pub const DEF_FLEE: TagInfo             = TagInfo { name: "defense:flee",              bit: 136, parent_bit: None };
    pub const DEF_HIDE: TagInfo             = TagInfo { name: "defense:hide",              bit: 137, parent_bit: None };
    pub const DEF_FIGHT: TagInfo            = TagInfo { name: "defense:fight",             bit: 138, parent_bit: None };
    pub const DEF_ARMOR: TagInfo            = TagInfo { name: "defense:armor",             bit: 139, parent_bit: None };
    pub const DEF_VENOM: TagInfo            = TagInfo { name: "defense:venom",             bit: 140, parent_bit: None };
    pub const DEF_CAMO: TagInfo             = TagInfo { name: "defense:camoflage",         bit: 141, parent_bit: None };
    pub const DEF_MIMICRY: TagInfo          = TagInfo { name: "defense:mimicry",           bit: 142, parent_bit: None };
    pub const DEF_CHEMICAL_SPRAY: TagInfo   = TagInfo { name: "defense:chemical_spray",    bit: 143, parent_bit: None };
    pub const DEF_AUTOTOMY: TagInfo         = TagInfo { name: "defense:autotomy",          bit: 144, parent_bit: None };

    // ── trait: thermo (bit 145–146) ──
    pub const THERMO_ENDOTHERM: TagInfo     = TagInfo { name: "thermo:endotherm",          bit: 145, parent_bit: None };
    pub const THERMO_ECTOTHERM: TagInfo     = TagInfo { name: "thermo:ectotherm",          bit: 146, parent_bit: None };

    // ── trait: metab (bit 147–150) ──
    pub const METAB_HIGH: TagInfo           = TagInfo { name: "metab:high",                bit: 147, parent_bit: None };
    pub const METAB_MEDIUM: TagInfo         = TagInfo { name: "metab:medium",              bit: 148, parent_bit: None };
    pub const METAB_LOW: TagInfo            = TagInfo { name: "metab:low",                 bit: 149, parent_bit: None };
    pub const METAB_TORPOR: TagInfo         = TagInfo { name: "metab:torpor",              bit: 150, parent_bit: None };

    // ── trait: repro (bit 151–158) ──
    pub const REPRO_FEW: TagInfo            = TagInfo { name: "repro:few_offspring",       bit: 151, parent_bit: None };
    pub const REPRO_MANY: TagInfo           = TagInfo { name: "repro:many_offspring",      bit: 152, parent_bit: None };
    pub const REPRO_PARENTAL_CARE: TagInfo  = TagInfo { name: "repro:parental_care",       bit: 153, parent_bit: None };
    pub const REPRO_NO_CARE: TagInfo        = TagInfo { name: "repro:no_parental_care",    bit: 154, parent_bit: None };
    pub const REPRO_EGG_LAYER: TagInfo      = TagInfo { name: "repro:egg_layer",           bit: 155, parent_bit: None };
    pub const REPRO_LIVE_BIRTH: TagInfo     = TagInfo { name: "repro:live_birth",          bit: 156, parent_bit: None };
    pub const REPRO_SEMELPAROUS: TagInfo    = TagInfo { name: "repro:semelparous",         bit: 157, parent_bit: None };
    pub const REPRO_ITEROPAROUS: TagInfo    = TagInfo { name: "repro:iteroparous",         bit: 158, parent_bit: None };

    // ── trait: growth (bit 159–162) ──
    pub const GROWTH_FAST: TagInfo          = TagInfo { name: "growth:fast",               bit: 159, parent_bit: None };
    pub const GROWTH_MEDIUM: TagInfo        = TagInfo { name: "growth:medium",             bit: 160, parent_bit: None };
    pub const GROWTH_SLOW: TagInfo          = TagInfo { name: "growth:slow",               bit: 161, parent_bit: None };
    pub const GROWTH_METAMORPHOSIS: TagInfo = TagInfo { name: "growth:metamorphosis",      bit: 162, parent_bit: None };

    // ── trait: social (bit 163–170) ──
    pub const SOCIAL_SOLITARY: TagInfo      = TagInfo { name: "social:solitary",           bit: 163, parent_bit: None };
    pub const SOCIAL_PAIR: TagInfo          = TagInfo { name: "social:pair",               bit: 164, parent_bit: None };
    pub const SOCIAL_PACK: TagInfo          = TagInfo { name: "social:pack",               bit: 165, parent_bit: None };
    pub const SOCIAL_HERD: TagInfo          = TagInfo { name: "social:herd",               bit: 166, parent_bit: None };
    pub const SOCIAL_COLONY: TagInfo        = TagInfo { name: "social:colony",             bit: 167, parent_bit: None };
    pub const SOCIAL_TERRITORIAL: TagInfo   = TagInfo { name: "social:territorial",        bit: 168, parent_bit: None };
    pub const SOCIAL_HIERARCHICAL: TagInfo  = TagInfo { name: "social:hierarchical",       bit: 169, parent_bit: None };
    pub const SOCIAL_EUSOCIAL: TagInfo      = TagInfo { name: "social:eusocial",           bit: 170, parent_bit: None };

    // ── trait: activity (bit 171–175) ──
    pub const ACT_DIURNAL: TagInfo          = TagInfo { name: "activity:diurnal",          bit: 171, parent_bit: None };
    pub const ACT_NOCTURNAL: TagInfo        = TagInfo { name: "activity:nocturnal",        bit: 172, parent_bit: None };
    pub const ACT_CREPUSCULAR: TagInfo      = TagInfo { name: "activity:crepuscular",      bit: 173, parent_bit: None };
    pub const ACT_CATHEMERAL: TagInfo       = TagInfo { name: "activity:cathemeral",       bit: 174, parent_bit: None };
    pub const ACT_ARRHYTHMIC: TagInfo       = TagInfo { name: "activity:arrhythmic",       bit: 175, parent_bit: None };

    // ── trait: movement (bit 176–179) ──
    pub const MOVE_SEDENTARY: TagInfo            = TagInfo { name: "movement:sedentary",             bit: 176, parent_bit: None };
    pub const MOVE_NOMADIC: TagInfo              = TagInfo { name: "movement:nomadic",               bit: 177, parent_bit: None };
    pub const MOVE_MIGRATORY: TagInfo            = TagInfo { name: "movement:migratory",             bit: 178, parent_bit: None };
    pub const MOVE_TERRITORIAL_PATROL: TagInfo   = TagInfo { name: "movement:territorial_patrol",    bit: 179, parent_bit: None };

    // ── trait: habitat_range (bit 180–181) ──
    pub const RANGE_SPECIALIST: TagInfo     = TagInfo { name: "habitat_range:specialist",   bit: 180, parent_bit: None };
    pub const RANGE_GENERALIST: TagInfo     = TagInfo { name: "habitat_range:generalist",   bit: 181, parent_bit: None };

    // ── trait: body_size (bit 182–186) ──
    pub const SIZE_TINY: TagInfo            = TagInfo { name: "body_size:tiny",             bit: 182, parent_bit: None };
    pub const SIZE_SMALL: TagInfo           = TagInfo { name: "body_size:small",            bit: 183, parent_bit: None };
    pub const SIZE_MEDIUM: TagInfo          = TagInfo { name: "body_size:medium",           bit: 184, parent_bit: None };
    pub const SIZE_LARGE: TagInfo           = TagInfo { name: "body_size:large",            bit: 185, parent_bit: None };
    pub const SIZE_HUGE: TagInfo            = TagInfo { name: "body_size:huge",             bit: 186, parent_bit: None };

    // ── trait: body_plan (bit 187–194) ──
    pub const PLAN_BIPED: TagInfo           = TagInfo { name: "body_plan:biped",            bit: 187, parent_bit: None };
    pub const PLAN_QUADRUPED: TagInfo       = TagInfo { name: "body_plan:quadruped",        bit: 188, parent_bit: None };
    pub const PLAN_SERPENTINE: TagInfo      = TagInfo { name: "body_plan:serpentine",       bit: 189, parent_bit: None };
    pub const PLAN_AVIAN: TagInfo           = TagInfo { name: "body_plan:avian",            bit: 190, parent_bit: None };
    pub const PLAN_FISH: TagInfo            = TagInfo { name: "body_plan:fish",             bit: 191, parent_bit: None };
    pub const PLAN_INSECTOID: TagInfo       = TagInfo { name: "body_plan:insectoid",        bit: 192, parent_bit: None };
    pub const PLAN_PLANT: TagInfo           = TagInfo { name: "body_plan:plant",            bit: 193, parent_bit: None };
    pub const PLAN_AMORPHOUS: TagInfo       = TagInfo { name: "body_plan:amorphous",        bit: 194, parent_bit: None };

    // ── trait: capability (bit 195–209) ──
    pub const CAP_FLY: TagInfo              = TagInfo { name: "capability:fly",             bit: 195, parent_bit: None };
    pub const CAP_SWIM: TagInfo             = TagInfo { name: "capability:swim",            bit: 196, parent_bit: None };
    pub const CAP_CLIMB: TagInfo            = TagInfo { name: "capability:climb",           bit: 197, parent_bit: None };
    pub const CAP_BURROW: TagInfo           = TagInfo { name: "capability:burrow",          bit: 198, parent_bit: None };
    pub const CAP_DIG: TagInfo              = TagInfo { name: "capability:dig",             bit: 199, parent_bit: None };
    pub const CAP_RUN: TagInfo              = TagInfo { name: "capability:run",             bit: 200, parent_bit: None };
    pub const CAP_JUMP: TagInfo             = TagInfo { name: "capability:jump",            bit: 201, parent_bit: None };
    pub const CAP_GLIDE: TagInfo            = TagInfo { name: "capability:glide",           bit: 202, parent_bit: None };
    pub const CAP_DIVE: TagInfo             = TagInfo { name: "capability:dive",            bit: 203, parent_bit: None };
    pub const CAP_GRASP: TagInfo            = TagInfo { name: "capability:grasp",           bit: 204, parent_bit: None };
    pub const CAP_BITE: TagInfo             = TagInfo { name: "capability:bite",            bit: 205, parent_bit: None };
    pub const CAP_CONSTRICT: TagInfo        = TagInfo { name: "capability:constrict",       bit: 206, parent_bit: None };
    pub const CAP_ECHOLOCATE: TagInfo       = TagInfo { name: "capability:echolocate",      bit: 207, parent_bit: None };
    pub const CAP_REGENERATE: TagInfo       = TagInfo { name: "capability:regenerate",      bit: 208, parent_bit: None };
    pub const CAP_TOOL_USE: TagInfo         = TagInfo { name: "capability:tool_use",        bit: 209, parent_bit: None };

    // ── trait: cognition (bit 210–214) ──
    pub const COG_INSTINCT: TagInfo         = TagInfo { name: "cognition:instinct_only",            bit: 210, parent_bit: None };
    pub const COG_BASIC_LEARNING: TagInfo   = TagInfo { name: "cognition:basic_learning",           bit: 211, parent_bit: None };
    pub const COG_TOOL_USE: TagInfo         = TagInfo { name: "cognition:tool_use",                 bit: 212, parent_bit: None };
    pub const COG_COMPLEX_REASONING: TagInfo= TagInfo { name: "cognition:complex_reasoning",        bit: 213, parent_bit: None };
    pub const COG_CULTURAL: TagInfo         = TagInfo { name: "cognition:cultural_transmission",    bit: 214, parent_bit: None };

    // ── trait: sense (bit 215–222) ──
    pub const SENSE_VISION: TagInfo         = TagInfo { name: "sense:vision",               bit: 215, parent_bit: None };
    pub const SENSE_HEARING: TagInfo        = TagInfo { name: "sense:hearing",              bit: 216, parent_bit: None };
    pub const SENSE_SMELL: TagInfo          = TagInfo { name: "sense:smell",                bit: 217, parent_bit: None };
    pub const SENSE_TOUCH: TagInfo          = TagInfo { name: "sense:touch",                bit: 218, parent_bit: None };
    pub const SENSE_ECHOLOCATION: TagInfo   = TagInfo { name: "sense:echolocation",         bit: 219, parent_bit: None };
    pub const SENSE_INFRARED: TagInfo       = TagInfo { name: "sense:infrared",             bit: 220, parent_bit: None };
    pub const SENSE_ELECTROSENSE: TagInfo   = TagInfo { name: "sense:electrosense",         bit: 221, parent_bit: None };
    pub const SENSE_MAGNETOSENSE: TagInfo   = TagInfo { name: "sense:magnetosense",         bit: 222, parent_bit: None };

    // ── trait: state (bit 223–230) ──
    pub const STATE_HEALTHY: TagInfo        = TagInfo { name: "state:healthy",              bit: 223, parent_bit: None };
    pub const STATE_INJURED: TagInfo        = TagInfo { name: "state:injured",              bit: 224, parent_bit: None };
    pub const STATE_SICK: TagInfo           = TagInfo { name: "state:sick",                 bit: 225, parent_bit: None };
    pub const STATE_STARVING: TagInfo       = TagInfo { name: "state:starving",             bit: 226, parent_bit: None };
    pub const STATE_EXHAUSTED: TagInfo      = TagInfo { name: "state:exhausted",            bit: 227, parent_bit: None };
    pub const STATE_PREGNANT: TagInfo       = TagInfo { name: "state:pregnant",             bit: 228, parent_bit: None };
    pub const STATE_GROWING: TagInfo        = TagInfo { name: "state:growing",              bit: 229, parent_bit: None };
    pub const STATE_DYING: TagInfo          = TagInfo { name: "state:dying",                bit: 230, parent_bit: None };

    // ── trait: nutrition (bit 231–235) ──
    pub const NUTRITION_AUTOTROPH: TagInfo       = TagInfo { name: "nutrition:autotroph",         bit: 231, parent_bit: None };
    pub const NUTRITION_HEMIPARASITIC: TagInfo   = TagInfo { name: "nutrition:hemiparasitic",     bit: 232, parent_bit: None };
    pub const NUTRITION_HOLOPARASITIC: TagInfo   = TagInfo { name: "nutrition:holoparasitic",     bit: 233, parent_bit: None };
    pub const NUTRITION_CARNIVOROUS: TagInfo     = TagInfo { name: "nutrition:carnivorous",       bit: 234, parent_bit: None };
    pub const NUTRITION_DETRITIVOROUS: TagInfo   = TagInfo { name: "nutrition:detritivorous",     bit: 235, parent_bit: None };

    // ── trait: fire_response (bit 236–238) ──
    pub const FIRE_KILLED: TagInfo               = TagInfo { name: "fire_response:killed",         bit: 236, parent_bit: None };
    pub const FIRE_RESPROUTING: TagInfo          = TagInfo { name: "fire_response:resprouting",    bit: 237, parent_bit: None };
    pub const FIRE_DEPENDENT: TagInfo            = TagInfo { name: "fire_response:fire_dependent", bit: 238, parent_bit: None };

    // ── trait: drought_tolerance (bit 239–241) ──
    pub const DROUGHT_LOW: TagInfo               = TagInfo { name: "drought_tolerance:low",        bit: 239, parent_bit: None };
    pub const DROUGHT_MEDIUM: TagInfo            = TagInfo { name: "drought_tolerance:medium",     bit: 240, parent_bit: None };
    pub const DROUGHT_HIGH: TagInfo              = TagInfo { name: "drought_tolerance:high",       bit: 241, parent_bit: None };

    // ── trait: flooding_tolerance (bit 242–244) ──
    pub const FLOOD_LOW: TagInfo                 = TagInfo { name: "flooding_tolerance:low",       bit: 242, parent_bit: None };
    pub const FLOOD_MEDIUM: TagInfo              = TagInfo { name: "flooding_tolerance:medium",    bit: 243, parent_bit: None };
    pub const FLOOD_HIGH: TagInfo                = TagInfo { name: "flooding_tolerance:high",      bit: 244, parent_bit: None };

    // ── trait: shade_tolerance (bit 245–247) ──
    pub const SHADE_LOW: TagInfo                 = TagInfo { name: "shade_tolerance:low",          bit: 245, parent_bit: None };
    pub const SHADE_MEDIUM: TagInfo              = TagInfo { name: "shade_tolerance:medium",       bit: 246, parent_bit: None };
    pub const SHADE_HIGH: TagInfo                = TagInfo { name: "shade_tolerance:high",         bit: 247, parent_bit: None };

    // ── trait: dispersal (bit 248–252) ──
    pub const DISPERSAL_WIND: TagInfo            = TagInfo { name: "dispersal:wind",               bit: 248, parent_bit: None };
    pub const DISPERSAL_ANIMAL: TagInfo          = TagInfo { name: "dispersal:animal",             bit: 249, parent_bit: None };
    pub const DISPERSAL_WATER: TagInfo           = TagInfo { name: "dispersal:water",              bit: 250, parent_bit: None };
    pub const DISPERSAL_EXPLOSIVE: TagInfo       = TagInfo { name: "dispersal:explosive",          bit: 251, parent_bit: None };
    pub const DISPERSAL_GRAVITY: TagInfo         = TagInfo { name: "dispersal:gravity",            bit: 252, parent_bit: None };

    // ── trait: growth_form (bit 253–258) ──
    pub const FORM_TREE: TagInfo                 = TagInfo { name: "growth_form:tree",             bit: 253, parent_bit: None };
    pub const FORM_SHRUB: TagInfo                = TagInfo { name: "growth_form:shrub",            bit: 254, parent_bit: None };
    pub const FORM_GRASS: TagInfo                = TagInfo { name: "growth_form:grass",            bit: 255, parent_bit: None };
    pub const FORM_VINE: TagInfo                 = TagInfo { name: "growth_form:vine",             bit: 256, parent_bit: None };
    pub const FORM_AQUATIC: TagInfo              = TagInfo { name: "growth_form:aquatic",          bit: 257, parent_bit: None };
    pub const FORM_EPIPHYTE: TagInfo             = TagInfo { name: "growth_form:epiphyte",         bit: 258, parent_bit: None };

    // ── trait: woodiness (bit 259–260) ──
    pub const WOODINESS_WOODY: TagInfo           = TagInfo { name: "woodiness:woody",              bit: 259, parent_bit: None };
    pub const WOODINESS_HERBACEOUS: TagInfo      = TagInfo { name: "woodiness:herbaceous",         bit: 260, parent_bit: None };

    // ── trait: lifespan (bit 261–263) ──
    pub const LIFESPAN_ANNUAL: TagInfo           = TagInfo { name: "lifespan:annual",             bit: 261, parent_bit: None };
    pub const LIFESPAN_PERENNIAL: TagInfo        = TagInfo { name: "lifespan:perennial",          bit: 262, parent_bit: None };
    pub const LIFESPAN_LONG_LIVED: TagInfo       = TagInfo { name: "lifespan:long_lived",         bit: 263, parent_bit: None };

    // ── 基础分类标签 (bit 264–268) ──
    pub const BASE_TERRAIN: TagInfo  = TagInfo { name: "terrain",  bit: 264, parent_bit: None };
    pub const BASE_TREE: TagInfo     = TagInfo { name: "tree",     bit: 265, parent_bit: None };
    pub const BASE_PLANT: TagInfo    = TagInfo { name: "plant",    bit: 266, parent_bit: None };
    pub const BASE_ANIMAL: TagInfo   = TagInfo { name: "animal",   bit: 267, parent_bit: None };
    pub const BASE_FISH: TagInfo     = TagInfo { name: "fish",     bit: 268, parent_bit: None };

    // ── state:dead (bit 269) ──
    pub const STATE_DEAD: TagInfo    = TagInfo { name: "state:dead", bit: 269, parent_bit: None };
}

// ===== TagBits — 512-bit 位掩码存储 =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagBits {
    bits: [u64; 8], // 512 bits total
}

impl TagBits {
    pub fn new() -> Self {
        Self { bits: [0; 8] }
    }

    pub fn set(&mut self, bit: u16) {
        self.bits[(bit / 64) as usize] |= 1 << (bit % 64);
    }

    #[allow(dead_code)]
    pub fn unset(&mut self, bit: u16) {
        self.bits[(bit / 64) as usize] &= !(1 << (bit % 64));
    }

    pub fn has(&self, bit: u16) -> bool {
        (self.bits[(bit / 64) as usize] >> (bit % 64)) & 1 == 1
    }

    pub fn union(&self, other: &TagBits) -> TagBits {
        let mut result = TagBits::new();
        for i in 0..8 {
            result.bits[i] = self.bits[i] | other.bits[i];
        }
        result
    }

    /// 将标签名列表转为 TagBits。
    /// 不在 registry 中的标签名被静默跳过。
    pub fn from_tag_names(names: &[String], registry: &TagRegistry) -> Self {
        let mut bits = TagBits::new();
        for name in names {
            if let Some(&bit) = registry.name_to_bit.get(name.as_str()) {
                bits.set(bit);
            }
        }
        bits
    }
}

// ===== TagRegistry =====

#[derive(Debug)]
pub struct TagRegistry {
    /// 标签名 → bit 位置
    pub(crate) name_to_bit: HashMap<String, u16>,
    /// bit 位置 → 标签名
    bit_to_name: HashMap<u16, String>,
    /// 标签 bit → 所有后代 bit 的位掩码 (用于 has_descendant_of)
    descendants: HashMap<u16, TagBits>,
    /// 下一个可分配的 bit
    next_bit: u16,
}

impl TagRegistry {
    /// 从编译期常量构建 TagRegistry。
    pub fn default_registry() -> Self {
        let mut reg = Self {
            name_to_bit: HashMap::new(),
            bit_to_name: HashMap::new(),
            descendants: HashMap::new(),
            next_bit: 270,
        };

        // 注册所有 tag 常量
        // 注意: 重名标签（如 "left"/"right"/"bone"）后注册的会覆盖先注册的。
        // 这与 v1 from_tags_ron 中 HashMap insert 覆盖行为一致。
        reg.register_all(&TAG_CONSTANTS);

        // 计算 descendants：遍历所有标签，构建 parent→children 映射，
        // 然后递归计算每个子树的位掩码。
        reg.compute_descendants();

        reg
    }

    /// 批量注册 TagInfo 常量。
    fn register_all(&mut self, infos: &[&'static TagInfo]) {
        for info in infos {
            // 跟踪最大 bit 用于 next_bit
            if info.bit >= self.next_bit {
                self.next_bit = info.bit + 1;
            }
            self.name_to_bit.insert(info.name.to_string(), info.bit);
            self.bit_to_name.insert(info.bit, info.name.to_string());
        }
    }

    /// 从 parent_bit 关系构建 descendants 位掩码。
    fn compute_descendants(&mut self) {
        // 构建 parent_bit → Vec<child_bit>
        let mut children: HashMap<u16, Vec<u16>> = HashMap::new();
        for info in TAG_CONSTANTS.iter() {
            if let Some(parent) = info.parent_bit {
                children.entry(parent).or_default().push(info.bit);
            }
        }
        // 同时添加自身到 children（根节点也需要处理）
        let all_bits: Vec<u16> = TAG_CONSTANTS.iter().map(|info| info.bit).collect();

        // 递归计算每个节点的 descendants 位掩码
        for bit in &all_bits {
            if !self.descendants.contains_key(bit) {
                let desc = self.compute_descendants_rec(*bit, &children);
                self.descendants.insert(*bit, desc);
            }
        }
    }

    /// 递归计算单个节点的后代位掩码。
    fn compute_descendants_rec(&self, bit: u16, children: &HashMap<u16, Vec<u16>>) -> TagBits {
        let mut desc = TagBits::new();
        if let Some(kids) = children.get(&bit) {
            for &child_bit in kids {
                desc.set(child_bit);
                // 递归合并子节点的后代
                let grand_desc = self.compute_descendants_rec(child_bit, children);
                desc = desc.union(&grand_desc);
            }
        }
        desc
    }

    /// 子树检查：child_bit 是 parent_bit 的后代吗？
    pub fn is_descendant_of(&self, child_bit: u16, parent_bit: u16) -> bool {
        self.descendants
            .get(&parent_bit)
            .map(|desc_bits| desc_bits.has(child_bit))
            .unwrap_or(false)
    }

    /// 注册标签与其后代位掩码（测试用辅助）。
    pub fn register(&mut self, name: &str, descendants: TagBits) -> u16 {
        let bit = self.next_bit;
        self.name_to_bit.insert(name.to_string(), bit);
        self.bit_to_name.insert(bit, name.to_string());
        self.descendants.insert(bit, descendants);
        self.next_bit += 1;
        bit
    }
}

/// 所有 tag 常量引用列表。`default_registry()` 遍历此切片注册。
/// 加新标签常量后，在此列表中追加一行 `&tag::NEW_TAG`。
static TAG_CONSTANTS: &[&TagInfo] = &[
    // positional
    &tag::BODY,
    &tag::HEAD,
    &tag::SKULL,
    &tag::BRAIN,
    &tag::EYE,
    &tag::EYE_LEFT,
    &tag::EYE_RIGHT,
    &tag::EAR,
    &tag::EAR_LEFT,
    &tag::EAR_RIGHT,
    &tag::JAW,
    &tag::TORSO,
    &tag::SPINE,
    &tag::RIBCAGE,
    &tag::ORGAN_HEART,
    &tag::ORGAN_LUNG,
    &tag::ORGAN_LIVER,
    &tag::ORGAN_KIDNEY,
    &tag::ORGAN_STOMACH,
    &tag::ORGAN_INTESTINE,
    &tag::VESSEL_AORTA,
    &tag::LIMB,
    &tag::ARM,
    &tag::UPPER_ARM,
    &tag::BONE_HUMERUS,
    &tag::FOREARM,
    &tag::BONE_RADIUS,
    &tag::BONE_ULNA,
    &tag::HAND,
    &tag::FINGER,
    &tag::LEG,
    &tag::THIGH,
    &tag::BONE_FEMUR,
    &tag::VESSEL_FEMORAL,
    &tag::SHIN,
    &tag::BONE_TIBIA,
    &tag::BONE_FIBULA,
    &tag::FOOT,
    &tag::TOE,
    // systemic
    &tag::SKELETAL,
    &tag::BONE,
    &tag::MUSCULAR,
    &tag::MUSCLE_SKELETAL,
    &tag::MUSCLE_CARDIAC,
    &tag::CIRCULATORY,
    &tag::VESSEL,
    &tag::NERVOUS,
    &tag::NERVE,
    &tag::RESPIRATORY,
    &tag::DIGESTIVE,
    // capability
    &tag::MOVE,
    &tag::FLY,
    &tag::SWIM,
    &tag::CLIMB,
    &tag::GRASP,
    &tag::BITE,
    &tag::SPEAK,
    &tag::CRAFT,
    // material
    &tag::FLESH,
    &tag::WOOD,
    &tag::STONE,
    &tag::IRON,
    &tag::COPPER,
    &tag::BRONZE,
    &tag::STEEL,
    &tag::GOLD,
    &tag::SILVER,
    &tag::LEATHER,
    &tag::GLASS,
    &tag::CLAY,
    &tag::ICE,
    &tag::WATER,
    // sense
    &tag::VISION,
    &tag::HEARING,
    &tag::SMELL,
    &tag::TOUCH,
    // behavior
    &tag::PREDATOR,
    &tag::HERBIVORE,
    &tag::OMNIVORE,
    &tag::SCAVENGER,
    &tag::NOCTURNAL,
    &tag::DIURNAL,
    &tag::TERRITORIAL,
    &tag::MIGRATORY,
    // social
    &tag::SOLITARY,
    &tag::PACK,
    &tag::HERD,
    &tag::FLOCK,
    &tag::COLONY,
    // personality
    &tag::PERSONALITY_BOLD,
    &tag::CAUTIOUS,
    &tag::CURIOUS,
    &tag::AGGRESSIVE,
    &tag::PERSONALITY_SOCIAL,
    // injury
    &tag::HEALTHY,
    &tag::BRUISED,
    &tag::DAMAGED,
    &tag::FRACTURED,
    &tag::SEVERED,
    &tag::MISSING,
    // trait: habitat
    &tag::HAB_AQUATIC,
    &tag::HAB_WETLAND,
    &tag::HAB_GRASSLAND,
    &tag::HAB_FOREST,
    &tag::HAB_MOUNTAIN,
    &tag::HAB_SUBTERRANEAN,
    &tag::HAB_AERIAL,
    // trait: diet
    &tag::DIET_CARNIVORE,
    &tag::DIET_HERBIVORE,
    &tag::DIET_OMNIVORE,
    &tag::DIET_PISCIVORE,
    &tag::DIET_INSECTIVORE,
    &tag::DIET_FRUGIVORE,
    &tag::DIET_GRANIVORE,
    &tag::DIET_DETRITIVORE,
    &tag::DIET_SCAVENGER,
    &tag::DIET_NECTAR,
    &tag::DIET_FILTER,
    &tag::DIET_WOOD,
    &tag::DIET_SANGUIVORE,
    // trait: foraging
    &tag::FORAGE_AMBUSH,
    &tag::FORAGE_PURSUIT,
    &tag::FORAGE_GRAZE,
    &tag::FORAGE_BROWSE,
    &tag::FORAGE_SCAVENGE,
    &tag::FORAGE_FILTER,
    &tag::FORAGE_DRILL,
    &tag::FORAGE_TRAP,
    &tag::FORAGE_COOP_HUNT,
    // trait: foraging_stratum
    &tag::STRATUM_GROUND,
    &tag::STRATUM_UNDERSTORY,
    &tag::STRATUM_CANOPY,
    &tag::STRATUM_AERIAL,
    &tag::STRATUM_AQUATIC_SURFACE,
    &tag::STRATUM_AQUATIC_SUBMERGED,
    &tag::STRATUM_BARK,
    // trait: defense
    &tag::DEF_FLEE,
    &tag::DEF_HIDE,
    &tag::DEF_FIGHT,
    &tag::DEF_ARMOR,
    &tag::DEF_VENOM,
    &tag::DEF_CAMO,
    &tag::DEF_MIMICRY,
    &tag::DEF_CHEMICAL_SPRAY,
    &tag::DEF_AUTOTOMY,
    // trait: thermo
    &tag::THERMO_ENDOTHERM,
    &tag::THERMO_ECTOTHERM,
    // trait: metab
    &tag::METAB_HIGH,
    &tag::METAB_MEDIUM,
    &tag::METAB_LOW,
    &tag::METAB_TORPOR,
    // trait: repro
    &tag::REPRO_FEW,
    &tag::REPRO_MANY,
    &tag::REPRO_PARENTAL_CARE,
    &tag::REPRO_NO_CARE,
    &tag::REPRO_EGG_LAYER,
    &tag::REPRO_LIVE_BIRTH,
    &tag::REPRO_SEMELPAROUS,
    &tag::REPRO_ITEROPAROUS,
    // trait: growth
    &tag::GROWTH_FAST,
    &tag::GROWTH_MEDIUM,
    &tag::GROWTH_SLOW,
    &tag::GROWTH_METAMORPHOSIS,
    // trait: social
    &tag::SOCIAL_SOLITARY,
    &tag::SOCIAL_PAIR,
    &tag::SOCIAL_PACK,
    &tag::SOCIAL_HERD,
    &tag::SOCIAL_COLONY,
    &tag::SOCIAL_TERRITORIAL,
    &tag::SOCIAL_HIERARCHICAL,
    &tag::SOCIAL_EUSOCIAL,
    // trait: activity
    &tag::ACT_DIURNAL,
    &tag::ACT_NOCTURNAL,
    &tag::ACT_CREPUSCULAR,
    &tag::ACT_CATHEMERAL,
    &tag::ACT_ARRHYTHMIC,
    // trait: movement
    &tag::MOVE_SEDENTARY,
    &tag::MOVE_NOMADIC,
    &tag::MOVE_MIGRATORY,
    &tag::MOVE_TERRITORIAL_PATROL,
    // trait: habitat_range
    &tag::RANGE_SPECIALIST,
    &tag::RANGE_GENERALIST,
    // trait: body_size
    &tag::SIZE_TINY,
    &tag::SIZE_SMALL,
    &tag::SIZE_MEDIUM,
    &tag::SIZE_LARGE,
    &tag::SIZE_HUGE,
    // trait: body_plan
    &tag::PLAN_BIPED,
    &tag::PLAN_QUADRUPED,
    &tag::PLAN_SERPENTINE,
    &tag::PLAN_AVIAN,
    &tag::PLAN_FISH,
    &tag::PLAN_INSECTOID,
    &tag::PLAN_PLANT,
    &tag::PLAN_AMORPHOUS,
    // trait: capability
    &tag::CAP_FLY,
    &tag::CAP_SWIM,
    &tag::CAP_CLIMB,
    &tag::CAP_BURROW,
    &tag::CAP_DIG,
    &tag::CAP_RUN,
    &tag::CAP_JUMP,
    &tag::CAP_GLIDE,
    &tag::CAP_DIVE,
    &tag::CAP_GRASP,
    &tag::CAP_BITE,
    &tag::CAP_CONSTRICT,
    &tag::CAP_ECHOLOCATE,
    &tag::CAP_REGENERATE,
    &tag::CAP_TOOL_USE,
    // trait: cognition
    &tag::COG_INSTINCT,
    &tag::COG_BASIC_LEARNING,
    &tag::COG_TOOL_USE,
    &tag::COG_COMPLEX_REASONING,
    &tag::COG_CULTURAL,
    // trait: sense
    &tag::SENSE_VISION,
    &tag::SENSE_HEARING,
    &tag::SENSE_SMELL,
    &tag::SENSE_TOUCH,
    &tag::SENSE_ECHOLOCATION,
    &tag::SENSE_INFRARED,
    &tag::SENSE_ELECTROSENSE,
    &tag::SENSE_MAGNETOSENSE,
    // trait: state
    &tag::STATE_HEALTHY,
    &tag::STATE_INJURED,
    &tag::STATE_SICK,
    &tag::STATE_STARVING,
    &tag::STATE_EXHAUSTED,
    &tag::STATE_PREGNANT,
    &tag::STATE_GROWING,
    &tag::STATE_DYING,
    // trait: nutrition
    &tag::NUTRITION_AUTOTROPH,
    &tag::NUTRITION_HEMIPARASITIC,
    &tag::NUTRITION_HOLOPARASITIC,
    &tag::NUTRITION_CARNIVOROUS,
    &tag::NUTRITION_DETRITIVOROUS,
    // trait: fire_response
    &tag::FIRE_KILLED,
    &tag::FIRE_RESPROUTING,
    &tag::FIRE_DEPENDENT,
    // trait: drought_tolerance
    &tag::DROUGHT_LOW,
    &tag::DROUGHT_MEDIUM,
    &tag::DROUGHT_HIGH,
    // trait: flooding_tolerance
    &tag::FLOOD_LOW,
    &tag::FLOOD_MEDIUM,
    &tag::FLOOD_HIGH,
    // trait: shade_tolerance
    &tag::SHADE_LOW,
    &tag::SHADE_MEDIUM,
    &tag::SHADE_HIGH,
    // trait: dispersal
    &tag::DISPERSAL_WIND,
    &tag::DISPERSAL_ANIMAL,
    &tag::DISPERSAL_WATER,
    &tag::DISPERSAL_EXPLOSIVE,
    &tag::DISPERSAL_GRAVITY,
    // trait: growth_form
    &tag::FORM_TREE,
    &tag::FORM_SHRUB,
    &tag::FORM_GRASS,
    &tag::FORM_VINE,
    &tag::FORM_AQUATIC,
    &tag::FORM_EPIPHYTE,
    // trait: woodiness
    &tag::WOODINESS_WOODY,
    &tag::WOODINESS_HERBACEOUS,
    // trait: lifespan
    &tag::LIFESPAN_ANNUAL,
    &tag::LIFESPAN_PERENNIAL,
    &tag::LIFESPAN_LONG_LIVED,
    // 基础分类标签
    &tag::BASE_TERRAIN,
    &tag::BASE_TREE,
    &tag::BASE_PLANT,
    &tag::BASE_ANIMAL,
    &tag::BASE_FISH,
    &tag::STATE_DEAD,
];

// ===== TagQuery — 便捷查询 trait =====

pub trait TagQuery {
    fn has_tag(&self, registry: &TagRegistry, name: &str) -> bool;
    fn has_descendant_of(&self, registry: &TagRegistry, parent_bit: u16) -> bool;
}

impl TagQuery for TagBits {
    fn has_tag(&self, registry: &TagRegistry, name: &str) -> bool {
        registry
            .name_to_bit
            .get(name)
            .map(|&bit| self.has(bit))
            .unwrap_or(false)
    }

    fn has_descendant_of(&self, registry: &TagRegistry, parent_bit: u16) -> bool {
        registry
            .descendants
            .get(&parent_bit)
            .map(|desc_bits| {
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

// ===== 测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    // ---- TagBits 基础 ----

    #[test]
    fn tagbits_set_and_has() {
        let mut bits = TagBits::new();
        bits.set(5);
        assert!(bits.has(5));
        assert!(!bits.has(6));
    }

    #[test]
    fn tagbits_multiple_words() {
        let mut bits = TagBits::new();
        bits.set(70);
        assert!(bits.has(70));
        assert!(!bits.has(69));
        assert!(!bits.has(71));
    }

    #[test]
    fn tagbits_union() {
        let mut a = TagBits::new();
        a.set(10);
        let mut b = TagBits::new();
        b.set(20);
        let u = a.union(&b);
        assert!(u.has(10));
        assert!(u.has(20));
        assert!(!u.has(30));
    }

    // ---- TagRegistry: 手动注册 ----

    #[test]
    fn registry_allocate_bits() {
        let mut reg = TagRegistry {
            name_to_bit: HashMap::new(),
            bit_to_name: HashMap::new(),
            descendants: HashMap::new(),
            next_bit: 0,
        };

        let skull_desc = TagBits::new();
        assert_eq!(reg.register("skull", skull_desc), 0);

        let mut head_desc = TagBits::new();
        head_desc.set(0);
        assert_eq!(reg.register("head", head_desc), 1);

        let mut body_desc = TagBits::new();
        body_desc.set(0);
        body_desc.set(1);
        assert_eq!(reg.register("body", body_desc), 2);

        assert!(reg.is_descendant_of(1, 2));
        assert!(reg.is_descendant_of(0, 2));
        assert!(reg.is_descendant_of(0, 1));
        assert!(!reg.is_descendant_of(2, 1));
        assert!(!reg.is_descendant_of(2, 2));

        let mut entity_bits = TagBits::new();
        entity_bits.set(0);
        assert!(entity_bits.has_tag(&reg, "skull"));
        assert!(!entity_bits.has_tag(&reg, "head"));
        assert!(entity_bits.has_descendant_of(&reg, 2));
        assert!(entity_bits.has_descendant_of(&reg, 1));
    }

    // ---- TagRegistry: 从编译期常量构建 ----

    #[test]
    fn default_registry_name_to_bit_non_empty() {
        let reg = TagRegistry::default_registry();
        assert!(!reg.name_to_bit.is_empty(), "TagRegistry 应包含至少一个标签");
    }

    #[test]
    fn body_descendants_contain_head() {
        let reg = TagRegistry::default_registry();
        let body_bit = reg.name_to_bit.get("body").expect("body 标签应存在");
        let head_bit = reg.name_to_bit.get("head").expect("head 标签应存在");
        assert!(
            reg.is_descendant_of(*head_bit, *body_bit),
            "head 应是 body 的后代"
        );
    }

    #[test]
    fn head_descendants_contain_skull() {
        let reg = TagRegistry::default_registry();
        let head_bit = reg.name_to_bit.get("head").expect("head 标签应存在");
        let skull_bit = reg.name_to_bit.get("skull").expect("skull 标签应存在");
        assert!(
            reg.is_descendant_of(*skull_bit, *head_bit),
            "skull 应是 head 的后代"
        );
    }

    #[test]
    fn body_descendants_contain_finger() {
        let reg = TagRegistry::default_registry();
        let body_bit = reg.name_to_bit.get("body").expect("body 标签应存在");
        let finger_bit = reg.name_to_bit.get("finger").expect("finger 标签应存在");
        assert!(
            reg.is_descendant_of(*finger_bit, *body_bit),
            "finger 应是 body 的后代（深层嵌套）"
        );
    }

    #[test]
    fn systemic_nodes_have_bits() {
        let reg = TagRegistry::default_registry();
        assert!(reg.name_to_bit.contains_key("skeletal"));
        assert!(reg.name_to_bit.contains_key("bone"));
        assert!(reg.name_to_bit.contains_key("circulatory"));
        assert!(reg.name_to_bit.contains_key("vessel"));
    }

    #[test]
    fn brain_bit_in_positional_range() {
        let reg = TagRegistry::default_registry();
        let brain_bit = reg.name_to_bit.get("brain").expect("brain 标签应存在");
        // brain 定义在 positional 范围 (bit 3)
        assert_eq!(*brain_bit, 3);
    }

    #[test]
    fn from_tag_names_maps_correctly() {
        let reg = TagRegistry::default_registry();
        let tags = vec![
            "predator".to_string(),
            "head".to_string(),
            "nonexistent_tag".to_string(),
        ];
        let bits = TagBits::from_tag_names(&tags, &reg);
        let pred_bit = reg.name_to_bit.get("predator").unwrap();
        let head_bit = reg.name_to_bit.get("head").unwrap();
        assert!(bits.has(*pred_bit), "predator 标签应映射");
        assert!(bits.has(*head_bit), "head 标签应映射");
        // nonexistent_tag 被静默跳过
    }

    #[test]
    fn behavior_tags_exist() {
        let reg = TagRegistry::default_registry();
        for name in &["predator", "herbivore", "omnivore", "scavenger"] {
            let bit = reg.name_to_bit.get(*name);
            assert!(bit.is_some(), "{} 标签应存在", name);
        }
    }

    #[test]
    fn bone_descendant_of_skeletal() {
        let reg = TagRegistry::default_registry();
        let skeletal_bit = reg.name_to_bit.get("skeletal").expect("skeletal 应存在");
        let bone_bit = reg.name_to_bit.get("bone").expect("bone 应存在");
        assert!(
            reg.is_descendant_of(*bone_bit, *skeletal_bit),
            "bone 应是 skeletal 的后代"
        );
    }
}
