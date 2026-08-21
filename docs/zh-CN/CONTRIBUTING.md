# 为 TERMCADE 做贡献

[English](../../CONTRIBUTING.md) · [한국어](../ko/CONTRIBUTING.md) · [简体中文](CONTRIBUTING.md)

欢迎贡献游戏、测试和文档。

## 开始之前

- 搜索现有 Issue 和 PR。
- 添加新游戏、更改存档兼容性或进行大型架构修改前，先创建 Issue 讨论。
- 小型修复和文档更正可以直接提交 PR。

## 开发流程

从最新 `main` 创建目标明确的分支：

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

需要 Rust 1.85 或更高版本。提交前运行与 CI 相同的检查：

```bash
./scripts/check.sh
```

架构、测试、存档和新增游戏流程请参阅[开发指南](DEVELOPMENT.md)。

## 修改要求

- 将游戏规则与终端渲染分离。
- 为新行为和缺陷修复添加测试。
- 保持基于种子的游戏行为可复现。
- 保持现有历史文件兼容，或提供明确的迁移。
- 不要加入 `unsafe` 代码、机密、生成的二进制文件或无关修改。
- 行为变化时更新用户文档。

## 分支、提交和 PR

遵循 [Git 工作流](GIT_WORKFLOW.md)。核心规则如下：

- 使用 `feat/new-game`、`fix/history-write`、`docs/update-controls` 这类带前缀的分支名。
- 使用 `feat:`、`fix:`、`docs:` 等 Conventional Commit 类型和清晰的摘要。
- PR 标题会成为 squash commit，因此必须使用 Conventional Commit 格式。
- 说明修改内容、原因和验证方式。
- 每个 PR 只处理一个逻辑修改。

## 合并要求

必需的 CI 检查必须通过，审阅对话必须解决。PR 通常使用 squash merge。提交的工作采用仓库的 [MIT 许可证](../../LICENSE)，无需单独签署 CLA。
