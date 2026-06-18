# 八、公式 vs 数据原则

```yaml
formula_vs_data:
  use_formula:  # 有跨物种一致的物理/热力学规律
    - "饥饿致死天数: 100 x M^(1-beta) / B0 x torpor_mult（热力学）"
    - "攻击力: impact_force(mass, velocity, area, hardness)（物理）"
    - "代谢率: baseline_energy(mass, metab_rate)（Kleiber定律）"
    - "体型->质量: estimate_mass_from_tags（等比例缩放）"
    - "营养衰减: metab_rate / (TICKS_PER_DAY x TICK_SECONDS)（能量守恒）"
  
  use_real_data:  # 物种差异太大，无万能公式
    - "妊娠期: gestation_days 标签（查资料填）"
    - "性成熟年龄: maturity_days 标签"
    - "最大寿命: max_age 标签"
    - "每胎数量: litter_size 标签"
  
  principle: "能用公式推的用公式（同构推导）。不能的用真实数据（同构记录）。两种都是同构。"
```
