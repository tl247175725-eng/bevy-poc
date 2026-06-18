# 三、世界设定

```yaml
world_setting:
  environment: "亚热带火山环境——火山土壤极肥沃，支撑高生物多样性"
  area_basis: "50人狩猎采集部落的生存面积 → ~100 km²（10km×10km）"
  grid: "64×64 = 4096 格（待实现，当前代码仍为 32×32）"
  cell_size: "~2.5 公顷（158m×158m）"
  species_scope: "中国历史上存在过的所有物种（含已灭绝）共存"
  stability: "高度稳定生态——不会轻易打破，只有极端干预才能破坏平衡"
  current_status: "16种动物是未完成子集——物种名单待补全"
  population_note: "当前捕食者/猎物比例失衡是因为物种不全，不是数字错误"
  building: "模块化——一种建筑占一格，不是一个家具一格"
```

### Z 轴与多层棋盘体系

```yaml
z_axis:
  position: "Entity { x: u8, y: u8, z: i16 }  // 负=地下, 0=地表, 正=空中"
  spatial_index: "3D 桶，每层独立 2D 棋盘"

  layer_types:
    abstract_channel:
      z_range: "命名层之间"
      display: "进度条 + 产出表 + 随机发现事件"
      permission: "挖/取/运输，不可建基地"
    named_board:
      z_range: "地质年代/大气层分界"
      display: "完整棋盘（36x24）"
      permission: "全部权限"
    sky_platform:
      z_range: "任意高度"
      display: "完整棋盘"
      permission: "需悬浮能力+持续动力消耗"

  activation:
    - "命名层 -> 触及层边界 -> 自动激活该层棋盘"
    - "放置基地卡 -> 任意 Z 坐标手动激活棋盘"

  travel_rules:
    same_z: "曼哈顿移动"
    cross_z: "需要运输卡（竖梯/电梯/火箭等）-> Z 层间不自动连通"
    sky: "位置 (x,y,z) 不自动连通地面 (x,y,0)"

  underground_layers:  # 地质年代（深度为游戏压缩值）
    - { name: "更新世", depth: "100m", resource: "猛犸化石、冰期沉积" }
    - { name: "上新世", depth: "300m", resource: "铁矿、褐煤" }
    - { name: "中新世", depth: "500m", resource: "石油层" }
    - { name: "白垩纪", depth: "800m", resource: "白垩岩、恐龙化石" }
    - { name: "石炭纪", depth: "1200m", resource: "煤矿层" }
    - { name: "寒武纪", depth: "2000m", resource: "页岩、三叶虫化石" }
    - { name: "前寒武纪", depth: "3500m", resource: "铁矿石、花岗岩" }

  sky_layers:  # 基于 NOAA 2026 标准大气
    - { name: "对流层", height: "0-12km", env: "正常~补充氧" }
    - { name: "平流层", height: "12-50km", env: "需增压服" }
    - { name: "中间层", height: "50-85km", env: "太空级维生" }
    - { name: "热层", height: "85-600km", env: "航天器" }
    - { name: "外大气层", height: "600km+", env: "轨道空间" }

  underground_vs_sky:
    underground: { support: "下方稳定链->地表", energy: "无（静态结构）", movable: "不可", travel: "物理连通（楼梯/竖井）", hover: false }
    sky: { support: "悬浮动力持续消耗", energy: "持续消耗（燃料/能源）", movable: "可移动（漂移/重定位）", travel: "运输链路（电梯/火箭）", hover: true }

  inactive_layer_handling:
    current: "完整 tick 模拟"
    non_current: "概率积分估计（时间 x 能力 x 资源 -> 最可能结果）"
    non_activated: "不存在（不存数据，不消耗性能）"
```
