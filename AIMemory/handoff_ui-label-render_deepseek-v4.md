# UI 信息栏标签化 + 空格子选中 + 窝/藏有显示

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-02
**Priority**: HIGH

---

## 目标

改三件事：
1. 选中卡/格子 → 信息栏显示中文标签，零英文
2. 空格子（河流、荒地等）可点击选中
3. cell.overlay 卡（狼窝/狐窝/humus）点击时展示，窝显示内容物，灌木显示藏有生物

---

## 一、标签翻译表

card_db 原始标签和能力 → 信息栏中文。不在前端显示任何英文字符串。

### 标签

| 原始 | 中文 |
|------|------|
| being | 生物 |
| animal | 动物 |
| herbivore | 食草 |
| predator | 掠食者 |
| mesopredator | 中型捕食者 |
| carnivore | 食肉 |
| smallPrey | 小型猎物 |
| largePrey | 大型猎物 |
| smallHerbivore | 小型食草 |
| wildPrey | 野生 |
| juvenile | 幼体 |
| scavenger | 清腐 |
| cover_user | 栖灌 |
| burrower | 掘穴 |
| omnivore.small | 杂食 |
| den | 巢穴 |
| shelter | 居所 |
| animalHome | 动物居所 |
| den.candidate.fox | 可占筑窝 |
| bush | 灌木 |
| grass | 草地 |
| cover.small | 小型掩体 |
| regenerates | 再生 |
| microfauna.host | 栖虫 |
| foodSource | 食源 |
| berry.source | 产浆果 |
| plant | 植物 |
| rooted | 根植 |
| water | 水源 |
| woodSource | 林源 |
| stoneSource | 岩源 |
| fuel | 燃料 |
| material.lumber | 木材 |
| material.stone | 石料 |
| material.shard | 碎片 |
| tool | 工具 |
| weapon | 武器 |
| sharp | 锋利 |
| blunt | 钝器 |
| hard | 坚硬 |
| heat | 热源 |
| corpse | 尸体 |
| organic | 有机 |
| food.raw | 生食 |
| food.edible | 可食 |
| cooked | 熟食 |
| basic | 基础 |
| commodity | 商品 |
| currency | 货币 |
| copper | 铜质 |
| human | 人类 |
| taoyuan | 桃源 |
| observer | 观察者 |
| elder | 长者 |
| forager | 采集者 |
| youth | 青年 |
| customer | 旅人 |
| worker | 劳工 |
| actor | 玩家 |
| autonomous | 自主 |
| craftPart | 零件 |
| structure | 建筑 |
| container.water | 容器 |
| soil | 土质 |
| fertile | 沃土 |
| wood | 木质 |
| wet | 潮湿 |

### 能力

| 原始 | 中文 |
|------|------|
| capability.hunt | 捕猎 |
| capability.forage | 觅食 |
| capability.flee | 逃跑 |
| capability.reproduce | 繁殖 |
| capability.grow | 成长 |
| capability.follow | 跟随 |
| capability.be_hunted | 被捕食 |
| capability.be_cared_for | 被哺育 |
| capability.care_child | 哺幼 |
| capability.return_home | 归巢 |
| capability.use_cover | 钻丛 |
| capability.escape_small | 灵巧 |
| capability.escape_fast | 疾逃 |
| capability.escape_cover | 藏匿 |
| capability.scavenge | 清腐 |
| capability.carry | 携物 |

---

## 二、展示格式

### 规则

1. 某分区无内容 → **不显示该行**（不是横杠，不是空白行）
2. 性别用【公】【母】【无】
3. cell.overlay 卡（den/humus）点击该格时优先于地面卡展示
4. 窝内/藏有动态列表实时更新
5. 空格子可选中

### 示例

```
鹿 【公】
身份：动物 · 食草 · 大型猎物 · 野生
能力：觅食 · 疾逃 · 繁殖
状态：吃草  目标：草皮
HP 3/3
```

```
狐 【母】
身份：动物 · 食肉 · 中型捕食者 · 清腐 · 栖灌
能力：捕猎 · 疾逃 · 钻丛 · 归巢 · 哺幼 · 繁殖
状态：捕猎  目标：田鼠
HP 3/3
```

```
狼窝
身份：巢穴 · 动物居所
窝内：成年狼×2  幼狼×1  储肉 2
砸毁 0/2
```

```
灌木丛
身份：植物 · 灌木 · 小型掩体 · 栖虫 · 可占筑窝
藏有：田鼠×1
状态：再生中
```

```
河岸
身份：草地 · 水源
状态：产草恢复中
```

```
篝火
身份：热源
```

---

## 三、涉及文件

| 文件 | 改什么 |
|------|--------|
| `scripts/ui/game_ui.gd` | `render_selected()`：标签走翻译表、分区空不显示、den/窝内容物、灌木藏有 |
| `scripts/core/world_rules_ui.gd` | 格子地形标签 + overlay 优先逻辑 |
| `scripts/input/*.gd` | 空格子可选中 |

---

## 约束

- L0 不降
- 不碰管线/签名/封闭模式
- 记 fix-log
