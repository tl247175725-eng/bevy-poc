# 六、生态系统

### 植物系统

```yaml
plant_distribution:
  principle: "植物能不能在某格生长 = 格子环境条件 in 植物耐受范围"
  matching_tags:
    cell_conditions: "水分(淹水/湿润/干燥) x 光照(全日照/半阴/全阴) x 海拔(低/中/高)"
    plant_tolerance: "flooding_tolerance x drought_tolerance x shade_tolerance"
  rule: "条件匹配->能长。不匹配->不能长。不硬编码'莲花只能在水里'"
  initial_generation: "方案A——按地形环预设合理分布，之后传播系统自维持"

plant_propagation:
  principle: "植物怎么繁衍 = dispersal标签决定传播方式 + growth标签决定速度"
  dispersal_methods:
    wind: "种子随风飘散，半径3-10格，方向随机"
    animal: "果实被吃->动物排泄->种子跟着动物路线走"
    water: "种子顺水流漂->沿水域扩散"
    gravity: "种子直接落脚下->只扩散到相邻格->形成密集群落"
    explosive: "豆荚爆裂->半径1-2格"
  growth_rate:
    fast: "草类——几天恢复"
    medium: "灌木——几十天"
    slow: "乔木——几百天"
  succession: "涌现效果：砍光森林->草先到(wind快)->灌木来(animal中)->树苗长(slow)->成熟森林"
  player_interaction: "抽=从格子取种子卡。叠=把种子放到新格子->条件匹配->发芽"

plant_storage:
  principle: "植物是格子上的数量卡，不是独立实体"
  model: "cell_resources: HashMap<(x,y), HashMap<plant_type, quantity>>"
  consume: "动物吃->quantity -= 1"
  regrow: "传播系统->符合条件的格子->quantity += 1"
```

### 繁殖系统

```yaml
reproduction:
  prerequisites:  # 全部从标签/运行时状态推导
    - "年龄 > 性成熟年龄（从 growth 标签推）"
    - "body_fat > 0.3（营养不良不繁殖）"
    - "生殖器官完好（ovary/testicle 未损伤）"
    - "state:pregnant 不存在"
    - "不在 state:starving"
    - "季节匹配（部分物种季节性繁殖）"

  mating:
    action: "MetaAction::Reproduce { partner }"
    condition: "同物种 + 异性 + 成年 + 邻格 + 双方繁殖需求激活"
    social_variation:
      solitary: "只在繁殖季寻找，交配后分开"
      pair: "固定配偶长期在一起"
      herd: "群内交配"
      pack: "通常只有头领繁殖"

  gestation:
    principle: "用真实数据标签，不用公式（妊娠期无可靠万能公式）"
    implementation: "每种动物在 card_defs.ron 标注 gestation_days: N"
    during_pregnancy: "营养需求增加、移动速度降低、不能再次交配"
    egg_layer: "先产卵（卵是一张卡）-> 孵化期 -> 幼崽出壳"

  birth:
    few_offspring: "1-2只（K策略，大型动物）"
    many_offspring: "3-8只（r策略，小型/鱼类）"
    fish_special: "产卵数百->概率存活率->实际幼鱼少"

  parental_care:
    with_care: "幼崽跟随母亲。mammary完好+body_fat>0.2->哺乳->存活"
    without_care: "出生即独立（鱼类、多数爬行类）"
    mammary_damage: "乳腺损伤->不能哺乳->幼崽存活率降低"

  inheritance:
    same_as_parents: "物种标签、body_plan、diet、habitat、capability"
    random: "personality（随机）、individual_modifier（正态分布）、sex（50/50）"
    not_inherited: "injury（伤疤不遗传）、experience（要靠Teach传授）、body_fat（从零积累）"
```

### 捕食链

```yaml
predation_chain:
  principle: "需求引擎的重新规划机制天然形成捕猎循环，不需要写循环逻辑"
  
  carnivore_knowledge:
    decomposition: "[Acquire(找animal), Act(Strike), Act(Consume)]"
    vs_current: "当前只有[Acquire, Consume]——缺Strike步骤"
  
  consume_animal_requirement:
    rule: "吃动物必须目标is_corpse=true（必须先杀死才能吃）"
    vs_current: "当前can_digest允许吃活的——需修改"
  
  natural_loop:
    - "虎饿了->制定计划[找猎物,Strike,Consume]"
    - "Strike->鹿HP减少但没死->Consume失败(鹿还活着)"
    - "计划失败->但还饿->重新制定计划->继续追同一只鹿"
    - "重复直到：鹿死了(Consume成功) 或 鹿逃了(Acquire失败->换目标)"
  
  give_up_conditions:
    - "猎物超出感知范围（跑太远）"
    - "猎物进入不可达地形（跳进水里）"
    - "自己stamina耗尽（跑不动了）"
    - "猎物太强（反击造成严重injury）"
```

### 腐烂与骨骼

```yaml
decomposition:
  formula: "积温日 ADD = sum max(daily_temp - 0C, 0) / mass_factor"
  mass_factor: "(mass_kg / 10.0)^0.25 — 大尸体腐烂更慢"
  principle: "万能公式——温度越高腐烂越快，冰冻=不烂。完全同构"

  soft_tissue_stages:
    fresh:        { add_threshold: 0,    desc: "刚死，外观无变化" }
    bloating:     { add_threshold: 100,  desc: "气体积累，发臭" }
    active_decay: { add_threshold: 200,  desc: "大量质量流失，吸引食腐动物" }
    advanced:     { add_threshold: 500,  desc: "软组织基本消失" }
    skeleton:     { add_threshold: 1000, desc: "只剩骨骼->产生骨骼资源卡" }

  bone_weathering:  # 骨骼=70%矿物质，本质是石头风化速度
    intact:       { add_threshold: 1000,  desc: "白骨暴露" }
    cracking:     { add_threshold: 5000,  desc: "表面裂纹" }
    fragmenting:  { add_threshold: 15000, desc: "断裂分解" }
    dissolved:    { add_threshold: 30000, desc: "完全融入土壤->肥力增加" }

  bone_as_resource:
    - "骨骼是资源卡，留在格子上"
    - "玩家可'抽'出来->做骨针/骨刀/骨笛"
    - "玩家可'砸'碎->骨粉->撒地上当肥料"
    - "食骨动物可 Consume 骨头（补钙）"
    - "玩家可'叠'到土里->埋入->加速分解+增加肥力"
    - "火烧->煅烧骨（更硬的材料，不是加速分解）"

  nutrient_cycle:
    - "尸体腐烂->养分回到土壤->格子 soil_fertility 增加->植物生长加速"
    - "碳循环闭合：植物->动物->土壤->植物"
    
  scavenger_interaction:
    - "active_decay 阶段->产生嗅觉信号->Perceive:smell->食腐动物感知"
    - "diet:scavenger 动物->Consume 尸体软组织部分"
    - "skeleton 阶段->无软组织可吃->食腐动物不再来"
```

### 温度系统

```yaml
planned_runtime_dimensions:
  - id: ambient_temperature
    type: "运行时（每格每 tick 变化）"
    source: "季节 + 海拔 + 水深 + 昼夜"
    priority: "必须实现——腐烂/代谢/冬眠/植物生长全依赖温度"
    affects_everything:
      - ectotherm_metabolism: "Q10=2.5: 温度每降10C代谢减半"
      - endotherm_energy_cost: "越冷->维持体温越费能量->饿得更快"
      - movement_speed: "变温动物冷了动不了"
      - hibernation_trigger: "温度降到阈值->metab:torpor动物进入冬眠"
      - plant_growth: "冬天停止生长->食物减少"
      - water_freeze: "鱼活动空间缩小"
      - decomposition: "ADD公式核心变量——温度决定腐烂速度"
      - disease_spread: "温暖潮湿->传染病传播加速"
    formula: "B_actual = B0 x Q10^((T - T_ref) / 10)"
    decomposition_formula: "ADD_per_tick = max(temp - 0, 0) / TICKS_PER_DAY"
    tags_involved: [thermo:endotherm/ectotherm, metab:torpor, habitat:aquatic]
    status: "未实现——必须优先实现"

  - id: body_condition
    type: "运行时（每个实体，随进食/消耗变化）"
    source: "进食量 - 代谢消耗 的累积"
    affects_everything:
      - fasting_endurance: "胖动物比瘦动物多活80%（Trondrud 2021 实测）"
      - hibernation_ability: "脂肪不够->不敢冬眠->冬天饿死"
      - reproduction: "体况太差->不繁殖"
      - combat_power: "饥饿动物战力下降"
      - movement_speed: "太瘦跑不动，太胖也慢——有最优区间"
      - offspring_survival: "母亲体况差->奶水不足->幼崽死亡率高"
    formula: "body_condition = fat_reserve / lean_mass（0.0=极瘦, 1.0=极胖）"
    tags_involved: [body_size, metab, repro]
    status: "未实现——优先级高，连接'吃了多少'和'能做什么'"
```
