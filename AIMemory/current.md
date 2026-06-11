# Current Handoff
- file: AIMemory/handoff_bulletin-board.md
- mode: Standard
- status: pending

## 架构计划
新增 BulletinBoard 资源，50 tick 全局扫描一次，实体通过标签按需查频道。

## 架构反馈
复用现有 spatial_index 扫描 + 标签系统控制访问。

## 智能验收
- 公告栏按频道更新
- 实体标签控制访问
- smoke test PASS
