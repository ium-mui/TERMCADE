<div align="center">

<img src="assets/termcade-logo.png" alt="TERMCADE" width="900">

### 터미널 안에 들어온 작은 아케이드

Rust와 가벼운 `crossterm` 렌더러로 만든 터미널 게임 모음입니다.
짧게 한 판 즐기고, 플레이 기록과 공용 게임 지갑을 쌓아보세요.

[English](README.md) · [한국어](README.ko.md) · [简体中文](README.zh-CN.md)

[![CI](https://github.com/ium-mui/TERMCADE/actions/workflows/ci.yml/badge.svg)](https://github.com/ium-mui/TERMCADE/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/ium-mui/TERMCADE)](https://github.com/ium-mui/TERMCADE/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[설치](#설치) · [게임](#게임) · [조작법](#조작법) · [문서](#문서) · [기여](#기여)

</div>

## 설치

[최신 GitHub Release](https://github.com/ium-mui/TERMCADE/releases/latest)에서 운영체제에 맞는 압축 파일을 내려받아 풀고 `tcade` 또는 `tcade.exe`를 `PATH`에 둡니다. 모든 릴리스는 SHA-256 체크섬을 제공합니다.

지원하는 릴리스 대상은 다음과 같습니다.

- Linux x86-64 및 ARM64
- macOS Intel 및 Apple Silicon
- Windows x86-64

Rust 1.85 이상이 설치되어 있다면 GitHub 소스에서 바로 빌드할 수 있습니다.

```bash
cargo install --git https://github.com/ium-mui/TERMCADE.git --locked
tcade
```

로컬 저장소에서 작업하려면 다음을 실행합니다.

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
cargo install --path . --locked
tcade
```

개발 중 설치 없이 실행할 수도 있습니다.

```bash
cargo run --
cargo run -- math
cargo run -- snake classic-1
```

## 게임

| 게임 | 예시 명령 | 목표 |
| --- | --- | --- |
| 암산 | `math` | 덧셈·뺄셈·곱셈·나눗셈 문제를 빠르게 풀기 |
| 스네이크 | `snake classic-1` | 벽과 자신의 몸을 피해 오래 살아남기 |
| 틱택토 | `tictactoe classic-1` | 3×3 보드에서 컴퓨터 이기기 |
| 2048 | `2048 classic-1` | 타일을 합쳐 2048 만들기 |
| 스도쿠 | `sudoku classic-1` | 유일한 해답이 있는 숫자 퍼즐 완성하기 |
| 지뢰찾기 | `minesweeper beginner` | 지뢰를 피해 모든 안전 칸 열기 |
| 커넥트 포 | `connect-four easy` | 난이도별 AI보다 먼저 디스크 네 개 연결하기 |
| 카드 짝맞추기 | `memory small` | 카드 위치를 기억해 모든 쌍 찾기 |
| 미로 탈출 | `maze alley` | 새로 생성된 미로에서 출구 찾기 |
| 카지노 | `gambling` | 블랙잭·룰렛·슬롯·AI 홀덤·타자 채굴 즐기기 |
| 벽돌깨기 | `breakout classic-1` | 공을 받아 모든 벽돌 부수기 |

`tcade`를 실행해 게임과 단계를 선택하거나 명령으로 바로 지정할 수 있습니다.

```bash
tcade gambling blackjack-1
tcade gambling holdem-1
tcade minesweeper expert
tcade connect-four hard
tcade memory grand
tcade maze abyss
tcade snake turbo
tcade sudoku hard
```

## 조작법

- 방향키 또는 지원하는 게임에서 `W` `A` `S` `D`: 이동
- `Enter`: 선택, 제출, 공개 또는 기본 동작
- `Space`: 게임별 보조 동작
- `Backspace`: 스도쿠 숫자 지우기
- `Esc` 또는 `Q`: 뒤로 가기 또는 게임 나가기
- `Ctrl+C`: 즉시 종료

지뢰찾기는 `F`로 깃발을 세웁니다. 커넥트 포는 좌우로 열을 고르고, 카드 짝맞추기는 `Enter`로 카드를 공개하고 틀린 쌍을 확인합니다. 카지노별 베팅과 행동 키는 플레이 화면에 표시됩니다.

## 데이터와 터미널 요구 사항

결과, 기록, 지갑 잔액은 운영체제의 애플리케이션 데이터 디렉터리에 JSON으로 저장됩니다. TERMCADE는 손상된 기록 파일을 조용히 덮어쓰지 않습니다. 상세 보드가 잘리지 않도록 최소 `64×24` 크기의 터미널을 권장합니다.

## 문서

- [문서 목차](docs/ko/README.md)
- [게임과 단계](docs/ko/GAMES.md)
- [아키텍처](docs/ko/ARCHITECTURE.md)
- [개발 가이드](docs/ko/DEVELOPMENT.md)
- [이슈·브랜치·커밋·PR 규칙](docs/ko/GIT_WORKFLOW.md)
- [릴리스 및 배포 절차](docs/ko/RELEASING.md)
- [번역 정책](docs/ko/TRANSLATIONS.md)

## 기여

모든 기여를 환영합니다. [기여 가이드](docs/ko/CONTRIBUTING.md)를 먼저 읽고, 큰 변경은 이슈에서 논의한 뒤 PR을 열기 전에 `./scripts/check.sh`를 실행해 주세요. 모든 참여에는 [행동 강령](docs/ko/CODE_OF_CONDUCT.md)이 적용됩니다.

도움이 필요하면 [지원 안내](docs/ko/SUPPORT.md)를, 보안 취약점을 발견했다면 [보안 정책](docs/ko/SECURITY.md)을 확인하세요.

## 라이선스

TERMCADE는 [MIT 라이선스](LICENSE)로 제공됩니다.
