# 이슈·브랜치·커밋·Pull Request 작업 규칙

[English](../GIT_WORKFLOW.md) · [한국어](GIT_WORKFLOW.md) · [简体中文](../zh-CN/GIT_WORKFLOW.md)

이 문서는 TERMCADE 협업의 기준 규칙입니다. 작은 문서 수정은 사전 이슈 논의를 생략할 수 있지만 모든 코드 변경은 Pull Request를 거칩니다.

## 1. 이슈

새 이슈를 만들기 전에 열린 이슈와 닫힌 이슈를 검색하고 알맞은 폼을 사용합니다.

- **Bug report**: 재현 가능한 잘못된 동작
- **Feature request**: 새 게임 또는 동작 변경
- **Documentation issue**: 누락되거나 잘못되거나 번역되지 않은 내용
- **Support request**: 사용 또는 개발에 관한 구체적인 질문

분류 가능한 이슈에는 사용자 문제, 재현 절차 또는 완료 조건, 영향받는 버전, 관련 환경이 있어야 합니다. 유지관리자는 다음 레이블을 사용합니다.

| 레이블 그룹 | 값 | 의미 |
| --- | --- | --- |
| 유형 | `bug`, `enhancement`, `documentation`, `support`, `dependencies` | 작업의 성격 |
| 상태 | `needs-triage`, `needs-info`, `accepted`, `blocked` | 현재 판단 상태 |
| 우선순위 | `priority: critical`, `priority: high`, `priority: normal`, `priority: low` | 일정 약속이 아닌 정렬 신호 |
| 난이도 | `good first issue`, `help wanted` | 기여자 참여 준비 상태 |
| 영역 | `area: game`, `area: ui`, `area: persistence`, `area: ci`, `area: i18n` | 주요 담당 영역 |

보안 문제는 공개 이슈에 올리지 말고 [보안 정책](SECURITY.md)을 따릅니다.

## 2. 브랜치

`main`은 언제나 릴리스 가능한 상태이며 Pull Request로만 변경합니다. 최신 `main`에서 짧게 유지할 브랜치를 만듭니다.

```bash
git switch main
git pull --ff-only
git switch -c feat/123-memory-timer
```

소문자 kebab-case와 다음 접두사를 사용합니다.

| 접두사 | 용도 |
| --- | --- |
| `feat/` | 사용자 기능 |
| `fix/` | 버그 수정 |
| `docs/` | 문서 또는 번역만 변경 |
| `refactor/` | 동작을 바꾸지 않는 내부 변경 |
| `test/` | 테스트만 변경 |
| `perf/` | 성능 개선 |
| `build/` | 빌드 또는 의존성 변경 |
| `ci/` | 자동화 변경 |
| `chore/` | 더 좁은 유형에 속하지 않는 유지보수 |
| `release/` | 자동 릴리스 PR 전용 |

이슈가 있으면 `fix/42-history-corruption`처럼 번호를 넣습니다. 병합 후 브랜치는 삭제합니다. 장기 개발 브랜치와 개인 브랜치는 공식 흐름에 포함하지 않습니다.

## 3. 커밋

커밋 제목은 Conventional Commits를 따릅니다.

```text
<type>[optional scope][!]: <imperative summary>

[optional body]

[optional footers]
```

허용 유형은 `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`, `chore`, `revert`입니다. 도움이 될 때 `ui`, `history`, `casino`, `snake`, `docs`, `release` 같은 안정적인 scope를 사용합니다.

```text
feat(maze): add deterministic abyss stage
fix(history): preserve wallet after a failed write
docs(ko): clarify the release checklist
refactor(session)!: replace per-game options with one session enum
```

규칙은 다음과 같습니다.

- 제목은 영어 명령형으로 쓰고 마침표를 붙이지 않습니다.
- 하나의 목적에 집중하고 이유와 trade-off는 본문에 설명합니다.
- `Refs: #123`으로 참조하거나 PR에서 `Closes #123`으로 닫습니다.
- 호환되지 않는 변경은 `!`와 `BREAKING CHANGE:` footer로 표시합니다.
- 비밀, 생성 바이너리, 무관한 포맷 변경, 개인 데이터를 포함하지 않습니다.

Release Please는 `fix`를 patch, `feat`를 minor, breaking change를 major 릴리스로 계산합니다. 다른 유형은 기록에 포함되지만 단독으로 버전을 올리지는 않습니다.

## 4. Pull Request

초기에 draft를 열어 피드백을 받고 모든 체크 항목을 만족하면 ready로 전환합니다. squash merge의 최종 커밋 제목이 되므로 PR 제목 자체가 유효한 Conventional Commit이어야 합니다.

모든 PR은 다음 조건을 만족합니다.

1. 하나의 논리적 변경만 다루고 해당할 때 이슈를 연결합니다.
2. 파일 목록이 아니라 동작과 변경 이유를 설명합니다.
3. 위험도에 맞는 테스트와 정확한 검증 명령을 포함합니다.
4. 사용자 동작이 바뀌면 영어 문서와 지원 번역을 갱신합니다.
5. CI의 `branch-name`, `pr-title`, `quality`, `msrv` 검사를 통과합니다.
6. 모든 리뷰 대화를 해결합니다. 다른 유지관리자가 있으면 최소 한 명의 승인을 받고, 단독 유지관리 단계에서는 PR에 자체 검토 내용을 기록하고 모든 필수 검사를 통과합니다.
7. 최신 `main`과 충돌이 없어야 합니다.

일반 기여는 **squash merge**합니다. 의도적으로 정리한 커밋 묶음을 보존할 가치가 있을 때만 **rebase merge**를 사용합니다. merge commit은 비활성화합니다. 병합자는 최종 제목이 Conventional Commit인지 확인합니다.

## 5. 보호된 `main` 설정

저장소 관리자는 `main` ruleset에 다음을 설정합니다.

- PR은 항상 필수입니다. 유지관리자가 한 명뿐일 때는 필수 승인 수를 0으로 두고, 독립 리뷰어가 생기면 즉시 1로 올립니다.
- 승인을 사용하기 시작한 뒤 새 커밋이 올라오면 오래된 승인을 해제합니다.
- 유효한 CODEOWNERS 팀을 만든 뒤 code owner 리뷰 필수
- 필수 검사: `CI` workflow의 `branch-name`, `pr-title`, `quality`, `msrv`
- 리뷰 대화 해결 필수
- force push와 브랜치 삭제 금지
- linear history 필수
- 관리자도 규칙 적용, 긴급 유지관리자만 우회 가능
- 직접 push 금지

단독 유지관리 예외도 CI 우회를 허용하지 않습니다. 긴급 우회는 후속 이슈에 기록하고 사건 후 검토해야 하며, CI를 기다리지 않으려는 목적으로 사용하면 안 됩니다.
