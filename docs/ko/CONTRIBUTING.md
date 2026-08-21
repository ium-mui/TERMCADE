# TERMCADE 기여 가이드

[English](../../CONTRIBUTING.md) · [한국어](CONTRIBUTING.md) · [简体中文](../zh-CN/CONTRIBUTING.md)

게임, 테스트, 문서, 번역에 대한 기여를 환영합니다. 영어·한국어·중국어 간체는 모두 지원하는 문서 언어이며, 어느 언어에서든 기여를 시작할 수 있습니다.

## 시작하기 전에

- 기존 이슈와 PR을 검색합니다.
- 새 게임 추가, 저장 데이터 호환성 변경, 큰 아키텍처 변경은 먼저 이슈에서 논의합니다.
- 작은 수정과 문서 교정은 바로 PR을 열어도 됩니다.

## 개발 흐름

최신 `main`에서 한 가지 목적을 가진 브랜치를 만듭니다.

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

Rust 1.85 이상이 필요합니다. 제출 전에 CI와 같은 검사를 실행합니다.

```bash
./scripts/check.sh
```

아키텍처, 테스트, 저장 데이터, 새 게임 추가 절차는 [개발 가이드](DEVELOPMENT.md)를 참고하세요.

## 변경 요구 사항

- 게임 규칙과 터미널 렌더링을 분리합니다.
- 새 동작과 버그 수정에는 테스트를 추가합니다.
- 시드 기반 게임 동작은 재현 가능하게 유지합니다.
- 기존 기록 파일과의 호환성을 지키거나 명시적인 마이그레이션을 추가합니다.
- `unsafe` 코드, 비밀 정보, 생성된 바이너리, 무관한 변경을 포함하지 않습니다.
- 사용자에게 보이는 변경은 관련 언어 문서에도 반영합니다.

## 브랜치, 커밋, PR

[Git 작업 규칙](GIT_WORKFLOW.md)을 따릅니다. 핵심 규칙은 다음과 같습니다.

- `feat/new-game`, `fix/history-write`, `docs/ko-controls`처럼 접두사가 있는 브랜치를 사용합니다.
- `feat:`, `fix:`, `docs:` 같은 Conventional Commit 유형을 사용합니다. 요약은 지원 문서 언어 중 하나로 명확하게 작성할 수 있습니다.
- PR 제목은 squash 커밋이 되므로 Conventional Commit 형식을 사용합니다.
- 무엇을 왜 바꿨고 어떻게 검증했는지 설명합니다.
- PR 하나에는 논리적인 변경 하나만 담습니다.

## 문서 언어

영어·한국어·중국어 간체 문서는 같은 프로젝트 문서의 동등한 버전입니다. 파일 위치는 특정 언어에 우선권을 주지 않습니다. 정확히 검토할 수 있는 언어에서 수정을 시작하고, 가능하면 관련 언어 문서를 함께 갱신합니다. 도움이 필요하면 PR에서 언어 검토를 요청합니다. 자세한 내용은 [언어 지원 가이드](TRANSLATIONS.md)를 참고하세요.

## 병합 조건

필수 CI가 통과하고 리뷰 대화가 해결되어야 합니다. PR은 보통 squash merge합니다. 제출된 작업은 저장소의 [MIT 라이선스](../../LICENSE)를 따르며 별도의 CLA는 요구하지 않습니다.
