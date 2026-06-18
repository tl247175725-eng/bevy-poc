# 九、关联映射

```yaml
cross_references:
  # 标签 -> 元数值
  tag_to_value:
    body_size -> estimate_mass_from_tags
    metab -> decay_rate, baseline_energy
    cognition -> combat_proficiency上限
    lifespan -> max_age, age_strength_factor

  # 标签 -> 元动作
  tag_to_action:
    diet -> Consume
    capability + body_size -> Strike
    body_plan + capability + body_size -> Move
    repro + social -> Reproduce
    cognition -> Decide

  # 标签 -> 标签
  tag_to_tag:
    diet + foraging + body_size -> 攻击策略
    social -> sexual_dimorphism(派生)
    reproduction + growth + lifespan -> 年龄曲线
```
