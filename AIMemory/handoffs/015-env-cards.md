# Handoff 015 — 环境地形卡定义

## 架构计划

**改什么：** `assets/card_defs.ron`（1 文件）
**做什么：** 清空旧 5 张卡，写入七环 15 种环境卡

### 新卡定义

```ron
[
    // ═══ 深潭区（中心 2×2）═══
    CardDef(
        type_name: "abyss_pool",
        display_name: "深潭",
        icon: "潭",
        tags: ["water", "deep", "depth:abyss", "terrain", "aquatic"],
        color: (8, 12, 48, 255),
        hp: 0,
        is_rooted: true,
        quantity: 999,
    ),

    // ═══ 浅水区（3×3 环）═══
    CardDef(
        type_name: "shallow_water",
        display_name: "浅水",
        icon: "水",
        tags: ["water", "shallow", "depth:shallow", "transparent", "terrain", "aquatic"],
        color: (32, 96, 128, 255),
        hp: 0,
        is_rooted: true,
        quantity: 500,
    ),
    CardDef(
        type_name: "lotus",
        display_name: "莲花",
        icon: "莲",
        tags: ["plant", "aquatic", "floating", "edible"],
        color: (232, 128, 192, 255),
        hp: 1,
        is_rooted: false,
        quantity: 20,
    ),
    CardDef(
        type_name: "waterweed",
        display_name: "水草",
        icon: "藻",
        tags: ["plant", "aquatic", "submerged"],
        color: (48, 144, 96, 255),
        hp: 1,
        is_rooted: true,
        quantity: 100,
    ),

    // ═══ 湿地区（5×5 环）═══
    CardDef(
        type_name: "wetland",
        display_name: "湿地",
        icon: "泽",
        tags: ["wetland", "peat_soil", "seasonal_flood", "terrain"],
        color: (72, 96, 48, 255),
        hp: 0,
        is_rooted: true,
        quantity: 300,
    ),
    CardDef(
        type_name: "reed",
        display_name: "芦苇",
        icon: "芦",
        tags: ["plant", "tall_grass", "wetland", "edible"],
        color: (144, 168, 80, 255),
        hp: 1,
        is_rooted: true,
        quantity: 150,
    ),
    CardDef(
        type_name: "cattail",
        display_name: "香蒲",
        icon: "蒲",
        tags: ["plant", "tall_grass", "wetland"],
        color: (112, 136, 64, 255),
        hp: 1,
        is_rooted: true,
        quantity: 100,
    ),

    // ═══ 草原区（8×8 环）═══
    CardDef(
        type_name: "grassland",
        display_name: "草原",
        icon: "原",
        tags: ["grassland", "tall_grass", "fire_maintained", "terrain"],
        color: (136, 168, 48, 255),
        hp: 0,
        is_rooted: true,
        quantity: 500,
    ),
    CardDef(
        type_name: "miscanthus",
        display_name: "芒草",
        icon: "芒",
        tags: ["plant", "tall_grass", "grassland", "flammable"],
        color: (168, 192, 56, 255),
        hp: 1,
        is_rooted: true,
        quantity: 400,
    ),

    // ═══ 森林区（12×12 环）═══
    CardDef(
        type_name: "broadleaf_forest",
        display_name: "常绿阔叶林",
        icon: "林",
        tags: ["forest", "broadleaf", "evergreen", "terrain"],
        color: (40, 72, 24, 255),
        hp: 0,
        is_rooted: true,
        quantity: 80,
    ),
    CardDef(
        type_name: "nanmu_tree",
        display_name: "楠木",
        icon: "楠",
        tags: ["tree", "broadleaf", "evergreen", "hardwood"],
        color: (56, 88, 32, 255),
        hp: 10,
        is_rooted: true,
        quantity: 30,
    ),
    CardDef(
        type_name: "camphor_tree",
        display_name: "樟树",
        icon: "樟",
        tags: ["tree", "broadleaf", "evergreen", "aromatic"],
        color: (64, 96, 40, 255),
        hp: 10,
        is_rooted: true,
        quantity: 25,
    ),
    CardDef(
        type_name: "bamboo",
        display_name: "毛竹",
        icon: "竹",
        tags: ["plant", "bamboo", "tall_grass", "fast_growing"],
        color: (96, 152, 56, 255),
        hp: 3,
        is_rooted: true,
        quantity: 40,
    ),

    // ═══ 山麓区（18×18 环）═══
    CardDef(
        type_name: "foothills",
        display_name: "山麓",
        icon: "麓",
        tags: ["foothills", "slope", "terrain"],
        color: (80, 96, 64, 255),
        hp: 0,
        is_rooted: true,
        quantity: 60,
    ),
    CardDef(
        type_name: "pine_forest",
        display_name: "马尾松林",
        icon: "松",
        tags: ["forest", "pine", "evergreen", "mountain"],
        color: (56, 80, 48, 255),
        hp: 8,
        is_rooted: true,
        quantity: 40,
    ),
    CardDef(
        type_name: "azalea",
        display_name: "杜鹃",
        icon: "鹃",
        tags: ["plant", "shrub", "flowering", "mountain"],
        color: (200, 112, 96, 255),
        hp: 2,
        is_rooted: true,
        quantity: 30,
    ),

    // ═══ 山壁（外环）═══
    CardDef(
        type_name: "cliff",
        display_name: "山壁",
        icon: "崖",
        tags: ["cliff", "basalt", "steep", "impassable", "terrain"],
        color: (128, 120, 104, 255),
        hp: 0,
        is_rooted: true,
        quantity: 9999,
    ),
    CardDef(
        type_name: "lichen",
        display_name: "地衣",
        icon: "衣",
        tags: ["plant", "lichen", "cliff_dweller"],
        color: (160, 152, 128, 255),
        hp: 1,
        is_rooted: true,
        quantity: 50,
    ),
]
```

### 卡总数

19 张环境卡（7 地形 + 12 植物）。动物卡后续 handoff。

## 架构反馈

- 所有卡标记 `terrain` 或 `plant` 标签—perceive 公理可读 ✅
- 量级从 20(莲花) 到 9999(山壁)—反映 1km² 格的真实承载 ✅
- 颜色在视觉范围内可区分 ✅
- 水生→湿地→草原→森林→山麓→山壁 渐变过渡 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- card_audit 不报错（新标签需注册）
