# 快捷短语（一键发送）

## 让 DeepSeek 读代码查问题
- `系统性检查代码，一次性找出所有问题，给清单和优先级`
- `对比 Godot 和 Bevy 两套代码，列出不一致的地方`
- `读代码找出 [具体现象] 的根因`

## 让 DeepSeek 写执行单
- `写 handoff`
- `发 Standard`（发 handoff 给 Cursor，Standard 模式关 Max）
- `发 Max`（发 handoff 给 Cursor，Max Mode）

## 问问进度
- `Cursor 在干嘛`
- `整体进度`

## 设计讨论
- `把这个思路记下来`
- `查一下业界做法 / 论文 / 有没有人做过类似的东西`

## 模式提醒
- `什么模式`（提醒我告诉你 Cursor 该用什么模式）
- `Standard，关 Max`
- `开 Max`

## 自动驱动 Cursor

### 启动自动化脚本
- 打开 PowerShell → 执行 `powershell -ExecutionPolicy Bypass -File tools/auto-cursor.ps1`
- 脚本会在后台每 30 秒检查一次 GitHub 是否有新 handoff
- 检测到 `AIMemory/current.md` 状态变为"待执行" → 自动切到 Cursor → 粘贴指令 → 发送

### DeepSeek 工作流
1. 跟我讨论完 → 我写 handoff → 更新 `AIMemory/current.md`（状态=待执行）
2. push 到 GitHub
3. 你的 `auto-cursor.ps1` 自动检测到 → 驱动 Cursor 开始干活
4. Cursor 完成后 git push → CI 自动验证 → 你去看 Actions 结果

### Cursor 完成后（手动）
- 切到 Actions 页面看结果：全绿 → 继续；有红 → 把错误发给我

## GitHub CI 工作流

### 仓库
- 地址：`https://github.com/tl247175725-eng/bevy-poc`
- CI 页面：`https://github.com/tl247175725-eng/bevy-poc/actions`
- 每次 push 自动运行 `cargo test --release` + `cargo run --release -- --smoke-test`

### 本地操作
- `cargo check` — 轻量语法检查（几十秒），本地唯一需要执行的
- **不要本地跑 `cargo test` 或 `cargo run --release`**，交给 CI

### Cursor 工作流
1. Cursor 改完代码 → `git add` + `git commit` + `git push`
2. 自动触发 GitHub Actions
3. 等几分钟 → 打开 Actions 页面看结果
4. 全绿 = PASS，有红 = 点进去看失败日志

### 验收标准
- CI 两个 job 全部绿色：`test` + `smoke`
- 不绿不继续下一步
