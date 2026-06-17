# Handoff 001 — 扩展 meta_values.rs 为完整 A/B 层元数值常量

## 架构计划

**改什么：** 仅 `src/meta_values.rs`（1 个文件）
**为什么：** 当前 meta_values.rs 是 66 行的 shell，包含魔法定值函数。需要替换为完整的 A/B 层元数值常量，所有数值追溯设计文档。

**依据设计文档：** `AIMemory/design-philosophy-v5.md` §8.1（A 层），§8.2（B 层）
**依据铁律：** 所有数字可追溯到 meta_values.rs，无魔法数字

**具体改动：**

1. 文件头注释更新：引用设计哲学 §8.1/§8.2，声明铁律 1 day=420 tick
2. 保留：`TICK_SECONDS=0.5`（标注：渲染采样率，非游戏内逻辑时间）
3. 保留：`TICKS_PER_DAY=420`
4. 保留：`GRID_CELL_SIZE=1.0`
5. 新增 A 层常量（全部为 pub const）：
   - 时间：`TICKS_PER_PHASE=60`，`PHASES_PER_DAY=7`
   - 空间：`CELL_AREA=1.0`，`CELL_VOLUME=1.0`
   - 材料物理参考值（pub const，作为引擎内置材质数据库的索引键）：
     `DENSITY_WATER=1000.0`，`DENSITY_WOOD=700.0`，`DENSITY_STONE=2700.0`，`DENSITY_IRON=7874.0`
     `DENSITY_COPPER=8960.0`，`DENSITY_BRONZE=8700.0`，`DENSITY_STEEL=7850.0`，`DENSITY_GOLD=19320.0`，`DENSITY_SILVER=10490.0`
     `DENSITY_LEATHER=860.0`，`DENSITY_BONE=1900.0`，`DENSITY_GLASS=2500.0`，`DENSITY_CLAY=1800.0`，`DENSITY_FLESH=1050.0`，`DENSITY_ICE=917.0`
     `HARDNESS_WOOD=1.0`（莫氏），`HARDNESS_COPPER=3.0`，`HARDNESS_BRONZE=3.5`，`HARDNESS_IRON=4.0`，`HARDNESS_STEEL=6.5`，`HARDNESS_STONE=7.0`，`HARDNESS_GLASS=5.5`
     `YIELD_WOOD=40.0`，`YIELD_COPPER=200.0`，`YIELD_BRONZE=350.0`，`YIELD_IRON=500.0`，`YIELD_STEEL=800.0`（MPa）
     `FRACTURE_WOOD=70.0`，`FRACTURE_COPPER=250.0`，`FRACTURE_BRONZE=500.0`，`FRACTURE_IRON=800.0`，`FRACTURE_STEEL=1200.0`（MPa）
     `TOUGHNESS_STEEL=150.0`，`TOUGHNESS_GLASS=0.01`，`TOUGHNESS_WOOD=2.0`（J/m³）
   - 热：`TEMP_ABSOLUTE_ZERO=-273.15`，`TEMP_FREEZING_WATER=0.0`，`TEMP_BOILING_WATER=100.0`，`TEMP_IGNITION_WOOD=300.0`，`TEMP_BODY_MAMMAL=37.0`
   - 感官默认：`VISION_RANGE_DEFAULT=6`，`HEARING_RANGE_DEFAULT=8`，`SMELL_RANGE_DEFAULT=4`
6. 新增 B 层常量（全部为 pub const）：
   - 生命：`HP_BASELINE=1`（工程保留，标注妥协性质），`METABOLISM_BASELINE=1.0`
   - 心智：`DECISION_THRESHOLD_DEFAULT=1.2`
   - 社会：`NORM_STRENGTH_DEFAULT=0.5`
7. 替换旧函数（签名不变，函数体改为元数值推导）：
   - `size_to_weight(size)` → `weight_from_mass_density(mass_kg, density)`：weight = mass × gravity_constant
   - `size_to_speed_mod(size)` → `speed_from_mass(mass_kg)`：speed = base_speed × (1.0 / mass.sqrt())
   - `impact_damage(weight, speed)` → `impact_force(mass_kg, velocity, contact_area, hardness_ratio)`：force = 0.5 × mass × velocity² / contact_area × hardness_ratio
   - `entity_move_speed(size)` → 删除（D 层），调用方后续迁移
   - `entity_sprint_speed(size, tier)` → 删除（D 层），调用方后续迁移
   - `BASE_ENERGY=1` → 删除（魔法数字），替换为 `baseline_energy(mass_kg, metabolism_rate)`
8. 新增辅助常量：`GRAVITY=9.8`（m/s²，地球参考值——非"地球"，是"1G 基准"）
9. 新增测试：
   - `ticks_per_day_is_420`
   - `ticks_per_phase_is_60`
   - `all_constants_non_negative`
   - `weight_is_mass_times_gravity`
   - `impact_force_increases_with_mass`
   - `speed_decreases_with_mass`

**不改：**
- 现有 valid 测试（`all_speeds_are_positive`）——修改为以新函数为基准
- 不碰其他任何文件

## 架构反馈

**与设计哲学一致性：**
- 所有常量可追溯到 `design-philosophy-v5.md` §8.1/§8.2 的具体条目 ✅
- 材料物理值为真实物理值（MPa/kg/m³/莫氏），不分级 ✅
- 函数从元数值推导，无魔法定值 ✅
- 删除的 D 层函数（speed/damage）会在后续迁移调用方时重新从元数值正确推导 ✅
- `HP_BASELINE=1` 标注为工程妥协，逻辑上由 wound/infection/toxin/hydration 等共同决定 ✅

**未解决的问题（留给后续 handoff）：**
- `entity_move_speed` 和 `entity_sprint_speed` 的调用方（movement.rs 等）尚未迁移——删除后编译会报错，需要后续 handoff 逐一修
- 判断：这两个函数是 D 层派生，不应在元数值文件中定义。当前 handoff 不下沉到调用方
- 如果编译报错，临时在调用方做最小迁移（直接调用 `speed_from_mass`）

**符合铁律：**
- 单一数字来源 ✅
- 无魔法数字 ✅
- 材质真实物理值 ✅
- 所有数值可追溯 ✅

## 智能验收

编译期（自动拦）：
- `cargo check` 零错误
- `cargo clippy -- -D warnings` 零错误
- `cargo fmt --check` 通过

测试期：
- `cargo test` 全 PASS
- 新增以下测试全部 PASS：
  - `ticks_per_day_is_420` — assert_eq!(TICKS_PER_DAY, 420)
  - `ticks_per_phase_is_60` — assert_eq!(TICKS_PER_PHASE, 60)
  - `weight_derives_from_mass_and_gravity` — assert_eq!(weight_from_mass_density(10.0, 1000.0), 10.0 × 9.8)
  - `speed_decreases_with_heavier_mass` — speed_from_mass(100.0) < speed_from_mass(10.0)
  - `impact_force_increases_with_mass` — impact_force(100.0, 1.0, 1.0, 1.0) > impact_force(10.0, 1.0, 1.0, 1.0)
  - `all_constants_positive` — 至少检查 TICKS_PER_DAY > 0, GRID_CELL_SIZE > 0, GRAVITY > 0
