# Issue、分支、提交与 Pull Request 工作流

[English](../GIT_WORKFLOW.md) · [한국어](../ko/GIT_WORKFLOW.md) · [简体中文](GIT_WORKFLOW.md)

本文是 TERMCADE 协作的规范流程。小型文档修正可以不预先创建 Issue，但所有代码变更仍必须通过 Pull Request。

## 1. Issue

创建前搜索开放和已关闭的 Issue，并选择正确表单：

- **Bug report**：可复现的错误行为
- **Feature request**：新游戏或行为变更
- **Documentation issue**：缺失、错误或未翻译的文档
- **Support request**：明确的使用或开发问题

可供分类的 Issue 应包含用户问题、复现步骤或验收标准、受影响版本和相关环境。维护者使用以下标签：

| 标签组 | 值 | 含义 |
| --- | --- | --- |
| 类型 | `bug`, `enhancement`, `documentation`, `support`, `dependencies` | 工作性质 |
| 状态 | `needs-triage`, `needs-info`, `accepted`, `blocked` | 当前决策状态 |
| 优先级 | `priority: critical`, `priority: high`, `priority: normal`, `priority: low` | 排序信号，不是交付承诺 |
| 难度 | `good first issue`, `help wanted` | 贡献者参与准备度 |
| 区域 | `area: game`, `area: ui`, `area: persistence`, `area: ci`, `area: i18n` | 主要负责区域 |

安全问题不得出现在公开 Issue 中；请遵循[安全政策](SECURITY.md)。

## 2. 分支

`main` 必须始终可发布，并且只能通过 PR 修改。从最新 `main` 创建短期分支：

```bash
git switch main
git pull --ff-only
git switch -c feat/123-memory-timer
```

使用小写 kebab-case 和以下前缀：

| 前缀 | 用途 |
| --- | --- |
| `feat/` | 面向用户的新功能 |
| `fix/` | Bug 修复 |
| `docs/` | 仅文档或翻译 |
| `refactor/` | 不改变行为的内部修改 |
| `test/` | 仅测试 |
| `perf/` | 性能改进 |
| `build/` | 构建或依赖变更 |
| `ci/` | 自动化变更 |
| `chore/` | 不适合更具体类型的维护工作 |
| `release/` | 仅供自动发布 PR 使用 |

有 Issue 时加入编号，如 `fix/42-history-corruption`。合并后删除分支。长期开发分支和个人分支不属于官方流程。

## 3. 提交

提交标题遵循 Conventional Commits：

```text
<type>[optional scope][!]: <imperative summary>

[optional body]

[optional footers]
```

允许的类型为 `feat`、`fix`、`docs`、`refactor`、`test`、`perf`、`build`、`ci`、`chore` 和 `revert`。有助于理解时使用稳定的 scope，如 `ui`、`history`、`casino`、`snake`、`docs` 或 `release`。

```text
feat(maze): add deterministic abyss stage
fix(history): preserve wallet after a failed write
docs(zh-CN): clarify the release checklist
refactor(session)!: replace per-game options with one session enum
```

规则：

- 标题使用英文祈使句，不加句号。
- 每次提交聚焦一个目的，在正文说明原因和取舍。
- 使用 `Refs: #123` 引用，或在 PR 中使用 `Closes #123` 关闭 Issue。
- 不兼容变更使用 `!` 和 `BREAKING CHANGE:` footer。
- 不得包含秘密、生成的二进制、无关格式化或私人数据。

Release Please 将 `fix` 映射为 patch，`feat` 映射为 minor，breaking change 映射为 major。其他类型会进入历史，但通常不会单独触发版本提升。

## 4. Pull Request

尽早打开 draft 获取反馈，满足清单后再标记 ready。由于 squash merge 会使用 PR 标题作为最终提交标题，PR 标题本身必须是有效的 Conventional Commit。

每个 PR 必须：

1. 只处理一个逻辑变更，并在适用时关联 Issue。
2. 解释行为和动机，而不只是列出文件。
3. 提供与风险相称的测试和准确验证命令。
4. 面向用户的行为变化必须更新英文文档和支持的翻译。
5. 通过 CI 的 `branch-name`、`pr-title`、`quality` 和 `msrv` 检查。
6. 解决所有评审对话。有其他维护者时至少获得一次独立批准；单人维护阶段则在 PR 中记录自我评审，并通过全部必需检查。
7. 与最新 `main` 无冲突。

普通贡献使用 **squash merge**。只有在保留精心整理的提交序列确有价值时才使用 **rebase merge**。禁用 merge commit。合并者必须确保最终标题是 Conventional Commit。

## 5. 受保护的 `main`

仓库管理员应为 `main` 配置 ruleset：

- 始终必须通过 PR。只有一名维护者时所需批准数设为 0；出现独立评审者后立即提高为 1。
- 启用批准要求后，新提交会撤销旧批准。
- 建立有效 CODEOWNERS 团队后要求 code owner 评审
- 必需检查：`CI` workflow 的 `branch-name`、`pr-title`、`quality` 和 `msrv`
- 必须解决评审对话
- 禁止 force push 和删除分支
- 要求 linear history
- 管理员也受规则约束，只有应急维护者可以绕过
- 禁止直接 push

单人维护例外也不允许绕过 CI。紧急绕过必须记录在后续 Issue 中并事后审查，不能只为跳过 CI 等待而使用。
