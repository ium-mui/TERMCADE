# 发布流程

[English](../RELEASING.md) · [한국어](../ko/RELEASING.md) · [简体中文](RELEASING.md)

TERMCADE 通过 GitHub Releases 分发可执行文件压缩包。`.github/workflows/release.yml` 从 `main` 构建发布版本。

## 自动流程

```text
main 上的 Conventional Commits
  → Release Please 准备版本 PR
  → 审阅并合并版本 PR
  → GitHub 创建 vX.Y.Z Release
  → CI 构建五个平台压缩包和 SHA256SUMS
```

Release Please 会更新 `Cargo.toml`、`Cargo.lock`、`CHANGELOG.md` 和 `.release-please-manifest.json`。

| 提交 | 版本变化 |
| --- | --- |
| `fix:` | Patch |
| `feat:` | Minor |
| `type!:` 或 `BREAKING CHANGE:` | Major |
| `docs:`、`test:`、`chore:` | 单独出现时不改变版本 |

## 仓库设置

`Release` 工作流需要 `contents: write` 权限来创建标签、Release 和文件。若要自动创建版本 PR，请添加名为 `RELEASE_PLEASE_TOKEN` 的专用 Actions secret，并授予 contents、issues 和 pull requests 写权限。没有该 secret 时，工作流仍可在格式正确的版本 PR 合并后发布 Release，但不会自动创建下一个版本 PR。

## 发布文件

| 平台 | Rust target | 压缩格式 |
| --- | --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

每个压缩包包含可执行文件、项目 README 和 `LICENSE`。`SHA256SUMS` 包含所有压缩包的校验和。

## 发布检查

合并版本 PR 前：

- 确认 CI 通过。
- 检查生成的版本和 changelog。
- 必要时确认用户文档已反映本次发布。

合并后：

- 确认 `vX.Y.Z` Release 已创建。
- 确认五个压缩包和 `SHA256SUMS` 已附加。
- 下载一个压缩包，验证校验和并运行 `tcade --version`。

构建或上传失败时，修复工作流后重新运行失败的任务。不要移动已有版本标签；已发布源码需要修改时应发布 patch 版本。
