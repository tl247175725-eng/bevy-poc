# 设计理念备忘

> 给 DeepSeek 自己用的。每次写 handoff 时对照检查，每次收到用户反馈后更新。

## 核心理念

1. **万物皆卡牌** — 新卡沿固定管线报到：CardDB → WorldRules → behavior → ecosystem → population → game_ui。缺一站就缺一个功能。

2. **标签定身份，能力定行为，规则定交互** — 不是"狼猎鹿"，是"有 hunt 能力的卡对有 wildPrey 标签的卡做捕猎"。规则通用，标签可组合。

3. **新增是机制增量，不是换皮** — 每张新卡必须改变至少一个循环。肉更厚/跑更快不算。

4. **同类同构** — 窝都走"定义域→绑定成员→可建可毁→砸毁还原"。动物都走"觅食→吃饱→繁殖→幼体→天敌→死亡"。新卡找到最相似的已有卡，复用 80% 结构。

5. **世界先行于商业** — 先让生态自洽、桃源社会自洽，玩家不碰它也在转。商业是往齿轮组里插楔子。

## handoff 纪律

- [ ] 每张新卡都画完整管线：哪个文件管定义、哪个管行为、哪个管生命周期、哪个管 UI 展示
- [ ] 明确标注"参考 XX 已有实现"，让 Cursor 先 grep 再写
- [ ] 明确写"同步更新 game_ui.gd 信息栏"——这是最容易漏的一站
- [ ] 同类卡的新增属性（sex/容量/砸击），必须检查同类已有卡的信息栏分支

## 已知教训

- **2026-05-30**：foxDen 缺砸击计数 → 根因：handoff 没写"参照 wolfDen 的 UI 分支"。教训：建筑类卡必须在 handoff 里写明 UI 信息栏的参照卡。
- **2026-05-30**：deer/fox 性别不显示 → 根因：game_ui 是硬编码 if-else 链，不是声明式渲染。handoff 必须显式提醒。

## 当前管线全貌

```
card_db.gd          注册定义（标签/能力/display_name）
world_rules.gd      登记域/成员判定/行为 key
behavior/*.gd       行为 tick
ecosystem_manager   注册 tick、生态逻辑
population_manager  生命周期（繁殖/成长/上限）
interaction_manager 冲击/转化配方
game_ui.gd          信息栏 render_selected()
world_manager.gd    初始 spawn _spawn_initial_cards
game_state.gd       全局常量
```
