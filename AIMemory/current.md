# Current Handoff
- file: AIMemory/handoff_fix-animation.md
- mode: Standard
- status: completed

## 架构计划
重写 lerp 替代失效的 bevy_tweening，视觉层动画与公理层无关。

## 架构反馈
外部 crate 不可靠，自写 lerp 更可控。

## 智能验收
- 断言：smoke test herbivore 移动数 > 0 且 predation > 0
