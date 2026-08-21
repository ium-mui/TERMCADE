# 文档与翻译政策

[English](../TRANSLATIONS.md) · [한국어](../ko/TRANSLATIONS.md) · [简体中文](TRANSLATIONS.md)

TERMCADE 文档以英文为规范，目前支持韩文（`ko`）和简体中文（`zh-CN`）。目录结构允许未来增加语言而无需移动英文文件。

## 规范文件结构

```text
README.md                  英文项目 README
README.ko.md               韩文项目 README
README.zh-CN.md            简体中文项目 README
docs/<NAME>.md             英文规范文档
docs/ko/<NAME>.md          韩文翻译
docs/zh-CN/<NAME>.md       简体中文翻译
<COMMUNITY>.md             英文规范社区政策
docs/<locale>/<COMMUNITY>.md  参考用政策翻译
```

翻译与英文冲突时以英文为准。该规则只用于解决歧义，并不意味着可以放任翻译过期。

## 更新文档

1. 先编辑英文，确定行为、命令名称和链接。
2. 在同一个 PR 中更新韩文和简体中文。
3. 保留标题、代码、标识符、命令输出、表格、警告和链接目标。
4. 自然翻译说明，但不要翻译 Rust 名称、CLI subcommand、按键名、路径和版本字符串。
5. 更新每个本地化页面顶部的语言导航。
6. 运行 `./scripts/check.sh`；`scripts/check-docs.sh` 会拒绝缺少语言文件的变更。

无法更新翻译时，贡献者必须创建并关联 `area: i18n` 后续 Issue。维护者根据用户影响决定是否允许暂时差异。

## 添加语言

使用有效 BCP 47 标签，如 `ja` 或 `pt-BR`。

1. 创建 Issue，定义语言代码、书面语言名称、首位翻译者和维护计划。
2. 添加 `README.<locale>.md`。
3. 在 `docs/<locale>/` 添加 `scripts/check-docs.sh` 列出的所有文件。
4. 在所有 README 和文档语言导航中添加该语言。
5. 扩展 `scripts/check-docs.sh`，让 CI 要求新语言。
6. 从最新英文规范翻译，不从其他翻译转译。
7. 由流利使用者和熟悉技术内容的维护者评审。

## 翻译风格

- 保持原意和确定性，不添加英文中不存在的承诺。
- 使用目标社区通行的软件术语。
- 示例必须可执行，代码内标点保持不变。
- 使用易懂且不预设性别的语言。
- 产品名统一保留为 `TERMCADE`。
- 可能影响后续文档的术语决定应记录在翻译 PR 中。

机器翻译可作为草稿，但合并前必须由人工检查技术准确性、流畅度、命令、链接和安全措辞。
