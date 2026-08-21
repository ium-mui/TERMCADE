# TERMCADE 架构

[English](../ARCHITECTURE.md) · [한국어](../ko/ARCHITECTURE.md) · [简体中文](ARCHITECTURE.md)

本文定义了 TERMCADE 在增加游戏和功能时必须保持的边界。游戏规则、应用流程、终端输入输出、渲染和持久化必须彼此分离。

## 依赖方向

```text
main
 └─ runtime ── terminal / input / cleanup
      ├─ App ── navigation / input dispatch / result orchestration
      │   ├─ GameCatalog
      │   ├─ GameSession ── one active game session
      │   ├─ CasinoState ── wager and card-hand phases
      │   └─ HistoryStore
      └─ Renderer ── read-only App projection

game modules ── rules, seeded randomness, RoundResult
```

依赖只能向下。单个游戏模块不得了解 `crossterm`、`ui` 或真实文件存储。渲染器只读取 `App` 状态，不修改状态。

## 模块职责

| 模块 | 职责 |
| --- | --- |
| `runtime.rs` | raw mode 和 alternate screen 生命周期、事件循环、错误后的终端恢复 |
| `app.rs` | 页面切换、按键分发、游戏开始和结束、结果与钱包协调 |
| `game_session.rs` | 唯一活动游戏会话及通用生命周期 |
| `casino.rs` | 下注输入、总下注、翻牌前阶段 |
| `domain.rs` | 游戏和关卡 ID、定义、注册与验证 |
| `history.rs` | 历史和钱包接口，以及 JSON 和内存实现 |
| `round.rs` | 通用回合结果与心算回合引擎 |
| `ui.rs` | 固定尺寸缓冲区渲染与终端差异输出 |
| 游戏模块 | 单个游戏的规则、状态、种子随机性和结果计算 |

## 必须保持的不变量

1. `App` 最多只有一个活动 `GameSession`，不得重新增加各游戏独立的 `Option` 字段。
2. 纸牌游戏只能按 `Covered → Betting → Playing` 顺序前进，不得用可任意组合的布尔值表示阶段。
3. 只有在 `HistoryStore` 成功持久化交易后才能修改内存钱包。
4. UI 不修改状态；`App` 处理输入，游戏会话处理规则。
5. 随机游戏提供种子构造路径，以便测试可复现。
6. `GameCatalog::try_from_modules` 或 `try_register` 在开始游戏前拒绝注册错误。
7. `TerminalSession` 是终端模式生命周期的唯一所有者。

## 主要执行流程

```text
KeyEvent
  → App::handle_key
  → screen/game-kind dispatch
  → game session mutation
  → optional RoundResult
  → HistoryStore persistence
  → Renderer::draw (read-only)
```

计时游戏由 `App::on_tick` 将 tick 转发给 `GameSession::tick`。事件循环周期和终端 API 不得进入游戏模块。

## 可靠性边界

- 历史 JSON 先写临时文件并同步，再移动到目标路径。
- 损坏的历史文件会返回错误，绝不静默覆盖。
- `TerminalSession` 在正常退出、提前错误和 unwind 后都会尝试恢复终端。
- `Renderer::draw_at` 可在没有真实 TTY 时测试不同尺寸的布局。
- 目录会拒绝空 ID、重复游戏或关卡 ID，以及不匹配的游戏类型。

## 决定代码位置

当相同状态转换在两个以上游戏模块中重复时，应提升为共享类型。仅服务一个游戏的规则保留在该模块中。如果 `App` 开始计算游戏规则，或 `ui.rs` 开始解释输入，就说明越过了职责边界。
