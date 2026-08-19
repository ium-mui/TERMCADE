<div align="center">

<img src="assets/termcade-logo.png" alt="TERMCADE" width="900">

### 터미널 안에 들어온 작은 아케이드

Rust와 `ratatui`로 만든 한국어 TUI 게임 모음입니다.
짧게 한 판 즐기고, 기록과 지갑을 쌓아보세요.

[설치](#설치) · [게임](#게임) · [조작법](#조작법) · [개발](#개발)

</div>

## 설치

Rust가 설치되어 있다면 GitHub에서 바로 설치할 수 있습니다.

```bash
cargo install --git https://github.com/ium-mui/TERMCADE.git
tcade
```

소스 코드를 내려받아 로컬에서 설치하려면:

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
cargo install --path .
tcade
```

설치하지 않고 개발 모드로 실행할 때는 `cargo run`을 사용합니다.

```bash
cargo run --
cargo run -- math
cargo run -- snake classic-1
```

## 게임

| 게임 | 명령어 | 내용 |
| --- | --- | --- |
| 암산 | `math` | 덧셈·뺄셈·곱셈·나눗셈 문제를 빠르게 풀기 |
| 스네이크 | `snake classic-1` | 벽과 자신의 몸을 피해 오래 살아남기 |
| 틱택토 | `tictactoe classic-1` | 3×3 보드에서 컴퓨터와 대결하기 |
| 2048 | `2048 classic-1` | 타일을 합쳐 2048 만들기 |
| 스도쿠 | `sudoku classic-1` | 숫자를 채워 퍼즐 완성하기 |
| 도박장 | `gambling` | 블랙잭·룰렛·슬롯·타자 채굴 즐기기 |
| 벽돌깨기 | `breakout classic-1` | 패들로 공을 받아 벽돌 부수기 |

게임을 실행하면 게임 선택 화면에서 원하는 게임과 스테이지를 고를 수 있습니다.
게임·스테이지를 명령어로 바로 지정할 수도 있습니다.

```bash
tcade
tcade math addition-1
tcade gambling blackjack-1
tcade gambling roulette-1
tcade gambling slots-1
tcade gambling typing-mine
tcade breakout classic-1
```

## 조작법

- `↑` `↓` `←` `→` 또는 게임에 따라 `W` `A` `S` `D`: 이동
- `Enter`: 선택·제출·실행
- `Space`: 게임별 보조 동작
- `Backspace`: 스도쿠 숫자 지우기
- `Esc` 또는 `Q`: 뒤로 가기·게임 종료
- `Ctrl+C`: 즉시 종료

암산은 숫자 키로 답을 입력하고 `Enter`로 제출합니다. 스도쿠는 칸을 선택한 뒤 숫자를 바로 입력합니다. 블랙잭에서는 `H`/`Enter`로 히트, `S`/`Space`로 스탠드합니다. 룰렛은 `1`·`2`·`3`으로 색을 고른 뒤 `Enter`로 돌립니다.

도박 게임은 1원 베팅 방식이며 당첨 배당금은 공용 지갑에 누적됩니다. 타자 채굴은 문장을 정확히 입력할 때마다 1원을 지급합니다.

## 기록과 지갑

플레이 결과, 전체 이력, 지갑 잔액은 운영체제별 앱 데이터 디렉터리의 JSON 파일에 저장됩니다. 오락실 플로어·캐비닛 입장·게임별 플레이 화면을 제공하며, 터미널이 너무 작으면 화면이 잘릴 수 있으므로 최소 `64×20` 크기를 권장합니다.

## 개발

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

새 게임은 `GameModule`을 구현해 `GameCatalog`에 등록하고, 새 스테이지는 `StageDefinition`으로 추가할 수 있습니다. 라운드 엔진과 TUI 화면은 게임별 생성 규칙과 분리되어 있습니다.

## 라이선스

MIT
