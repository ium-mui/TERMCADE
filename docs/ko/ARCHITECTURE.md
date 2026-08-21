# TERMCADE 아키텍처

[English](../ARCHITECTURE.md) · [한국어](ARCHITECTURE.md) · [简体中文](../zh-CN/ARCHITECTURE.md)

이 문서는 새 게임과 기능이 늘어나도 TERMCADE를 유지보수할 수 있게 하는 경계를 정의합니다. 게임 규칙, 애플리케이션 흐름, 터미널 입출력, 렌더링, 영속화는 서로 분리해야 합니다.

## 의존성 방향

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

의존성은 아래 방향으로만 향합니다. 개별 게임 모듈은 `crossterm`, `ui`, 실제 파일 저장소를 알면 안 됩니다. 렌더러는 `App` 상태를 읽지만 변경하지 않습니다.

## 모듈 책임

| 모듈 | 책임 |
| --- | --- |
| `runtime.rs` | raw mode와 alternate screen 수명, 이벤트 루프, 오류 시 터미널 복구 |
| `app.rs` | 화면 전환, 키 라우팅, 게임 시작과 종료, 결과와 지갑 조율 |
| `game_session.rs` | 하나의 활성 게임 세션과 공통 생명주기 |
| `casino.rs` | 베팅 입력, 총 베팅, 카드 공개 전 단계 |
| `domain.rs` | 게임·단계 ID와 정의, 등록, 검증 |
| `history.rs` | 기록·지갑 인터페이스와 JSON·메모리 구현 |
| `round.rs` | 공통 라운드 결과와 암산 라운드 엔진 |
| `ui.rs` | 고정 크기 버퍼 렌더링과 터미널 diff 출력 |
| 게임 모듈 | 한 게임의 규칙, 상태, seeded randomness, 결과 계산 |

## 반드시 유지할 불변식

1. `App`에는 최대 하나의 `GameSession`만 활성화됩니다. 게임별 `Option` 필드를 따로 추가하지 않습니다.
2. 카드 게임은 `Covered → Betting → Playing` 순서로만 진행합니다. 조합 가능한 불리언으로 단계를 표현하지 않습니다.
3. `HistoryStore`가 거래를 저장한 뒤에만 메모리 지갑 상태를 바꿉니다.
4. UI는 상태를 변경하지 않습니다. `App`이 입력을, 게임 세션이 규칙을 처리합니다.
5. 랜덤 게임은 테스트 재현을 위한 seeded 생성 경로를 제공합니다.
6. `GameCatalog::try_from_modules` 또는 `try_register`가 플레이 전에 등록 오류를 거부합니다.
7. 터미널 모드 수명은 `TerminalSession`만 소유합니다.

## 주요 실행 흐름

```text
KeyEvent
  → App::handle_key
  → screen/game-kind dispatch
  → game session mutation
  → optional RoundResult
  → HistoryStore persistence
  → Renderer::draw (read-only)
```

시간 기반 게임은 `App::on_tick`에서 `GameSession::tick`으로 tick을 전달합니다. 이벤트 루프 주기와 터미널 API는 게임 모듈 내부에 두지 않습니다.

## 안정성 경계

- 기록 JSON은 임시 파일에 쓰고 동기화한 뒤 대상 경로로 이동합니다.
- 손상된 기록 파일은 오류를 반환하며 조용히 덮어쓰지 않습니다.
- `TerminalSession`은 정상 종료, 조기 오류, unwind 후에 터미널 복구를 시도합니다.
- `Renderer::draw_at`으로 실제 TTY 없이 여러 크기의 레이아웃을 검사할 수 있습니다.
- 카탈로그는 빈 ID, 중복 게임·단계 ID, 일치하지 않는 게임 종류를 거부합니다.

## 코드 위치 결정

두 개 이상의 게임 모듈에서 같은 상태 전이가 반복되면 공통 타입으로 올립니다. 한 게임에만 쓰는 규칙은 그 모듈에 둡니다. `App`이 게임 규칙을 계산하거나 `ui.rs`가 입력을 해석하기 시작하면 책임 경계를 넘은 것입니다.
