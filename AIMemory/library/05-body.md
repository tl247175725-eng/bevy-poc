# 五、身体模拟系统（齐平矮人要塞深度）

```yaml
body_simulation:
  principle: "标签驱动+数据定义，零硬编码。加新物种=选模板+填标签"
  depth: "组织层（皮肤/脂肪/肌肉/肌腱/骨骼/器官）。细胞以下不模拟"
  
  body_templates:  # 按 body_plan 标签自动选择模板
    quadruped: "四足（鹿虎象犀猪牛豺熊獭鳄）"
    biped: "双足（人类/虎人）"
    avian: "鸟形"
    serpentine: "蛇形（蟒蛇）"
    fish: "鱼形（鲤鱼/鲶鱼）"
    insectoid: "虫形（容纳型）"
    plant: "植物型"

  tissue_layers:  # 每个身体区域从外到内的层
    - skin: "第一道防线。defense:armor 修改此层强度"
    - fat: "能量储备+缓冲+保温。body_fat 池决定厚度"
    - muscle: "力量来源。损伤=该区域力量下降"
    - tendon: "连接肌肉和骨骼。断裂=肌肉无法作用于关节"
    - bone: "结构支撑。骨折=结构失效"
    - organ: "核心功能。穿透到此=器官损伤"

  body_systems:  # 每个器官属于一个系统，系统损伤有全局后果
    circulatory: "心脏+血管。心脏破=即死。出血=blood_volume下降"
    respiratory: "肺x2+气管+鳃(鱼)。窒息/溺水"
    nervous: "脑+脊髓+神经。瘫痪/昏迷/即死"
    digestive: "胃肠+肝+肾。穿透=感染。肝伤=代谢崩溃"
    reproductive: "睾丸x2/卵巢x2+子宫+乳腺。损伤=不育/不能哺乳"
    sensory: "眼x2+耳x2+鼻+口舌+侧线(鱼)。损伤=感知通道降级"
    musculoskeletal: "各部位骨骼+肌肉+肌腱。骨折/肌肉撕裂/肌腱断裂"
    integumentary: "皮肤+毛发/鳞片/羽毛。破裂=出血+感染风险"

  entity_pools:  # 全身级别的宏观池（不是每个部位独立跟踪）
    blood_volume: "1.0=正常, <0.5=虚弱, <0.2=休克, 0=死。bleeding伤口每tick扣减"
    body_fat: "0.0=极瘦, 1.0=肥胖。决定冬眠能力/饥饿耐受/繁殖条件"
    stamina: "短期体力。sprint/strike消耗，rest恢复。0=被迫休息"
    pain: "痛觉累积。>阈值1=迟缓, >阈值2=昏迷"
    consciousness: "清醒/眩晕/昏迷/死亡。从pain+blood_loss+brain_damage推导"

  missing_systems_to_implement:  # 补齐矮人要塞深度
    - pain_consciousness: "痛觉累积->昏迷。每个受伤部位产生痛觉值"
    - healing_scarring: "bleeding->止血->healing->scarred。severed不可愈合"
    - stance_prone: "四足需>=3腿站立。<3=倒地=不能移动+防御x0.3"
    - grasp_hold: "bite咬住=Constrain。jaw伤=松嘴。hand伤=掉工具"
    - fatigue_stamina: "高强度行动消耗stamina。0=被迫休息"
    - fur_feather_scale: "最外层覆盖。影响保温+防御"

  body_process_framework:  # 统一的身体过程框架（数据驱动）
    schema: |
      BodyProcess = {
        trigger: age/season/contact/per_action/random/injury_present,
        target: 具体body_part或organ或system,
        effect: degrade/grow/shed/infect/heal,
        rate: f32,
        reversible: bool,
        transmissible: bool,
        immunity_after: bool
      }
    examples:
      - "tooth_wear: trigger=per_consume, target=teeth, effect=degrade, rate=0.001"
      - "antler_shed: trigger=season:winter, target=antler, effect=shed"
      - "aging_vision: trigger=age>0.7, target=eye, effect=degrade, rate=0.01"
      - "plague: trigger=contact, target=全身, effect=degrade, rate=0.3, transmissible=true"
      - "wound_infection: trigger=injury:bleeding_present, target=wound_site, effect=infect"
      - "healing: trigger=time, target=injured_part, effect=heal, rate=0.05"
      - "cancer: trigger=random(age_weighted), target=random_organ, effect=degrade"
      - "pregnancy: trigger=mating_success, target=uterus, effect=grow, duration=species_specific"
    principle: "加新疾病/衰老效果/季节变化 = 加一行数据，不写代码"

  species_override:  # 种内差异化（不改模板，改参数）
    principle: "body_plan选模板，标签改参数，overrides加特殊部位"
    examples:
      - "犀牛: defense:armor -> skin.yield_strength x5"
      - "大象: 额外body_part:trunk（鼻=第五肢）"
      - "鹿(公): 额外body_part:antler（季节性）"
      - "穿山甲: skin.material=keratin_scale, yield_strength x8"
      - "蟒蛇: 只有右肺功能(左肺退化), 热感应窝(sense:infrared)"

  aging_interaction: "衰老不是独立过程——是所有BodyProcess的速率乘以age_modifier"
  
  hormone_abstraction: "不模拟激素浓度——通过need/personality/state标签代替效果"
```
