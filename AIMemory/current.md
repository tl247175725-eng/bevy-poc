# Current Handoff
- file: AIMemory/handoff_bulletin-board.md
- mode: Standard
- status: completed

## 架构计划
新增 BulletinBoard 资源，50 tick 全局扫描一次，实体按标签订阅对应频道。

## 架构反馈
复用现有 spatial_index 扫描 + 标签系统，低频访问。

## 智能验收
- 饥饿动物朝远处食区移动
- 公告板 50 tick 更新一次
- smoke test PASS