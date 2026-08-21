# 개발 가이드

[English](../DEVELOPMENT.md) · [한국어](DEVELOPMENT.md) · [简体中文](../zh-CN/DEVELOPMENT.md)

## 사전 요구 사항

- Rust 1.85 이상
- Git
- ANSI escape sequence를 지원하는 터미널

저장소를 복제하고 전체 검증을 실행합니다.

```bash
git clone https://github.com/ium-mui/TERMCADE.git
cd TERMCADE
./scripts/check.sh
```

이 스크립트는 CI와 동일한 문서 번역 파일 검사, 포맷, Clippy, 테스트를 수행합니다. 개발 중에는 필요한 명령만 실행할 수 있습니다.

```bash
cargo test casino
cargo test --test app_flows
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

## 새 게임 추가

1. `src/<game>.rs`에 UI와 터미널 API를 모르는 순수 세션 타입을 만듭니다.
2. `GameKind`에 종류를 추가하고 `GameModule::definition`으로 ID와 단계를 정의합니다.
3. `GameCatalog::default`에 모듈을 등록하며 카탈로그 검증을 우회하지 않습니다.
4. `GameSession`에 variant, 생성 경로, 접근자, `tick`, `finish_abandoned` 라우팅을 추가합니다.
5. `App::handle_playing`에서 키를 연결하되 규칙 계산은 세션에 둡니다.
6. `ui.rs`에 공개된 읽기 전용 접근자만 사용해 화면과 조작 안내를 추가합니다.
7. `tests/app_flows.rs`의 기본 단계 표와 핵심 사용자 흐름 테스트를 확장합니다.
8. 영향을 받는 지원 언어의 게임 문서를 갱신하고 `./scripts/check.sh`를 실행합니다.

## 게임 세션 계약

- 생성 직후 상태가 유효해야 합니다.
- 같은 seed와 입력은 같은 상태 전이를 만듭니다.
- 끝난 세션은 두 번째 결과나 보상을 생성하지 않습니다.
- `finish_abandoned`는 끝나지 않은 세션에 대해 최대 한 번 결과를 반환합니다.
- 시간은 게임 내부에서 임의로 읽지 않고 주입한 clock 또는 명시적 tick에서 받습니다.
- 점수, 시도 횟수, 정확도의 의미를 모듈 테스트로 고정합니다.

## 테스트 전략

`tests/support/AppHarness`는 터미널을 열지 않고 실제 `App`에 키를 보냅니다. 메뉴 이동, 베팅, 다시 플레이 같은 사용자 흐름에 사용합니다. 내부 메서드만 테스트하면 키 매핑과 화면 전환 회귀를 놓칠 수 있습니다.

렌더링 검사는 고정된 크기와 `Vec<u8>`를 `Renderer::draw_at`에 전달합니다. 지원 최소 크기 바로 아래, `64×24`, 넓은 터미널을 테스트합니다. 레이아웃 계산은 포화 연산을 사용하며 작은 화면에서도 panic이 없어야 합니다.

게임 규칙 테스트는 해당 모듈에 둡니다. 랜덤 결과를 단정할 때는 seeded 생성자를 사용합니다.

## 저장 형식 변경

`history.json`은 사용자 데이터입니다. 필드를 추가할 때는 `#[serde(default)]`를 우선 사용해 이전 파일을 읽을 수 있게 합니다. 호환되지 않는 변경은 `HISTORY_SCHEMA_VERSION`을 올리고 명시적 마이그레이션을 먼저 구현합니다. 손상된 파일을 조용히 초기화하거나 덮어쓰지 않습니다.

## 문서 변경

영어·한국어·중국어 간체는 동등한 문서 버전입니다. 지원 언어 중 어디에서든 수정을 시작하고, 가능하면 관련 언어 문서를 함께 갱신합니다. 필요하면 PR에서 언어 검토를 요청합니다. [언어 지원 가이드](TRANSLATIONS.md)를 따르며 CI는 언어별 파일 구성을 확인합니다.

## 완료 기준

- 핵심 동작에 회귀 테스트가 있습니다.
- 작은 터미널에서도 렌더링이 panic을 일으키지 않습니다.
- 저장 실패 시 거짓 성공 메시지나 메모리 지갑 변경이 없습니다.
- 새 라우팅이 `GameSession`과 카탈로그 검증을 통과합니다.
- `./scripts/check.sh`가 성공합니다.
- 사용자 동작, 아키텍처, 관련 번역이 갱신됐습니다.
- 브랜치, 커밋, PR이 [Git 작업 규칙](GIT_WORKFLOW.md)을 따릅니다.
