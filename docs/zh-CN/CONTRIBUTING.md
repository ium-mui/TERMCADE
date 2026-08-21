# 参与 TERMCADE 贡献

[English](../../CONTRIBUTING.md) · [한국어](../ko/CONTRIBUTING.md) · [简体中文](CONTRIBUTING.md)

感谢帮助改进 TERMCADE。我们欢迎代码、测试、文档、翻译、Bug 报告和设计反馈。

参与即表示同意遵守[行为准则](CODE_OF_CONDUCT.md)。漏洞不得使用公开 Issue，请遵循[安全政策](SECURITY.md)的私密流程。

## 开始前

1. 搜索现有 Issue 和 PR。
2. 实现新游戏、依赖、持久化格式、公共接口或大型架构变更前先创建 feature request。
3. 大量投入前等待 `accepted` 决定。讨论会确定范围，但不保证合并。
4. 意图明确的小型 Bug 修复、测试和文档修正可直接提交 PR。

## 开发流程

Fork 仓库，从最新 `main` 创建聚焦的分支：

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

需要 Rust 1.85 或更高版本。提交前运行完整检查：

```bash
./scripts/check.sh
```

架构、测试、持久化安全和添加游戏的流程见[开发指南](DEVELOPMENT.md)。

## 变更要求

- 保持[架构](ARCHITECTURE.md)规定的边界。
- 修复要有回归测试，功能要有行为测试。
- 通过种子构造路径让随机行为可复现。
- 将历史文件视为持久用户数据并保持向后兼容。
- 不引入 `unsafe` 代码。
- 面向用户的变更应更新英文、韩文和简体中文文档。
- 不要混入无关格式化、依赖升级、生成产物和顺手重构。

## 提交与 PR

完整规则见 [Git 工作流](GIT_WORKFLOW.md)：

- 使用 `feat/123-new-game` 形式的前缀和 kebab-case 分支。
- 使用英文 Conventional Commit，如 `fix(ui): avoid clipping narrow boards`。
- PR 标题会成为 squash commit，因此也必须符合 Conventional Commit。
- 解释问题、方案、风险和准确的验证步骤。
- PR 完全解决 Issue 时使用 `Closes #123`。
- 处理评审意见后再 resolve 对话。

维护者可能要求缩小范围、增加测试和文档或先做设计讨论。与项目方向冲突、维护成本不成比例、缺少安全迁移或无法可靠支持的贡献可能被拒绝。

## 文档与翻译

英文为规范来源。本地化文件位于 `docs/ko` 和 `docs/zh-CN`，项目 README 翻译位于仓库根目录。请遵循[翻译政策](TRANSLATIONS.md)。

翻译 PR 应说明核对的英文 revision，准确保留代码和链接，并接受语言质量和技术准确性两方面评审。

## 评审与合并

所有必需 CI 必须通过，所有评审对话都必须解决。有其他维护者时需要一次独立批准。单人维护阶段应在 PR 中记录自我评审，并且只有全部必需检查通过后才能合并。通常使用 squash merge；维护者可以调整最终标题，让 Release Please 正确计算版本。

贡献者保留作品版权，并将提交的贡献以仓库 [MIT License](../../LICENSE)授权。不要求单独的 CLA。
