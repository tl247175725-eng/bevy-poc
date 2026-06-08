# 轻量 Rete：标签索引规则网络

**状态**：设计预案，Player AI 落地后推进
**日期**：2026-06-06

---

## 动机

当前 WorldRules 的规则是独立 if/else 链。200 条标签组合时足够。2000 条时每条规则的前件和其他规则大量重叠——"有 predator 标签？""有 prey 标签？""距离近？"被反复检查。

Rete 把共同条件合并成共享节点。标签变了→只重评估受影响的分支。不走全规则。

**不引入 `rust-rule-engine` crate。** 我们的规则数不到 Rete-NT 的适用门槛（200K+）。只取 Rete 的核心思想——标签 = 索引键，规则 = 条件树——在 WorldRules 内轻量实现。

---

## 核心结构

```
RuleIndex:
  tag_index: HashMap<Tag, Vec<RuleId>>     # "predator" → [Rule::Hunt, Rule::FleeIfAlone, ...]
  rules: Vec<Rule>                           # 所有规则定义
  shared_nodes: HashMap<TagSet, Node>        # 共同条件合并节点

Node:
  conditions: Vec<Condition>     # 该节点要判断的条件
  children: Vec<NodeId>          # 子节点
  rules: Vec<RuleId>             # 该节点可触发的规则

当一张卡的 predator 标签变更时：
  → tag_index["predator"] → [Hunt, Stalk, FleeIfAlone, ...]
  → 只评估这些规则，不扫全规则表
```

---

## 例子

没有 Rete：

```
规则1：predator + prey + near → hunt
规则2：predator + prey + near + weapon → stalk
规则3：predator + camp + near + alone → flee
每条独立判断所有条件。
```

有 Rete：

```
                    predator 标签索引
                         │
              (查1次：有prey?有camp?)
                    ┌────┴────┐
                prey         camp
                  │            │
               near?        near?
                  │            │
               weapon?      alone?
               ↗    ↖        ↓
          (有)hunt stalk    flee
           (无)
```

一次查标签→只走匹配的分支。

---

## 与现有系统关系

- WorldRules 仍为唯一入口
- 规则判定函数不改——只改"找到需要评估的规则"的方式
- `card_defs.ron` 不改
- 断言不改

---

## 计划顺序

Player AI → 轻量 Rete → Observer 深化 → 族群封装
