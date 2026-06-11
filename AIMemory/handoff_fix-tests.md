# 修复 CI 测试失败（3 个）

**Priority**: P0

## 问题
`cargo test --release` 有 3 个失败。藏=容纳逻辑更新后可能打破了旧测试假设。

## 修复
1. 跑 `cargo test --release 2>&1 | grep "FAILED"` 找出具体测试名
2. 读测试代码，调整为新逻辑的期望值
3. 全量测试 PASS

## 验收
- `cargo test --release` 全 PASS
- smoke test PASS
