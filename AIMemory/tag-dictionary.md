# 标签字典

> 单一真源。新增标签必须先注册于此。禁止使用未列入的标签。

## 注册规则

- **格式**：`标签名(参数)`
- **读者**：标注哪个系统读取
- **示例**：每条至少一个实际用例

---

## 1. 身份标签

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `type_name` | CardDef 字段 | 全局 | 给人看的名，不参与逻辑 |
| `being` | flag | profile, tick_reactive | 是生物（有感知默认范围） |
| `player` | flag | profile | 玩家卡标记 |
| `animal` | flag | 分类用 | 动物 |
| `corpse` | flag | compose, tick_environment | 是尸体，不占格 |

## 2. 体型与物理

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `size:N` | N=整数 | compose | 抽象体积。缺省 1 |
| `body.tiny` | flag | parse_size | → size=1 |
| `body.small` | flag | parse_size | → size=1 |
| `body.medium` | flag | parse_size | → size=1 |
| `body.large` | flag | parse_size | → size=3 |
| `height:flat` | flag | compose | 不占格 |
| `height:low` | flag | compose | 不占格 |
| `height:medium` | flag | compose | 占格，可被挤 |
| `height:high` | flag | compose | 占格，不可挤 |
| `weight:feather` | flag | smash | 砸伤害 0 |
| `weight:light` | flag | smash | 砸伤害 1 |
| `weight:medium` | flag | smash | 砸伤害 2 |
| `weight:heavy` | flag | smash | 砸伤害 3，不可搬运 |
| `weight:immovable` | flag | smash | 不可搬运 |

## 3. 介质与移动

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `medium:NAME` | NAME=land/water/air | traverse | 原生介质 |
| `bridge:FROM->TO` | FROM,TO=介质名 | traverse | 跨介质通行证 |
| `bridge:omnimedium` | flag | traverse | 全介质通行 |
| `capability.incorporeal` | flag | compose | 不占物理承载 |
| `capability.move` | flag | is_autonomous | 能移动 |
| `move_speed:N` | N=秒数 | move_animation | 每格动画时长，缺省 0.25 |

## 4. 感知

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `perception:CH(r=N)` | CH=visual/olfactory/auditory, N=范围 | perceive | 感知通道 |
| `perception:keen_eyed(m=F)` | F=乘数 | perceive | 感知修正 |
| `visibility:tiny` | flag | perceive | 难被感知 |
| `visibility:transparent` | flag | perceive | 不可见 |
| `bridge:perceive->MEDIUM` | flag | perceive | 跨介质感知通行证 |

## 5. 驱动力

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `drive:seek(target=TAG, range=N, priority=N)` | TAG=目标标签 | active_drives | 觅食/寻物 |
| `drive:flee(target=TAG, range=N, priority=N)` | TAG=威胁标签 | active_drives | 逃离 |
| `drive:flock(target=TAG, range=N, priority=N)` | TAG=同类标签 | active_drives | 群聚 |
| `drive:hide(target=TAG, priority=N)` | TAG=覆盖物标签 | active_drives | 躲藏 |
| `drive:return_den(priority=N)` | flag | active_drives | 回巢 |
| `drive:scavenge(target=TAG, range=N, priority=N)` | TAG=尸体标签 | active_drives | 食腐 |

## 6. 冲刺

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `sprint:slow` | flag | execute_drive | 0.18s/格 |
| `sprint:normal` | flag | execute_drive | 0.12s/格 |
| `sprint:fast` | flag | execute_drive | 0.08s/格 |
| `sprint:burst` | flag | execute_drive | 0.05s/格 |
| `sprint:endurance` | flag | execute_drive | 略快+最长持续 |

## 7. 人格（需求/特质/价值观）

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `trait:NAME(N)` | N=0-100 | tick_reactive | 人格特质 |
| `value:NAME` | flag | tick_reactive | 价值观 |
| `need:NAME(rate=N,curve=CURVE)` | N=速率, CURVE=steep/flat/sharp | decision_engine | 需求 |

## 8. 生态

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `foodSource` | flag | drive:seek | 可被食草动物觅食 |
| `grass` | flag | 分类 | 草皮 |
| `bush` | flag | 分类 | 灌木 |
| `cover.small` | flag | hide | 可容纳小型动物 |
| `berry.source` | flag | 觅食 | 产浆果 |
| `forages:TYPE` | TYPE=bush/underground | tick_reactive | 觅食类型 |
| `rooted` | flag | movement | 不可移动 |
| `organize.locked` | flag | interaction | 不可被叠（山、石） |
| `max_starve:N` | N=天数 | tick_starvation | 饿死阈值 |
| `meat_yield:N` | N=数量 | hunt_kill | 产肉量 |
| `corpse_type:NAME` | NAME=尸体类型 | finalize_prey_kill | 死亡后生成的尸体卡名 |
| `repro_offspring:NAME` | NAME=子代类型 | tick_reproduction | 繁殖后代 |
| `repro_pop_cap:N` | N=上限 | tick_reproduction | 种群上限 |
| `repro_cycle:N` | N=tick数 | tick_reproduction | 繁殖周期 |
| `repro_litter:N` | N=数量 | tick_reproduction | 每胎数量 |

## 9. 载体/容纳

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `cover:release_on_destroy` | flag | tick_reactive | 载体破坏→释放所有内容物 |
| `hide:exit_when_safe` | flag | tick_reactive | 安全→出藏（标签在动物） |
| `hide:exit_when_hungry` | flag | tick_reactive | 饿→出藏（标签在动物） |
| `cover_user` | flag | tick_reactive | 倾向于使用遮蔽物 |

## 10. 腐败

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `decay:fresh` | flag | tick_environment | 无腐败 |
| `decay:early` | flag | tick_environment | 变色 |
| `decay:advanced` | flag | tick_environment | 软化 |
| `decay:severe` | flag | tick_environment | 濒崩→变腐殖土 |
| `perishable` | flag | tick_environment | 可腐败 |

## 11. 疾病

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `food_poisoned` | flag | tick_reactive | 吃腐肉导致，行为变慢 |
| `fever` | flag | tick_reactive | 受寒淋雨导致 |
| `infected` | flag | tick_reactive | 咬伤导致 |

## 12. 地块

| 标签 | 格式 | 读取者 | 说明 |
|---|---|---|---|
| `soil:dry` | flag | plant_spawn | 旱地 |
| `soil:rich` | flag | plant_spawn | 沃土 |
| `soil:loose` | flag | plant_spawn | 松土（林地） |
| `soil:rocky` | flag | plant_spawn | 石地 |
| `soil:deep` | flag | plant_spawn | 深土 |
| `soil:wet` | flag | plant_spawn | 湿地 |
| `fertility:low\|high\|none` | flag | plant_spawn | 肥力 |
| `shaded` | flag | plant_spawn | 树荫下 |

---

> 变更记录：2026-06-11 初建。未列入的标签不得在代码中使用。
