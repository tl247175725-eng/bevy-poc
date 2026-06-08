# C# 热路径迁移：WorldRules + EcosystemManager

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0（GDScript 天花板探测）
**Profiler 基准**: Phase 2 后 _game_tick 717ms。热点：domain 链 ~170s、type_has_tag 55s、card_has_tag 23s、CardBase._process 76s

---

## 目标

把 GDScript 中最热的纯数据函数迁移到 Godot C#（.NET 8），验证 Godot 性能上限。不动行为逻辑，只换语言实现。

---

## 一、迁移范围

### 1.1 必须迁（P0）

| GDScript 函数 | 累计耗时 | C# 类 |
|------|------|------|
| `card_has_tag` + `type_has_tag` | ~78s | `TagLookup` |
| `defines_domain` / `type_domains` / `camp_domain_anchor` | ~170s 链 | `DomainResolver` |
| `best_feed_source_for` / `best_hunt_target` | ~62s | `FeedHuntSolver` |
| `SpatialIndex.query_tag` / `query_near` / `query_at` | ~9s | `SpatialIndexSharp` |

### 1.2 可选迁（P1）

| GDScript 函数 | 累计耗时 | 
|------|------|
| `get_card_at` (13s) | |
| `CardBase._refresh_face` (38s) | |
| `PlayerBehavior.tick` (19s) | |

### 1.3 不迁

- UI 渲染（bevy_ui 等前端活，C# 不管）
- 精灵动画、纹理加载
- L0 测试框架（保留 GDScript 验证，C# 跑同一逻辑）

---

## 二、实现方式

### 2.1 Godot C# 项目配置

Godot 4.x 使用 .NET 8。项目需 Mono 版本 Godot（非标准版）。确认你的 Godot 是 Mono 构建。

在项目根目录创建 `.csproj`，或在 Godot 编辑器中 `Project → Tools → C# → Create C# Solution`。

### 2.2 GDScript ↔ C# 互调

**GDScript 调 C#**：C# 类挂到 Autoload 或作为 `Node` 子节点：

```csharp
// SpatialIndexSharp.cs
using Godot;
using System.Collections.Generic;

public partial class SpatialIndexSharp : Node
{
    private Dictionary<string, List<Node>> byTag = new();
    private Dictionary<Vector2I, Dictionary<string, List<Node>>> byCell = new();

    public void OnSpawn(Node card, string[] tags, int x, int y) { ... }
    public List<Node> QueryTag(string tag) { ... }
    public List<Node> QueryNear(int x, int y, string tag, int radius) { ... }
}
```

GDScript 中：
```gdscript
var sharp = $SpatialIndexSharp
var grasses = sharp.QueryTag("grass")
```

**热路径原则**：一次调用进入 C# 后，在 C# 内完成全部计算，尽量减少 GDScript↔C# 往返。

### 2.3 数据传递

- 基础类型（int、string、Vector2I）零开销
- `Node` 引用传 GDScript 卡对象——C# 可以持有 GDScript 对象引用
- 返回 `Array` 时用 Godot.Collections.Array 或 `List<Node>` 转 `GodotArray`

---

## 三、分步执行

### Step 1：TagLookup（最小验证）

把 `card_has_tag` 和 `type_has_tag` 的热路径迁到 C#。

GDScript 中改为：
```gdscript
static func card_has_tag(card, tag):
    return TagLookup.instance.card_has_tag(card, tag)
```

C# 中 `TagLookup` 维护 `Dictionary<Node, HashSet<string>>` 和 `Dictionary<string, HashSet<string>>`（type → tag set）。

**验收**：L0 中 `card_has_tag` 相关断言 PASS。Profiler 中 `card_has_tag` 累计 < 1s。

### Step 2：DomainResolver

把 `defines_domain` / `type_domains` / `camp_domain_anchor` / `in_camp_home_territory` 整条链迁到 C#。

关键：`camp_domain_anchor` 返回值在 C# 内缓存（per tick），不每次查。

**验收**：`defines_domain` + `type_domains` 累计 < 5s。

### Step 3：FeedHuntSolver

把 `best_feed_source_for` / `best_hunt_target` 迁到 C#。输入 actor 卡 + 候选列表（从 SpatialIndex 查），输出最优目标。

**验收**：`best_feed_source` 累计 < 5s，狼狐行为不变。

### Step 4：SpatialIndexSharp

当前 `SpatialIndex.gd` 是纯 GDScript。用 C# 重写一份，数据结构用 `Dictionary<Vector2I, Dictionary<string, List<Node>>>`，API 一致。

**验收**：Profiler 中 `SpatialIndex.query_*` 累计 < 3s，生态行为不变。

---

## 四、涉及文件

| 文件 | 改动 |
|------|------|
| `csharp/TagLookup.cs` | **新建** |
| `csharp/DomainResolver.cs` | **新建** |
| `csharp/FeedHuntSolver.cs` | **新建** |
| `csharp/SpatialIndexSharp.cs` | **新建** |
| `scripts/core/world_rules.gd` | 热点函数委托到 C# |
| `scripts/world/ecosystem_manager.gd` | feed/hunt 调用改 C# |
| `scripts/core/spatial_index.gd` | 可选：双轨（GDScript 保留，C# 镜像） |
| `*.csproj` | **新建** |

## 五、验收

- L0 993 断言不降
- Profiler 复测：`_game_tick` 目标 < 150ms
- `type_has_tag` / `card_has_tag` / domain 链累计 < 10s
- 生态行为完全一致

## 约束

- 不碰行为逻辑
- 不碰标签/能力定义
- GDScript 保留回退路径（若 C# 异常，fallback 到原 GDScript 实现）
- 记 fix-log
