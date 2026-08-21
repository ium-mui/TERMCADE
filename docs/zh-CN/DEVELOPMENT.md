# 开发指南

[English](../DEVELOPMENT.md) · [한국어](../ko/DEVELOPMENT.md) · [简体中文](DEVELOPMENT.md)

## 前置条件

- Rust 1.85 或更高版本
- Git
- 支持 ANSI 转义序列的终端

克隆仓库并执行完整验证：

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
./scripts/check.sh
```

该脚本执行与 CI 相同的翻译文件检查、格式检查、Clippy 和测试。迭代时可以运行更小范围的命令：

```bash
cargo test casino
cargo test --test app_flows
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

## 添加游戏

1. 在 `src/<game>.rs` 中创建不了解 UI 和终端 API 的纯会话类型。
2. 向 `GameKind` 添加类型，并通过 `GameModule::definition` 定义 ID 和关卡。
3. 在 `GameCatalog::default` 注册模块，不得绕过目录验证。
4. 向 `GameSession` 添加 variant、构造路径、访问器、`tick` 和 `finish_abandoned` 路由。
5. 在 `App::handle_playing` 连接按键，把规则计算留在会话中。
6. 在 `ui.rs` 使用公开的只读访问器添加页面和操作提示。
7. 扩展 `tests/app_flows.rs` 的默认关卡表和核心用户流程测试。
8. 更新受影响的受支持语言游戏文档，然后运行 `./scripts/check.sh`。

## 游戏会话契约

- 构造后状态立即有效。
- 相同种子和输入产生相同状态转换。
- 已结束的会话不会再次生成结果或奖励。
- `finish_abandoned` 对未结束会话最多返回一次结果。
- 时间来自注入的时钟或显式 tick，不在游戏逻辑中任意读取。
- 模块测试固定分数、尝试次数和准确率的含义。

## 测试策略

`tests/support/AppHarness` 无需打开终端即可向真实 `App` 发送按键，用于菜单、下注、重新开始等用户流程。只测试内部方法可能遗漏按键映射和页面切换回归。

渲染测试向 `Renderer::draw_at` 传入固定尺寸和 `Vec<u8>`，覆盖低于最小尺寸、`64×24` 和宽终端。布局计算使用饱和运算，小屏幕也不得 panic。

游戏规则测试放在对应模块。需要断言随机结果时使用种子构造器。

## 持久化变更

`history.json` 是用户数据。新增字段优先使用 `#[serde(default)]` 以读取旧文件。不兼容变更必须先提高 `HISTORY_SCHEMA_VERSION` 并实现显式迁移。不得静默重置或覆盖损坏文件。

## 文档变更

英语、韩语和简体中文是平行的文档版本。可以从任一受支持语言开始修改，并尽量同步相关语言版本；需要时在 PR 中请求语言审阅。遵循[语言支持指南](TRANSLATIONS.md)，CI 会检查每种语言的文件集合。

## 完成标准

- 核心行为有回归测试。
- 小终端渲染不会 panic。
- 持久化失败不会显示虚假成功，也不会修改内存钱包。
- 新路由通过 `GameSession` 和目录验证路径。
- `./scripts/check.sh` 成功。
- 用户行为、架构和相关翻译已更新。
- 分支、提交和 PR 遵循 [Git 工作流](GIT_WORKFLOW.md)。
