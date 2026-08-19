# CLI Game

Rust 2024와 `ratatui`로 만든 확장형 터미널 게임입니다. 표시 문구는 한국어이며 게임·스테이지 ID는 영문으로 유지합니다.

## 실행

```bash
cargo run
cargo run -- cli-game
cargo run -- cli-game math
cargo run -- cli-game math addition-1
cargo run -- cli-game snake classic-1
cargo run -- cli-game tictactoe classic-1
cargo run -- cli-game 2048 classic-1
cargo run -- cli-game sudoku classic-1
cargo run -- cli-game gambling
cargo run -- cli-game gambling blackjack-1
cargo run -- cli-game gambling roulette-1
cargo run -- cli-game gambling slots-1
cargo run -- cli-game gambling typing-mine
cargo run -- cli-game breakout classic-1
```

지원하는 게임은 암산(`math`), 스네이크(`snake`), 틱택토(`tictactoe`), 2048(`2048`), 스도쿠(`sudoku`), 도박장(`gambling`), 벽돌깨기(`breakout`)입니다. 도박장에는 블랙잭, 룰렛, 슬롯머신, 타자 채굴이 있습니다. 모든 게임은 게임 선택 화면에서 고를 수 있고, 위 명령어처럼 게임·스테이지를 직접 지정해 시작할 수도 있습니다. `--help`와 `--version`도 제공합니다.

암산은 숫자 키로 답을 입력하고 `Enter`로 제출합니다. 스네이크와 2048은 방향키 또는 `W/A/S/D`를 사용하고, 틱택토는 방향키로 칸을 선택한 뒤 `Enter`로 둡니다. 스도쿠는 방향키로 칸을 선택하고 숫자를 바로 입력하며 `Backspace`로 지웁니다. 블랙잭은 `Enter`/`H`로 히트, `S`/`Space`로 스탠드합니다. 룰렛은 `1/2/3`으로 색을 고르고 Enter로 돌리며, 약 1.8초 동안 휠 숫자가 실제로 회전한 뒤 결과 숫자·색상·배당을 표시합니다. 슬롯머신은 Enter로 레버를 당깁니다. 도박 게임은 1원 베팅이며 당첨 배당금이 공용 지갑에 들어옵니다. 한 판이 끝나면 결과와 배당을 보여준 뒤 자동으로 다음 판을 시작하고, `Esc` 또는 `q`로 도박장을 나갑니다. 잔액이 부족하면 다음 판을 시작하지 않고 종료 안내를 표시합니다. 타자 채굴은 문장을 정확히 입력할 때마다 1원을 지급하고, 틀리면 지급하지 않습니다. 벽돌깨기는 방향키 또는 `A/D`로 패들을 움직입니다. `Esc`는 중단, `Ctrl+C`는 종료입니다. 결과·전체 플레이 이력·지갑 잔액은 운영체제별 앱 데이터 디렉터리의 JSON 파일에 저장됩니다.

게임 화면은 보드와 상태 패널을 분리해 표시하며, 보드가 잘리지 않도록 터미널 최소 크기 44×20을 안내합니다.

## 개발 검증

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

새 게임은 `GameModule`을 구현해 `GameCatalog`에 등록하고, 새 스테이지는 `StageDefinition`으로 추가할 수 있습니다. 라운드 엔진과 TUI 화면은 게임별 생성 규칙에 직접 의존하지 않습니다.
