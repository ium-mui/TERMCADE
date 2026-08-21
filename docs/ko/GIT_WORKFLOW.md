# Git 작업 규칙

[English](../GIT_WORKFLOW.md) · [한국어](GIT_WORKFLOW.md) · [简体中文](../zh-CN/GIT_WORKFLOW.md)

이 규칙은 TERMCADE 변경을 검토하고 릴리스하기 쉽게 만들기 위한 개발 작업 규칙입니다.

## 이슈

논의하거나 추적할 필요가 있는 변경은 이슈를 사용합니다.

- **Bug report**: 재현 가능한 잘못된 동작
- **Feature request**: 새 게임이나 동작
- **Documentation issue**: 누락되거나 잘못되었거나 번역되지 않은 내용

작고 명확한 수정은 바로 PR을 열어도 됩니다. 버그에는 재현 절차를, 기능에는 기대 결과를 적습니다. 보안과 직접 관련된 버그만 간단한 [보안 안내](SECURITY.md)를 따릅니다.

## 브랜치

최신 `main`에서 수명이 짧은 브랜치를 만듭니다.

```bash
git switch main
git pull --ff-only
git switch -c feat/new-game
```

브랜치 이름은 자동화 도구가 사용하므로 소문자 ASCII kebab-case로 작성합니다.

| 접두사 | 용도 |
| --- | --- |
| `feat/` | 새 동작 |
| `fix/` | 버그 수정 |
| `docs/` | 문서 또는 언어 작업 |
| `refactor/` | 내부 구조 변경 |
| `test/` | 테스트 전용 변경 |
| `build/` | 의존성 또는 패키징 |
| `ci/` | GitHub Actions와 자동화 |
| `chore/` | 기타 유지보수 |

필요하면 `fix/42-history-write`처럼 이슈 번호를 넣습니다. 병합 후 브랜치를 삭제합니다.

## 커밋

Conventional Commits 형식을 사용합니다.

```text
<type>[optional scope][!]: <summary>
```

지원 유형은 `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`, `chore`, `revert`입니다. 릴리스 도구가 읽는 유형과 선택적 scope는 ASCII로 유지하고, 요약은 영어·한국어·중국어 간체 중 하나로 명확하게 작성할 수 있습니다.

```text
feat(maze): add deterministic abyss stage
fix(history): 기록 저장 실패 시 지갑 상태 유지
docs(zh-CN): 补充扫雷操作说明
```

호환되지 않는 변경은 `!` 또는 `BREAKING CHANGE:` footer로 표시합니다. 비밀 정보, 생성된 릴리스 압축 파일, 무관한 포맷 변경은 커밋하지 않습니다.

## PR

- PR 하나에는 논리적인 변경 하나만 담습니다.
- squash 커밋이 되는 PR 제목은 Conventional Commit 형식을 사용합니다.
- 목적, 중요한 구현 선택, 검증 방법을 설명합니다.
- 변경된 동작에는 테스트를 추가하고 사용자 문서는 관련 언어 버전을 갱신합니다.
- 로컬에서 `./scripts/check.sh`를 실행합니다.
- 리뷰 대화를 해결하고 `main`이 변경되면 브랜치를 최신화합니다.

PR 설명과 리뷰는 지원 문서 언어 중 어느 언어로든 작성할 수 있습니다. 코드 식별자와 명령은 정확히 적어 언어와 관계없이 변경을 검증할 수 있게 합니다.

## 보호된 `main`

변경은 PR을 통해 `main`에 들어갑니다. 필수 검사는 `branch-name`, `pr-title`, `quality`, `msrv`입니다. 브랜치는 최신 `main`을 기준으로 해야 하고 리뷰 대화가 해결되어야 합니다. 선형 이력을 유지하며 강제 push와 브랜치 삭제는 차단합니다. 일반 PR은 squash merge합니다.
