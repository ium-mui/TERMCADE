# Git 工作流

[English](../GIT_WORKFLOW.md) · [한국어](../ko/GIT_WORKFLOW.md) · [简体中文](GIT_WORKFLOW.md)

这些规则只用于让 TERMCADE 的开发修改更容易审阅和发布。

## Issue

需要讨论或跟踪的修改应创建 Issue：

- **Bug report**：可复现的错误行为
- **Feature request**：新游戏或新行为
- **Documentation issue**：缺失、错误或尚未翻译的内容

小型且明确的修改可以直接提交 PR。Bug 应包含复现步骤，功能请求应说明期望结果。只有安全相关 Bug 才需要遵循简短的[安全说明](SECURITY.md)。

## 分支

从最新 `main` 创建短期分支：

```bash
git switch main
git pull --ff-only
git switch -c feat/new-game
```

分支名会被自动化工具读取，因此使用小写 ASCII kebab-case。

| 前缀 | 用途 |
| --- | --- |
| `feat/` | 新行为 |
| `fix/` | Bug 修复 |
| `docs/` | 文档或语言工作 |
| `refactor/` | 内部重构 |
| `test/` | 仅测试修改 |
| `build/` | 依赖或打包 |
| `ci/` | GitHub Actions 和自动化 |
| `chore/` | 其他维护 |

需要时加入 Issue 编号，例如 `fix/42-history-write`。合并后删除分支。

## 提交

使用 Conventional Commits：

```text
<type>[optional scope][!]: <summary>
```

支持的类型为 `feat`、`fix`、`docs`、`refactor`、`test`、`perf`、`build`、`ci`、`chore` 和 `revert`。发布工具读取的类型和可选 scope 保持 ASCII；摘要可以用英语、韩语或简体中文清楚书写。

```text
feat(maze): add deterministic abyss stage
fix(history): 기록 저장 실패 시 지갑 상태 유지
docs(zh-CN): 补充扫雷操作说明
```

不兼容修改使用 `!` 或 `BREAKING CHANGE:` footer。不要提交机密、生成的发布压缩包或无关格式化。

## PR

- 每个 PR 只包含一个逻辑修改。
- PR 标题会成为 squash commit，因此使用 Conventional Commit 格式。
- 说明目的、重要实现选择和验证方式。
- 为修改的行为添加测试，并更新用户文档的相关语言版本。
- 在本地运行 `./scripts/check.sh`。
- 解决审阅对话；`main` 更新后同步分支。

PR 描述和审阅讨论可以使用任一受支持的文档语言。代码标识符和命令应保持准确，便于跨语言验证修改。

## 受保护的 `main`

修改通过 PR 进入 `main`。必需检查为 `branch-name`、`pr-title`、`quality` 和 `msrv`。分支必须基于最新 `main`，审阅对话必须解决。仓库要求线性历史，并阻止强制推送和删除分支。普通 PR 使用 squash merge。
