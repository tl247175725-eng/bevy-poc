# 技术栈——每次执行 handoff 前必读

## 运行时
- **Rust**: 稳定版（当前 ~1.83+）
- **Bevy**: `0.15`（非 0.16 非 0.17！）
- **bevy_egui**: `0.31`
- **ron**: `0.8`
- **serde**: `1`（derive 特性）

## Bevy 0.15 关键 API（和其他版本不同之处）

```
⚠️ 这些是 0.15 特有——不兼容 0.16/0.17 写法：

Mesh 创建:
  Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())  // 2 参数！

Mesh3d vs Handle<Mesh>:
  实体用 Mesh3d 组件，不是 Handle<Mesh>
  获取 mesh: meshes.get_mut(&handle)（Assets<Mesh> 资源）

RenderAssetUsages:
  use bevy::render::render_asset::RenderAssetUsages;

Indices:
  mesh.insert_indices(Indices::U32(indices));  // 仍可用

SimClock:
  game_time_seconds: f64  // 不是 tick_count: u64
  tick = game_time_seconds / 0.5
  TICKS_PER_DAY = 420

WorldState:
  模拟不在 Bevy ECS 里——并行不会触碰到 ECS
  main_tick(&mut WorldState, delta) 是串行循环
```

## 关键版号

```
TICKS_PER_DAY: 420
GRID_WIDTH: 32
GRID_HEIGHT: 32
Bevy: 0.15
```

## dsc 配置（项目 `.deepseek/config.toml`）
```
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
base_url = "https://api.deepseek.com/v1"   ← 必须有！缺了会 EOF
```

## 外部 crate 引入规则

任何新 crate 必须查验其 Bevy 版本要求。不匹配→不引入→参考源码自己写。
绝不为用新 crate 升级 Bevy。
