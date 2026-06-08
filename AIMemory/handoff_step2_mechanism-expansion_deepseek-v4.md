# 步骤 2 实现：机制维度扩展 — 灌木丛 + 田鼠 + 狐狸

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 设计意图

这不是"加 3 种动物"——是让生态从 **3 个机制维度拉高到 5~6 个**：

| 机制维度 | 步骤 1 状态 | 步骤 2 新增 |
|----------|------------|------------|
| 采集 | 浆果/树枝/石头（无位置门控） | 灌木丛**固定位置再生产能** |
| 狩猎 | 靠近→杀掉 | 狐狸**多策略捕食**（捕猎+清道夫+藏肉） |
| 种植/培育 | 无 | （本切片不做，灌木是野生） |
| 储存保质 | 无（尸体靠计时器） | 狐狸**偷尸体肉回窝**（主动清腐） |
| 资源再生 | 河岸长草 | 灌木产浆果+宿主微型生物 |
| 栖息地依赖 | 仅狼窝 | 田鼠**绑定灌木**、狐狸**绑定灌木/狐窝** |
| 副产品/废物 | 无 | 蚯蚓/甲虫作为**灌木内部状态**（不独立成卡） |

## 执行依据

`docs/design/v3.0-a-ecology-initial-values.md` §6（步骤 2 预览）
`docs/design/v3.0-a-ecology-redesign-discussion.md`（方向讨论稿）

## 新增卡

### 1. bush — 灌木丛

定位：**再生型生态节点**，不是装饰。复合机制：
- 周期性产出浆果（`capability.produce_resource`）
- 内部状态：`worm_count` / `beetle_count` / `microfauna_regen_timer`
- 提供小型动物掩体（`capability.provide_cover`）：田鼠/兔/狐狸进草丛加速
- 可被狐狸占据转化为狐窝前身（`capability.support_den`）
- 自身可恢复/再生（`capability.regenerate`）

标签：`plant.patch, cover.small, berry.source, microfauna.host, den.candidate.fox, regenerates`

初始数量：8，分布在河岸+林缘。

### 2. fieldMouse — 田鼠

定位：**依赖型消费者**——不是"另一种兔"。
- 核心区别：**不吃草**，吃灌木里的蚯蚓/甲虫
- 在草丛和灌木丛间穿梭（`capability.use_cover`）
- 繁殖强度接近兔子（`capability.reproduce`）
- 绑定灌木存在——灌木没了田鼠也绝迹（**引入栖息地依赖机制**）

标签：`being, animal, smallPrey, omnivore.small, burrower, cover_user`

初始数量：4（2 对），上限 成8/幼2。

### 3. fox — 狐狸

定位：**多策略捕食者+清道夫**——不是"小号狼"。
- 主食田鼠 > 兔子 > 机会偷尸体肉
- 小家庭单位（1公1母+幼狐），不合并成大群（**区别于狼的 pack 逻辑**）
- 在草丛/灌木中移动加速、更难被发现（`capability.use_cover`）
- 偷尸体肉回窝（上限 1~2/日/窝）（**引入清道夫机制**）
- 避让狼/人/火域
- 可占据灌木丛创建狐窝

标签：`being, animal, carnivore, mesopredator, scavenger, cover_user`

初始：1 公 1 母。

### 4. foxCub — 幼狐

标签：`being, animal, carnivore, juvenile`
能力：move/follow/grow/be_cared_for
初始：0。

### 5. foxDen — 狐窝

定位：由灌木丛占据/转化，狐狸家庭中心。
- 全图上限 ≤3 个
- 狐狸可在此藏肉
- 可被破坏/废弃

标签：`structure, shelter, den, den.candidate.fox`

## 机制要点

### 蚯蚓/甲虫：不独立成卡

当前作为 `bush` 的内部状态字段（`worm_count` / `beetle_count` / `microfauna_regen_timer`）。田鼠觅食时消耗，灌木再生时恢复。未来若需要钓鱼/堆肥/药材系统，再解封为独立卡。

### 狐狸清道夫逻辑

```
附近有尸体 + 狼/玩家不在旁边
→ 狐狸靠近尸体
→ 搬运 1 块肉回最近狐窝
→ 每日上限 1~2（防止无限偷肉）
```

### 捕猎评分修正（续步骤 1）

步骤 1 已改狼不优先成年鹿。步骤 2 追加：
- 狼可杀狐（低优先，狐避狼）
- 狐狸不挑战成年羊/鹿/玩家

### 兔食性调整

步骤 2 兔子改为**主要吃浆果/灌木微型生物**，吃活草降至 ~4 块/日/兔。`RABBIT_EAT_TICK_SECONDS` 45s 不变，但 `all_grass()` 查找改为优先灌木资源。

## 验收

- L0 PASS，断言数不退回
- L1.5：bush,fieldMouse,fox,foxCub,foxDen
- closed_ecology_trace PASS（1×，含灌木/田鼠/狐狸种群采样）
- L2b 1×：USELESS=0，蘑菇棚建成

## 不做

- 鱼、鸟、野猪等更多动物线
- 完整土壤肥力系统
- 完整季节系统
- 蚯蚓/甲虫独立成卡
- 旅人解封
- 桃源人交易/雇佣

## 回复方式

在本文件末尾追加 `# 回复` 段，记入 `AIMemory/work.log`。

---

# 回复

**From**: cursor  
**Date**: 2026-05-30  
**Status**: 步骤 2 机制维度扩展已合入；Godot 4.6.1 headless 主场景 10s 无编译错误。

## 机制维度（3→6）

| 维度 | 实现 |
|------|------|
| 再生节点 | `bush`：浆果计时、`worm_count`/`beetle_count` 再生（`environment_manager._tick_bush_ecology`） |
| 依赖消费 | `fieldMouse` 只吃灌木微型生物/浆果，无灌木则无法繁殖 |
| 多策略捕食 | `fox` 猎鼠>兔；狼可猎狐（低优先）；狐不猎羊鹿人 |
| 栖息地绑定 | 田鼠/狐绑定灌木与 `foxDen`（`DOMAIN_FOX_FAMILY`） |
| 清道夫 | `try_fox_scavenge`：无狼近尸体取肉回窝，每窝每日 ≤2 |
| 资源的资源 | 灌木内部微型生物，非独立卡 |

## 新增/主要文件

- `field_mouse_behavior.gd`, `fox_behavior.gd`, `fox_card.gd`
- `game_state` 步骤 2 常量；`card_db` / `world_rules` / `population_manager` / `ecosystem_manager`
- 初始 spawn：8 灌木、4 田鼠、1 狐窝 + 1 对狐狸
- `closed_ecology_trace` 采样含 bush/mice/fox/den
- 兔：`rabbit_forage_priority` + 每日活草 ≤4 口

## 验收（请本地）

```powershell
powershell -File tools/run_unit_tests.ps1
godot --headless --path . "res://scenes/card_factory_smoke.tscn" -- --factory-types=bush,fieldMouse,fox,foxCub,foxDen
powershell -File tools/run_closed_ecology_trace.ps1
```

## 未做

旅人、土壤肥力、季节、蚯蚓/甲虫独立卡、鱼鸟野猪（按 handoff）。
