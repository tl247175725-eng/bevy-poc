# 方寸商国：桃花源记 — 设计全貌

> 最后更新：2026-06-06（Bevy M3 UI 交互 + 生态二期全量落地）

---

## 零、技术栈（Bevy 迁移现状）

| 层 | 实现 |
|----|------|
| 运行时 | Rust + Bevy 0.14 ECS |
| 卡定义 | `assets/card_defs.ron`（85 卡，由 `tools/gen_card_defs.py` 生成） |
| 规则引擎 | `src/world_rules.rs` + `src/capabilities.rs` |
| 生态 tick | `src/systems/main_tick.rs` 调度 9 个子系统 |
| 格网渲染 | `src/grid_render.rs`（36×24 批次 mesh + 卡牌 sprite） |
| UI 交互 | `src/ui_interaction.rs` + `src/panel_ui.rs` + `src/selection_info.rs` |
| 中文标签 | `src/tag_zh.rs`（`TAG_ZH` / `CAP_ZH`） |
| 验收 | `cargo test`（121 断言）/ `cargo run --release` |

---

## 一、这个世界是什么

一片封闭的河谷。西侧暗河源头水潭，水从高处经河岸流下，汇入东侧暗河出口。万物皆卡——每张卡有标签，标签决定它在这个世界里的身份、能力和与其他卡的关系。

不是"狼会猎鹿"。是有 `hunt` 能力的卡对有 `wildPrey` 标签的卡通过 WorldRules 产生捕猎关系。行为涌现，不脚本。

---

## 二、生态全貌（当前已实现 — Bevy `WorldState` + `main_tick`）

### 能量入口

| 来源 | 卡 | 机制 |
|------|-----|------|
| 草皮 | grass | 河岸再生，被所有食草动物吃 |
| 树林 | tree | 产树枝（natural_drop），容纳栎树/松树/鸟巢，斧砍出木头 |
| 水藻 | algae | 水潭自然恢复，被水虫和贝吃 |
| 灌木 | bush | 产浆果、栖虫，藏田鼠，狐狸可占为窝 |

### 消费层

| 物种 | 卡 | 标签特征 |
|------|-----|---------|
| 羊 | sheep | `flocking` 群居，孤羊不繁殖 |
| 鹿 | deer | `wildPrey` 机警，更大逃跑半径 |
| 水牛 | waterBuffalo | `tough` 成体不可猎 |
| 兔 | rabbit | `prolific` 高产，每 3s 一胎 3 只 |
| 田鼠 | fieldMouse | `burrower` 掘穴，灌丛依赖 |
| 雉鸡 | pheasant | `flocking` 地栖鸟，吃虫种子 |
| 竹鼠 | bambooRat | `burrower` 林地掘根，无灌丛依赖 |
| 鱼 | fish | `aquatic` 仅水格移动，经暗河迁入 |
| 贝 | shellfish | `sessile` 固着浅水，滤食稳藻 |
| 人类 | taoyuanElder/Forager/Youth | 狩猎采集者，鱼+猎物+采集 60/30/10 |

### 捕食层

| 物种 | 卡 | 标签特征 |
|------|-----|---------|
| 狼 | wolf | `pack_hunter` 群猎，单狼只猎小猎物 |
| 狐 | fox | `mesopredator` 多策略，限杀 2 只/天，兼清腐 |

### 分解层

| 过程 | 机制 |
|------|------|
| 尸体自然腐解 | → humus → 草加速恢复 |
| 狐清腐取肉 | scavenger 能力 |
| 陆虫趋尸 | landBug 被尸体吸引 → 加速分解 → 喂鼠鸟 |
| 生肉腐坏 | `perishable` 标签，90 tick 自毁 |

### 容纳宿主（溢出乐趣）

| 宿主 | 容纳内容 | 取法一 | 取法二 | 取法三 |
|------|---------|--------|--------|--------|
| 树林 | 栎树 oak / 松树 pine / 鸟巢 birdNest | 橡子/松塔自然掉落邻格 | 人摘橡子/松塔 | 斧砍出木头 |
| 水潭 | 藻/鱼/贝/水虫/菱角 waterCaltrop / 莲 lotus | 矛刺鱼 | 人采菱角/莲蓬 | 摸贝 |
| 林下 | 野山药 wildYam（地下） | 人挖山药 | 山药藤蔓延→邻格新生 | 竹鼠拱出山药 |
| 岩壁 | —（地形） | 锤敲→碎石 | 自然风化→碎石掉落 | 人捡 |

---

## 三、当前已实现的完整卡表

### 大型动物
sheep / lamb / deer / deerFawn / wolf / wolfCub / fox / foxCub / waterBuffalo / waterBuffaloCalf

### 小型动物
rabbit / fieldMouse / fieldMousePup / pheasant / pheasantChick / bambooRat

### 水生
algae / waterBug / fish / shellfish

### 树内容纳卡
oak（栎树，产橡子）/ pine（松树，产松塔）/ birdNest（鸟巢）

### 水潭容纳卡
waterCaltrop（菱角）/ lotus（莲）

### 地下容纳卡
wildYam（野山药）

### 掉落物
acorn（橡子）/ pineCone（松塔）/ caltropFruit（菱角果）/ lotusSeed（莲子）/ wildYamRoot（山药，易腐）

### 陆虫
landBug（趋尸，加速分解，喂鼠鸟）

### 建筑与居所
wolfDen / foxDen / fire（篝火）/ hut（草棚）/ table（桌子，冻结中）

### 植物
grass（活草）/ dryGrass（干草）/ bush（灌木丛）

### 资源与物品
stone / shard / tri / square / knife / spear / wood / twig / woodStruct / axe / hoe / hammer / bucket / waterbucket / halfbucket / berry / mushroom / wetWood / mushroomWood / charcoal / grassRope

### 肉类（全部 perishable）
sheepMeat / rabbitMeat / deerMeat / wolfMeat / humanMeat / fishMeat / cookmeat

### 尸体
sheepCorpse / deerCorpse / wolfCorpse / playerCorpse

### 人类
player（玩家）/ taoyuanElder / taoyuanForager / taoyuanYouth

### 冻结卡（经营层，CardDB 保留但禁用 spawn）
traveler / mushroomFarmer / mushroomGreenhouse / coin / copperBlock / copperCraft

---

## 四、食物网

```
草皮 ──→ 羊、鹿、水牛、兔 ──→ 狼
灌木 ──→ 兔、田鼠、雉鸡 ──→ 狐
水藻 ──→ 水虫 ──→ 鱼 ──→ 人、鸟
橡子/松塔 ──→ 兔、鼠、雉鸡、人
尸体 ──→ 狐（清腐）、landBug（趋尸）
landBug ──→ 田鼠、雉鸡、竹鼠
尸体腐解 ──→ humus ──→ 草加速恢复
```

---

## 五、标签驱动行为管线

每张自主卡在每个 tick 被 `main_tick` 唤醒。子系统读能力标签分发：

- `tick_herbivore.rs` — 羊/鹿/兔/水牛/雉鸡（`herbivore_grazer_profile`）
- `tick_predator.rs` — 狼/狐
- `tick_cover_forager.rs` — 田鼠/竹鼠
- `tick_aquatic.rs` — 鱼/虫/贝/藻

一张新卡只要在 `card_defs.ron` 写好标签和能力，自动匹配正确管线——不改分发代码。

---

## 六、玩家卡五层大脑

1. **可供性层**（Gibson affordance）：世界提供什么——`AFFORDANCE_TABLE` 注册表
2. **需求层**（SDT）：自主/胜任/关联三维——`NEED_TAG_RULES` 标签驱动
3. **意向层**（BDI）：长期目标 vs 当前意图——`LONG_TERM_TABLE`
4. **行为层**：五层整合 + 去魔法数字
5. **运行时标签层**：`hungry`/`has_weapon`/`predator_nearby` 等运行时标签

---

## 七、地形

- 36×24 河谷。西侧暗河源头水潭，东侧暗河出口
- 水潭（pool）— 水生卡隐藏其中，点选显示容纳列表；空桶可取水
- 河岸（bank）— 草皮再生
- 河流（river）— 水流路径
- 岩壁（cliff）— 山卡邻格，风化掉碎石
- 每格有海拔数值

---

## 八、三阶段愿景

### 阶段一：生态自洽（当前）

封闭河谷内完整的生产者→消费者→捕食者→分解者循环。50 人狩猎采集部落可自持。玩家作为外来者，先学会在这片土地上生存。

### 阶段二：社会松动（v3.x 规划中）

桃源人是一个有完整社会规则和文化的 50 人部落。他们有守旧派（长者，坚守边界），有务实派（采集者，看实际价值），有年轻派（青年，对差异好奇）。

玩家卡曾是商人——百科全书般的精英阶层。他逐渐用商品和服务软化桃源人的铁律。铁律是：**任何人不得离开桃花源。** 外来者一旦误入，困于此地。离开就是死亡。但只要不试图离开、不冒犯规矩，桃源人基本不干涉外来者的生活。

偶尔有更多外人进入。他们比桃源人生存能力更差，比玩家更偏单项技能。他们可能和玩家抱团，也可能各自为营。

玩家在自己的一块区域逐渐打造商业雏形。最终，桃源人同意与外界通商——但在他们的规则之下。这是第一次复杂度跃迁：更多商品、外来物种、外来文化涌入。

### 阶段三：商业枢纽（v4.x 远期）

桃源人彻底认同玩家的带领。规则松动到只剩文化特征。桃花源成为巨大的商业枢纽——前端销售丰富商品、提供海量服务；后端生产和管理资源。阶级涌现。游戏达到巅峰形态。

---

## 九、设计铁律

1. **标签即因果**：每行决策从标签推导，零魔法数字
2. **WorldRules 是唯一中介**：卡不直接交互，所有关系由规则引擎计算
3. **硬写代码 = 项目完蛋**：新机制通过标签/能力/维度表达，新增可供性 = 加注册表条目
4. **一口一层**：每层一个可玩闭环，能停手
5. **溢出乐趣**：同一产出多种取法——不是资源点，是"这里有事在发生"
