# 《方寸商国》视觉规范 — 从 Godot 源码提取

**用途**：Bevy 实现的唯一视觉验收标准。每一条都来自 Godot 源码。
**提取来源**：`card_base.gd`、`game_ui.gd`、`selection_info_panel.gd`、`terrain_visual_palette.gd`、`world_manager.gd`

---

## 一、窗口布局

来源：`main.tscn` + `game_ui.gd _build_panel()`

```
┌──────────────────────────────────────┬────────────┐
│                                      │  360px     │
│         世界区域                      │  右侧面板   │
│         (窗口宽 - 360, 窗口高)        │  全高      │
│                                      │  背景色     │
│         #efe2c8 暖米底色              │  #fff7e6   │
│                                      │  面板色     │
│         #f5ead6 盒子色               │           │
│                                      │  左边框     │
│                                      │  3px       │
│                                      │  #6b5337   │
└──────────────────────────────────────┴────────────┘
```

**关键常量**：
- CELL_SIZE = 56（来源 `GameState.CELL_SIZE`）
- CARD_SIZE = 50（来源 `CardBase.CARD_SIZE`）
- 右侧面板宽 = 360px 固定（来源 `main.tscn` `offset_right = -360`）
- 世界背景色 = #efe2c8（来源 `card_base.gd` 默认 bg + 面板 `BOX_BG` 推导）
- 面板背景色 = #fff7e6（来源 `game_ui.gd PANEL_BG`）
- 面板盒子色 = #f5ead6（来源 `game_ui.gd BOX_BG`）

---

## 二、卡牌视觉

来源：`card_base.gd _setup_visual()` + `_card_style()`

### 每张卡的构成

```
┌───── 选中框（仅选中时）──────┐
│  #16813a 绿色 3px 边框       │
│  (CARD_SIZE + 8, CARD_SIZE + 8)│
│  ┌──── 卡牌边框 ───────┐     │
│  │ 1px 实色边框          │     │
│  │ ┌── 卡牌背景 ──┐     │     │
│  │ │  (50×50)    │     │     │
│  │ │             │     │     │
│  │ │   【图标】    │     │     │
│  │ │   22px 居中   │     │     │
│  │ │             │     │     │
│  │ │   【名称】    │     │     │
│  │ │   12px 底部   │     │     │
│  │ └─────────────┘     │     │
│  └─────────────────────┘     │
└──────────────────────────────┘
```

- 边框 = `ColorRect` 50+4=54×54，偏移 -2,-2，色值为 card_style 的 border 色
- 背景 = `ColorRect` 50×50，色值为 card_style 的 bg 色
- 图标 = `Label` 50×28，偏移 (0,4)，字号 22，居中
- 名称 = `Label` 50×16，偏移 (0, CARD_SIZE-18)，字号 12，居中
- 文字色 = `TEXT_DARK` #2c2117（card_style 可覆盖）
- 选中框 = `ReferenceRect` 58×58，偏移 -4,-4，绿色 #16813a，3px 宽
- Qty badge = `Label` 偏移 (44, -10)，字号 10，仅 qty > 1 时显示

### 每张卡的颜色（来源 `_card_style()`）

```
草皮 grass          bg:#c9f28b  border:#5d9d36
干草 dryGrass       bg:#ead179  border:#aa7d22
树林 tree           bg:#2f7f43  border:#18552b  text:#fffdf5
灌木 bush           bg:#3f8f55  border:#1f5f36  text:#fffdf5
水藻 algae          bg:#4a9e6a  border:#2a6e42  text:#fffdf5
腐殖土 humus        bg:#5a4030  border:#3a2818  text:#fffdf5

石头 stone          bg:#d1d3d4  border:#777b80
碎石 shard/tri/square  bg:#c5c8ca  border:#6f7478
木头 wood           bg:#c58b4b  border:#7b4d25
树枝 twig           bg:#d0a15d  border:#8b5f2c
木构件 woodStruct   bg:#b47b45  border:#6c4323
草棚 hut            bg:#d8b76a  border:#8a6226
篝火 fire           bg:#ffb156  border:#c53a1c
山 mountain         bg:#9e9fa1  border:#555759  text:#fffdf5

羊 sheep            bg:#f1dfbd  border:#9b7351
羔 lamb             bg:#fff1cb  border:#b9905a
兔 rabbit           bg:#e9cfae  border:#ad7e55
雉鸡 pheasant       bg:#c8a858  border:#8a6020
雉鸡雏 pheasantChick bg:#e8d0a0 border:#a88040
竹鼠 bambooRat      bg:#9a8a70  border:#5a4a38
鹿 deer             bg:#e8dcc8  border:#8b6914
幼鹿 deerFawn       bg:#f5ecd8  border:#a8844a
田鼠 fieldMouse     bg:#c9b8a0  border:#6a5040
幼鼠 fieldMousePup  bg:#e0d4c4  border:#8a7060
狐 fox              bg:#e8a060  border:#a85820  text:#fffdf5
幼狐 foxCub         bg:#f0c090  border:#c07830
水牛 waterBuffalo   bg:#6a5a50  border:#3a2a20  text:#fffdf5
水牛犊 waterBuffaloCalf bg:#9a8a7a border:#5a4a40

狼 wolf             bg:#7b403d  border:#5c1f25  text:#fffdf5
幼狼 wolfCub        bg:#98605a  border:#6a2f31  text:#fffdf5
狐窝 foxDen         bg:#8a6040  border:#4a3020
狼窝 wolfDen        bg:#6f5140  border:#3d2c25  text:#fffdf5

水虫 waterBug       bg:#8ab84a  border:#5a8828
鱼 fish             bg:#8ed0e8  border:#3a88a8  text:#2c2117
贝 shellfish        bg:#e8c8a0  border:#a88860

羊肉 sheepMeat      bg:#e68475  border:#a43d36
兔肉 rabbitMeat     bg:#d9856a  border:#9a3a28
鹿肉 deerMeat       bg:#d47868  border:#9a4030
鱼肉 fishMeat       bg:#c8a0a0  border:#886868
浆果 berry          bg:#d84d78  border:#8d2645  text:#fffdf5
蘑菇 mushroom       bg:#f6d6db  border:#b75b73
熟肉 cookmeat       bg:#f2a66d  border:#a75227

橡子 acorn          bg:#c8a060  border:#8a6030
松塔 pineCone       bg:#a88858  border:#685028
菱角果 caltropFruit bg:#8ab878  border:#4a7848
莲子 lotusSeed      bg:#f0d8b0  border:#c0a070
山药根 wildYamRoot  bg:#d8b878  border:#a88848
陆虫 landBug        bg:#8a8a48  border:#5a5a28

尸体 sheepCorpse    bg:#8f6c67  border:#563938  text:#fffdf5
鹿尸 deerCorpse     bg:#7a6358  border:#4a3828  text:#fffdf5
狼尸 wolfCorpse     bg:#5c3830  border:#3a1e18  text:#fffdf5

水桶 bucket         bg:#d8b76a  border:#7b5b2d
水桶满 waterbucket  bg:#8fd0f5  border:#2879a9
长矛 spear          bg:#c8a870  border:#7a5225
小刀 knife          bg:#e7e9ec  border:#59616a
斧头 axe            bg:#e0d0ad  border:#6c4c2c
锤子 hammer         bg:#d5d9dd  border:#5e6670

玩家 player         bg:#cfe7ff  border:#3571a8
旅人 traveler       bg:#e7dcff  border:#7152a6
桃源长者 taoyuanElder   bg:#e8e0d0  border:#6a5a4a
桃源采集者 taoyuanForager bg:#dce8d0  border:#5a6a48
桃源青年 taoyuanYouth   bg:#e0e8f0  border:#4a6080

栎树 oak            bg:#6a8a48  border:#3a5a28  text:#fffdf5
松树 pine           bg:#4a7a58  border:#2a5038  text:#fffdf5
菱角 waterCaltrop   bg:#7ab878  border:#4a8848  text:#fffdf5
莲 lotus            bg:#f0b8d8  border:#c878a8  text:#fffdf5
鸟巢 birdNest       bg:#b89868  border:#786040  text:#fffdf5
野山药 wildYam      bg:#9a7848  border:#6a5030  text:#fffdf5

默认 bg:#fffdf5 border:#6b5337
```

---

## 三、地形颜色

来源：`terrain_visual_palette.gd`

```
水潭 pool（中心）     #1a3040 深蓝
水潭 pool（1环）      #2a6a82
水潭 pool（2环）      #3a8aa4
水潭 pool（3环）      #4aa8c4
水潭 pool（4环外）    #6ec0dc
河沟 river            #7eb8c8
浅滩 ford             #9ec8d4
湿地 bank/wetland     #a8c9a0
河岸草 riparian       #c5dbb8
荒地 land              #ead7ab
边缘 edge              #26231e
暗河源头 dark_river   #1a3040
```

---

## 四、右侧面板内容（自上而下）

来源：`game_ui.gd _build_panel()`

### 4.1 标题
"方寸商国 · MVP v2.24"，深色文字 `TEXT_DARK` #322318

### 4.2 规则说明（三条米色盒子）
1. "本轮目标：夜归机制、营地堆叠上限倒塌、逃跑受击2次反击、营地食物整理。"
2. "左键：物理拖拽，碰撞砸击；松手后停在拖到的位置。"
3. "右键：幽灵叠放 / 关系变化。"

每条：`BOX_BG` #f5ead6 底色，`TEXT_DARK` 文字，12-13px 字号

### 4.3 一键重置按钮
"一键重置" 文字按钮，点击触发 reset

### 4.4 倍速按钮组
横向排列：1× 1.5× 2× 2.5× 3× 暂停
- 当前激活项绿色高亮
- 默认 1× 选中
- 点击切换倍速，写日志

### 4.5 时间与天气
分割标题"时间与天气"，下方一个米色盒子内容：
```
游戏时间：00:04
时间比例：现实 1 秒 = 游戏 1 分钟
天气：晴天
距下雨：5:55
河岸压力：0/100
```
来源：`sim_clock.rs` 的 `_update_world_state_label()`

### 4.6 当前选中
分割标题"当前选中"，下方米色盒子显示选中卡详情：
- 标签列表（中文）
- 能力列表（中文）
- 状态
- HP
- 性别
- 容纳列表（如树内容纳、水潭容纳）
来源：`selection_info_panel.gd build_card_lines()`

### 4.7 微缩地图
来源：`minimap_panel.gd`
小窗口显示整个地图边界，红点标记位置

### 4.8 日志流
来源：`game_ui.gd`
- 滚动文本，最新在上
- 最多 80 条
- 字号 12px
- 颜色 `TEXT_DARK`
- 如"世界初始化完成，卡牌数：123"

### 4.9 试玩清单（底部）
测试用 checklist

---

## 五、初始生成卡牌（~123 张）

来源：`world_manager.gd _spawn_initial_cards()` + `_spawn_step2_ecology()`

```
玩家：1
石头：2
树枝：1
木头：2
树林：2（含栎树/松树容纳、鸟巢）
桶：1
山：2
草皮：~25（沿河岸）
羊：5（公2母3）
狼：2（公1母1）
兔：6（公3母3）
鹿：5（公2母3）
水牛：3（公1母2）
桃源人：3（长者/采集者/青年）
灌木：8
田鼠：8（+幼鼠4）
雉鸡：5
竹鼠：4
狐：2
水藻：若干
水虫：若干
鱼：若干
贝：若干
菱角/莲：若干
野山药：3-4
鸟巢：1-2
```

---

## 六、交互行为

来源：`drag_manager.gd` + `card_base.gd`

### 左键
- 物理拖拽，有碰撞
- 砸击：两次有效接触触发结果（第1次红色数字+抖动，第2次触发结果）
- 松手后停在最终位置

### 右键
- 幽灵叠放 / 关系变化
- 幽灵移动，无碰撞

### 选中
- 左键点击卡 → 选中 → 绿色边框出现
- 点空格 → 取消选中

### 堆叠
- 同格多卡 → 纵向偏移堆叠（Godot 通过 `position` 偏移实现）
- qty > 1 → 显示数量角标
