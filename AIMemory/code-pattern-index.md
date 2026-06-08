# 代码模式索引 — 方寸商国：桃花源记

> 给 Cursor 的参考模板。实现任何新卡前，必须先 grep 最相似的已有实现，复用其 80% 结构，只写差异部分。

## 通用纪律

1. 新卡定义 → 同时更新 `card_db.gd` + `world_rules.gd`（标签/能力/域名/is_xxx 判断）
2. 新卡行为 → 先找最相似的已有 behavior.gd，复用其核心循环
3. **新卡信息栏 → 必须同步更新 `game_ui.gd` 的 `render_selected()`**，追加对应卡类型的 if 分支
4. 新卡冲击/交互 → 检查是否需要补 `interaction_manager.gd` 的 `_impact_reg`
5. 新卡生成 → `world_manager.gd` `_spawn_initial_cards` + `population_manager.gd` 生命周期

---

## 一、建筑/窝类

### 狼窝 wolfDen（母模板）

| 涉及文件 | 做什么 |
|----------|--------|
| `card_db.gd` L47 | 注册：tags=["den","shelter","animalHome"], is_rooted=true |
| `world_rules.gd` L73 | 能力：define_domain, bond_to_actor, support_reproduce, provide_service |
| `world_rules.gd` L140-141 | 域名映射：DOMAIN_WOLF_PACK |
| `world_rules.gd` L180-191 | 成员判定：is_wolf_pack_member / is_adult_wolf / is_wolf_pack_cub |
| `world_rules.gd` L417-453 | 归属：home_for_actor / is_home_for_actor / wolf_pack_homes |
| `world_rules.gd` L537-551 | 筑窝者选择：wolf_den_builder() |
| `ecosystem_manager.gd` L34-104 | 容量淘汰、筑窝判定、回窝投喂逻辑 |
| `ecosystem_manager.gd` L114-179 | 筑窝流程：叼草→寻址→spawn→绑定 homeDenId |
| `interaction_manager.gd` L116 | 冲击配方注册 |
| `interaction_manager.gd` L165-183 | 砸毁 handler：全体驱逐 → 产生 **dryGrass** |
| `game_ui.gd` L669-686 | **信息栏**：窝内狼数、容量、砸击计数 |
| `game_state.gd` L56 | WOLF_DEN_CAPACITY = 3 |

### 狐狸窝 foxDen（继承 wolfDen，差异：灌木转化）

| 与 wolfDen 相同的 | 与 wolfDen 不同的 |
|-------------------|-------------------|
| 能力四件套 | 域名：DOMAIN_FOX_FAMILY（非 WOLF_PACK） |
| homeDenId 绑定机制 | 建窝：**占灌木转化**（非叼草建窝） |
| 筑窝者选择（fox_den_builder） | 砸毁产物：**bush**（非 dryGrass） |
| 冲击破坏模式 | 容量：FOX_DEN_CAP=3 + FOX_DEN_FAMILY_CAPACITY=3 |
| 成员驱逐 | 破坏后动物：惊惧+逃离（非惊怒+寻猎） |

**已发现缺口**：狐窝 UI 缺砸击计数（wolfDen 有 `"砸击：%d / 2"`）。

---

## 二、动物类

### 狼 wolf（捕食者模板）

| 涉及文件 | 做什么 |
|----------|--------|
| `wolf_card.gd` | 子类 CardBase，有 sex/hunger/meatFedToday/homeDenId |
| `wolf_behavior.gd` L1-100 | 行为 tick：惧火、猎食、回窝、叼肉、筑巢 |
| `world_rules.gd` | is_adult_wolf / wolfCub 判定、捕猎评分 |
| `ecosystem_manager.gd` | 筑窝流程、回窝喂养、should_wolf_leave |
| `population_manager.gd` | 狼群迁出/召回（步骤1已禁）、性别补齐（步骤1已禁） |
| `game_ui.gd` L659-668 | **信息栏**：性别、成长天/断粮天/今日肉量 |

### 狐狸 fox（继承 wolf 捕食者模式，差异：多策略+清道夫）

| 与 wolf 相同的 | 与 wolf 不同的 |
|-------------------|-------------------|
| homeDenId 绑定 | 多策略捕食：田鼠>兔>偷尸体肉 |
| 幼兽跟随/成长 | 小家庭单位（不合并大群） |
| 惧火/躲人 | 草丛/灌木中加速+隐藏 |
| 繁殖条件门控 | 清道夫：偷肉回窝（1~2/日/窝） |
| | 避让狼 |

### 羊 sheep → 鹿 deer → fieldMouse

| 模板 | 食性 | 繁殖 | 特殊 |
|------|------|------|------|
| sheep（母模板） | 吃草（game_delta） | 公母+草≥3+无狼 | 逃跑、跟随成年羊 |
| deer（继承羊） | 吃草（同羊） | 公母+草≥4+无狼5格+deerFawn上限1 | 惧火/躲人优先、terrain择地 |
| fieldMouse（继承兔） | **不吃草**，吃灌木蚯蚓/甲虫/浆果 | 绑定灌木、上限成8/幼2 | 草丛/灌木中穿梭+隐藏 |

### 兔子 rabbit（小型猎物模板）

| 涉及文件 | 做什么 |
|----------|--------|
| `rabbit_behavior.gd` | 吃草（real_delta计时）、逃跑、隐藏草丛 |
| `game_state.gd` L119 | RABBIT_EAT_TICK_SECONDS（步骤1改为45s） |

**步骤2兔食性调整**：优先灌木浆果/微型生物，活草降至~4块/日。

---

## 三、植物/资源类

### 草 grass → 灌木 bush

| 模板 | 再生 | 被消耗 | 额外 |
|------|------|--------|------|
| grass | 河岸恢复（6s间隔+上限18） | 羊/鹿/兔吃 | 无 |
| bush | 自身再生、浆果计时 | 田鼠吃蚯蚓/甲虫，兔吃浆果 | 内部状态 worm/beetle、掩体加速、狐窝前身 |

---

## 四、必须检查的落点清单（每张新卡）

| 文件 | 必须检查 |
|------|---------|
| `card_db.gd` | 注册卡定义（tags/capabilities/is_rooted/display_name） |
| `world_rules.gd` | 域名映射、is_xxx 判断、capability 登记 |
| `world_manager.gd` `_spawn_initial_cards` | 初始生成 |
| `population_manager.gd` | 生命周期（繁殖/成长/上限/死亡） |
| `ecosystem_manager.gd` | 行为注册（EcosystemTickRegistry） |
| `interaction_manager.gd` | 冲击/交互配方 |
| **`game_ui.gd` `render_selected()`** | **信息栏显示** |
| `game_state.gd` | 常量 |
| 对应 behavior.gd | 行为逻辑 |
| CardRuleAudit | 审计维度归类 |
| L0 单测 | 新增断言 |
