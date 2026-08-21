# 发布与部署流程

[English](../RELEASING.md) · [한국어](../ko/RELEASING.md) · [简体中文](RELEASING.md)

TERMCADE 是 CLI 应用，因此部署指将带版本的可执行文件压缩包发布到 GitHub Releases。`main` 是唯一发布来源。

## 发布模型

每次 `main` 变化都会运行 `Release` workflow：

```text
main 上的 Conventional Commits
  → Release Please 更新一个发布 PR
  → 维护者评审并合并发布 PR
  → 创建 vX.Y.Z 标签和 GitHub Release
  → 为五个目标构建原生二进制
  → 附加压缩包和 SHA256SUMS
```

Release Please 更新 `Cargo.toml`、`Cargo.lock`、`CHANGELOG.md` 和 `.release-please-manifest.json`。版本遵循 Semantic Versioning：

| 提交 | 版本影响 | 示例 |
| --- | --- | --- |
| `fix:` | Patch | `1.2.3 → 1.2.4` |
| `feat:` | Minor | `1.2.3 → 1.3.0` |
| `type!:` 或 `BREAKING CHANGE:` | Major | `1.2.3 → 2.0.0` |
| `docs:`, `test:`, `chore:` | 单独不提升 | 有其他可发布变更时包含 |

没有记录明确理由时，不要手动编辑自动发布 PR 的版本或 changelog。

## 仓库一次性设置

1. 在 **Settings → Actions → General → Workflow permissions** 启用 GitHub Actions 并允许 Actions 创建 PR。
2. 创建具有仓库 `Contents: write`、`Issues: write`、`Pull requests: write` 和 `Workflows: write` 权限的 fine-grained bot token。
3. 将其保存为 Actions secret `RELEASE_PLEASE_TOKEN`。
4. 按 [Git 工作流](GIT_WORKFLOW.md)配置 `main` 保护规则。
5. 创建 Git 文档列出的标签；若自动化未创建，再添加 `autorelease: pending`、`autorelease: tagged` 和 `autorelease: snapshot`。

workflow 会回退到 `GITHUB_TOKEN`，但该令牌创建的 PR 不会触发其他 workflow，因此完全自动的发布 PR CI 需要专用令牌。

## 支持的产物

| Runner | Rust target | 压缩格式 |
| --- | --- | --- |
| Ubuntu x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Ubuntu ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

每个压缩包包含 `tcade` 可执行文件、英文 README 和许可证。`SHA256SUMS` 校验所有压缩包。

## 维护者发布清单

合并发布 PR 前：

- 确认包含的所有 PR 已通过 CI 和评审。
- 从用户视角阅读生成的 changelog。
- 确认 `Cargo.toml` 与 manifest 版本一致。
- 确认 breaking change 和迁移清晰可见。
- 确认文档和翻译覆盖本次发布。
- 使用生成的 Conventional Commit 标题合并。

合并后：

- 确认 `vX.Y.Z` 标签指向 `main` 上的发布提交。
- 确认五个压缩包和 `SHA256SUMS` 均已附加。
- 下载一个产物，验证校验和，运行 `tcade --version` 并 smoke test 一个游戏。
- 产物构建失败时重新运行失败 job；上传步骤会安全替换同一 Release 的产物。
- 所有产物就绪后再公告发布。

## 回滚与 hotfix

GitHub Release 和公开标签是不可重写的历史记录。不得通过移动或删除公开版本来隐藏问题。

普通缺陷通过 `fix:` PR 回退或修复，再合并新的发布 PR 生成 patch。安全缺陷按[安全政策](SECURITY.md)私下协调修复与 advisory，然后发布 patch。

若源码和标签正确但二进制损坏，重新运行产物 job 并验证校验和。若源码错误，应发布新 patch，而不是替换历史。

## 手动恢复

错过 GitHub 事件时可通过 `workflow_dispatch` 重新运行 Release Please，也可以在 Actions UI 重跑失败 job。手动标签或 Release 是最后手段，必须使用 `vX.Y.Z`、指向 `main` 中的提交、使用生成的 changelog，并提供相同的五个产物和校验和。
