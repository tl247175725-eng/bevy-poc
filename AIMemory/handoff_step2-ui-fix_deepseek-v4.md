# 步骤 2 补修：狐狸窝砸击UI + 信息栏完整性

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 问题 1：狐狸窝缺砸击 UI

狐窝可被砸毁（interaction_manager L141-163 已有 handler），但 UI 信息栏没有砸击计数。

**参考**：狼窝在 `game_ui.gd` L686 有 `"砸击：%d / 2"`。狐窝的 `render_selected()` 分支在 L698-712，砸击行缺失。

**要求**：给狐窝的信息栏补上砸击计数，与狼窝一致（2 次砸毁）。

## 问题 2：信息栏缺失字段

多张卡有属性但信息栏不显示。检查 `game_ui.gd` 的 `render_selected()` 函数：

- deer/fieldMouse/fox 有 `sex` 但鹿和田鼠可能被 L734-741 的 fallback 漏掉或条件不匹配
- bush 有 worm_count/beetle_count/berry_ready → 已在 L713-721 ✓
- fieldMouse 信息栏 L722-733：确认性别是否正确显示
- fox 信息栏 L687-697：确认性别是否正确显示

**要求**：逐一核对上述卡的信息栏完整性，补上缺失字段。

## 参考

- 代码模式索引：`AIMemory/code-pattern-index.md`
- 狼窝砸击参考：`game_ui.gd` L669-686
- 信息栏入口：`game_ui.gd` `render_selected()` L584+

## 回复方式

在本文件末尾追加 `# 回复` 段，记入 `AIMemory/work.log`。
