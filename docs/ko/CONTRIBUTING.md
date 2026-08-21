# TERMCADE 기여 가이드

[English](../../CONTRIBUTING.md) · [한국어](CONTRIBUTING.md) · [简体中文](../zh-CN/CONTRIBUTING.md)

TERMCADE를 개선해 주셔서 감사합니다. 코드, 테스트, 문서, 번역, 버그 보고, 설계 의견을 모두 환영합니다.

참여하면 [행동 강령](CODE_OF_CONDUCT.md)에 동의한 것으로 봅니다. 취약점은 공개 이슈에 올리지 말고 [보안 정책](SECURITY.md)의 비공개 절차를 따릅니다.

## 시작 전

1. 기존 이슈와 PR을 검색합니다.
2. 새 게임, 의존성, 저장 형식, 공개 인터페이스, 큰 아키텍처 변경은 구현 전에 feature request를 엽니다.
3. 큰 작업은 `accepted` 판단을 기다립니다. 논의는 범위를 정하지만 병합을 보장하지 않습니다.
4. 의도가 명확한 작은 버그 수정, 테스트, 문서 수정은 이슈 없이 PR로 진행할 수 있습니다.

## 개발 흐름

저장소를 fork하고 최신 `main`에서 하나의 목적에 집중한 브랜치를 만듭니다.

```bash
git clone https://github.com/<your-account>/TERMCADE.git
cd TERMCADE
git remote add upstream https://github.com/ium-mui/TERMCADE.git
git fetch upstream
git switch -c fix/42-short-description upstream/main
```

Rust 1.85 이상이 필요합니다. 제출 전 전체 검사를 실행합니다.

```bash
./scripts/check.sh
```

아키텍처, 테스트, 저장 안전성, 새 게임 절차는 [개발 가이드](DEVELOPMENT.md)를 확인하세요.

## 변경 요구 사항

- [아키텍처](ARCHITECTURE.md)의 경계를 유지합니다.
- 버그에는 회귀 테스트, 기능에는 동작 테스트를 추가합니다.
- 랜덤 동작은 seeded 생성 경로로 재현 가능하게 합니다.
- 기록 파일을 지속되는 사용자 데이터로 취급하고 이전 호환성을 유지합니다.
- `unsafe` 코드를 추가하지 않습니다.
- 사용자 동작이 바뀌면 영어, 한국어, 중국어 간체 문서를 갱신합니다.
- 무관한 포맷 변경, 의존성 갱신, 생성 아티팩트, 우발적 refactor를 섞지 않습니다.

## 커밋과 PR

자세한 내용은 [Git 작업 규칙](GIT_WORKFLOW.md)을 따릅니다.

- `feat/123-new-game` 같은 접두사와 kebab-case 브랜치를 사용합니다.
- `fix(ui): avoid clipping narrow boards` 같은 영어 Conventional Commit 제목을 씁니다.
- PR 제목은 squash commit이 되므로 Conventional Commit으로 작성합니다.
- 문제, 접근 방식, 위험, 정확한 검증 내용을 설명합니다.
- PR이 이슈를 완전히 해결하면 `Closes #123`으로 연결합니다.
- 리뷰 우려를 해결한 뒤 대화를 resolve합니다.

유지관리자는 범위 축소, 추가 테스트·문서·설계 논의를 요청할 수 있습니다. 프로젝트 방향과 충돌하거나 유지보수 비용이 과도하거나 안전한 마이그레이션이 없거나 안정적으로 지원할 수 없는 기여는 거절될 수 있습니다.

## 문서와 번역

영어가 기준입니다. 로케일 문서는 `docs/ko`, `docs/zh-CN`에 있고 프로젝트 README 번역은 저장소 루트에 있습니다. [번역 정책](TRANSLATIONS.md)을 따릅니다.

번역 PR은 검토한 원문 revision을 밝히고 코드와 링크를 그대로 보존하며 언어 품질과 기술 정확성 리뷰를 모두 받아야 합니다.

## 리뷰와 병합

필수 CI가 모두 통과하고 모든 리뷰 대화가 해결돼야 합니다. 다른 유지관리자가 있으면 독립 승인 한 개가 필요합니다. 단독 유지관리 단계에서는 PR에 자체 검토를 기록하고 모든 필수 검사가 통과한 뒤에만 병합할 수 있습니다. 일반적으로 squash merge하며 Release Please가 버전을 올바르게 계산하도록 최종 제목을 수정할 수 있습니다.

기여자는 자신의 저작권을 유지하고 제출한 기여를 저장소의 [MIT License](../../LICENSE)로 허가합니다. 별도 CLA는 요구하지 않습니다.
