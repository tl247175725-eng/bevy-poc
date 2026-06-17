# Handoff 002-a1 — 创建标签定义文件

## 架构计划

**改什么：** 新建 `assets/tags.ron`（1 个文件）
**为什么：** 标签系统从字符串匹配换为位掩码 + 参数侧表，标签宇宙需要统一来源

**依据设计哲学：** `AIMemory/design-philosophy-v5.md` §1（标签即存在）、§9（元动作体系）
**依据架构决策：** 双维身体标签（位置树 + 系统树）、位掩码 512 bit

**文件结构：**

```ron
TagsConfig(
    // === 位范围分配（512 bit 总量） ===
    bit_ranges: {
        "positional":  (0, 127),    // 128 bits — 身体位置树
        "systemic":    (128, 191),   // 64 bits  — 功能系统树
        "vital":       (192, 223),   // 32 bits  — 生命关键
        "capability":  (224, 287),   // 64 bits  — 行为能力
        "material":    (288, 351),   // 64 bits  — 材质类型
        "sense":       (352, 383),   // 32 bits  — 感官
        "behavior":    (384, 447),   // 64 bits  — 行为角色
        "social":      (448, 479),   // 32 bits  — 社会结构
        "dynamic":     (480, 511),   // 32 bits  — 运行时学习动态标签
    },

    // === 位置维度（层级树） ===
    positional: {
        body: {
            head: {
                skull: {},
                brain: { vital: true },
                eye: { left: {}, right: {} },
                ear: { left: {}, right: {} },
                jaw: {},
            },
            torso: {
                spine: { nervous: true },
                ribcage: {},
                organ_heart: { vital: true, system: "circulatory" },
                organ_lung: { vital: true, system: "respiratory", count: 2 },
                organ_liver: { vital: true },
                organ_kidney: { vital: true, count: 2 },
                organ_stomach: {},
                organ_intestine: {},
                vessel_aorta: { system: "circulatory" },
            },
            limb: {
                arm: {
                    upper_arm: { bone_humerus: {} },
                    forearm: { bone_radius: {}, bone_ulna: {} },
                    hand: { finger: { count: 5 } },
                },
                leg: {
                    thigh: { bone_femur: {}, vessel_femoral: { system: "circulatory" } },
                    shin: { bone_tibia: {}, bone_fibula: {} },
                    foot: { toe: { count: 5 } },
                },
            },
        },
    },

    // === 系统维度（功能网络） ===
    systemic: {
        skeletal: {
            rule: "structural",   // 骨骼损伤→支撑力下降
            bone: {},
        },
        muscular: {
            rule: "structural",
            muscle_skeletal: {},
            muscle_cardiac: { vital: true },  // 心肌
        },
        circulatory: {
            rule: "network",      // 叶结点损伤→容量下降，不传播到父级
            vital: ["organ_heart"],
            vessel: {},
        },
        nervous: {
            rule: "network",
            vital: ["spine", "brain"],
            nerve: {},
        },
        respiratory: {
            rule: "network",
            vital: ["organ_lung"],
        },
        digestive: {
            rule: "continuous",   // 连续性管道——任一段阻塞→整条失效
            vital: [],
        },
    },

    // === 生命关键 ===
    vital: ["brain", "organ_heart", "organ_lung", "organ_liver", "organ_kidney"],

    // === 能力维度 ===
    capability: {
        move: {},
        fly: {},
        swim: {},
        climb: {},
        grasp: {},
        bite: {},
        speak: {},
        craft: {},
    },

    // === 材质维度（来自 meta_values.rs） ===
    material: {
        flesh: { density: "DENSITY_FLESH" },
        bone: { density: "DENSITY_BONE" },
        wood: { density: "DENSITY_WOOD", hardness: "HARDNESS_WOOD", yield: "YIELD_WOOD", fracture: "FRACTURE_WOOD" },
        stone: { density: "DENSITY_STONE", hardness: "HARDNESS_STONE" },
        iron: { density: "DENSITY_IRON", hardness: "HARDNESS_IRON", yield: "YIELD_IRON", fracture: "FRACTURE_IRON" },
        copper: { density: "DENSITY_COPPER", hardness: "HARDNESS_COPPER", yield: "YIELD_COPPER", fracture: "FRACTURE_COPPER" },
        bronze: { density: "DENSITY_BRONZE", hardness: "HARDNESS_BRONZE", yield: "YIELD_BRONZE", fracture: "FRACTURE_BRONZE" },
        steel: { density: "DENSITY_STEEL", hardness: "HARDNESS_STEEL", yield: "YIELD_STEEL", fracture: "FRACTURE_STEEL", toughness: "TOUGHNESS_STEEL" },
        gold: { density: "DENSITY_GOLD", hardness: "HARDNESS_GOLD" },
        silver: { density: "DENSITY_SILVER" },
        leather: { density: "DENSITY_LEATHER" },
        glass: { density: "DENSITY_GLASS", hardness: "HARDNESS_GLASS", toughness: "TOUGHNESS_GLASS" },
        clay: { density: "DENSITY_CLAY" },
        ice: { density: "DENSITY_ICE" },
        water: { density: "DENSITY_WATER" },
    },

    // === 感官维度 ===
    sense: {
        vision: { range_default: 6 },
        hearing: { range_default: 8 },
        smell: { range_default: 4 },
        touch: { range_default: 1 },
    },

    // === 行为维度 ===
    behavior: {
        predator: {},
        herbivore: {},
        omnivore: {},
        scavenger: {},
        nocturnal: {},
        diurnal: {},
        territorial: {},
        migratory: {},
    },

    // === 社会维度 ===
    social: {
        solitary: {},
        pack: { hierarchy: true },
        herd: {},
        flock: {},
        colony: {},
    },

    // === 人格/状态标签（运行时动态） ===
    personality: {
        reckless: {},
        cautious: {},
        curious: {},
        aggressive: {},
        peaceful: {},
    },

    // === 损伤状态标签（运行时动态） ===
    injury: {
        healthy: {},
        bruised: {},
        damaged: {},
        fractured: {},
        severed: {},
        missing: {},
    },
)
```

**说明：**
- 属性 `vital: true` 表示该部位是生命关键——损毁 = 死亡
- 属性 `system: "xxx"` 表示该部位属于指定功能系统
- 属性 `count: N` 表示该部位有多个（如两个肺、五根手指）
- 属性 `nervous: true` 表示该部位承担神经中枢功能
- 每种材料引用 `meta_values.rs` 的常量名
- 位分配预留 32 bit 给 `dynamic` 范围——运行时学习的知识标签
- `rule` 字段定义系统树的因果逻辑类型：structural（父→子覆盖）、network（叶→容量）、continuous（连续性）

## 架构反馈

**与设计哲学一致性：**
- 标签平铺，无继承树 ✅
- 层级通过位包含实现（父位包含所有子位）✅
- 材质引用 meta_values.rs 常量，无魔法数字 ✅
- 运行时动态标签预留空间（学习新知识）✅
- 双维身体（位置 + 系统）各自独立因果规则 ✅

**设计缺陷/未来工作：**
- 肢体左右侧（left/right arm）通过 `left:{} right:{}` 子节点表达——需要确认这种表达是否足够应对偏侧伤
- 非人形生物的身体标签未定义（鱼/鸟/虫/树）——后续 handoff
- 系统树的 `rule` 字段目前只是注释——需要后续 handoff 实现对应因果逻辑
- 损伤状态（injury 部分）的层级覆盖逻辑未实现

## 智能验收

- 文件存在且 RON 格式合法（可解析）
- 不涉及代码改动，不需要 cargo check
- 位范围总和 ≤ 512，各类别不重叠
- 所有 vital 标签都在 positional 树中有对应节点
- 所有 system 引用都在 systemic 树中有定义
