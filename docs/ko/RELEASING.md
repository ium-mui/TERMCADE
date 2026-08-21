# 릴리스 및 배포 절차

[English](../RELEASING.md) · [한국어](RELEASING.md) · [简体中文](../zh-CN/RELEASING.md)

TERMCADE는 CLI 애플리케이션이므로 배포는 버전이 붙은 실행 파일 압축본을 GitHub Releases에 게시하는 것을 뜻합니다. 릴리스 소스는 `main`뿐입니다.

## 릴리스 모델

`main`이 변경될 때마다 `Release` workflow가 실행됩니다.

```text
main의 Conventional Commits
  → Release Please가 하나의 릴리스 PR 갱신
  → 유지관리자가 검토하고 릴리스 PR 병합
  → vX.Y.Z 태그와 GitHub Release 생성
  → 5개 대상의 네이티브 바이너리 빌드
  → 압축 파일과 SHA256SUMS 첨부
```

Release Please는 `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `.release-please-manifest.json`을 갱신합니다. Semantic Versioning 규칙은 다음과 같습니다.

| 커밋 | 버전 영향 | 예시 |
| --- | --- | --- |
| `fix:` | Patch | `1.2.3 → 1.2.4` |
| `feat:` | Minor | `1.2.3 → 1.3.0` |
| `type!:` 또는 `BREAKING CHANGE:` | Major | `1.2.3 → 2.0.0` |
| `docs:`, `test:`, `chore:` | 단독 변경 없음 | 다른 릴리스 변경이 있을 때 포함 |

명확히 기록한 이유 없이 자동 릴리스 PR의 버전이나 changelog를 직접 바꾸지 않습니다.

## 저장소 최초 설정

1. **Settings → Actions → General → Workflow permissions**에서 GitHub Actions와 Actions의 PR 생성을 허용합니다.
2. 저장소 `Contents: write`, `Issues: write`, `Pull requests: write`, `Workflows: write` 권한이 있는 fine-grained bot token을 만듭니다.
3. Actions secret `RELEASE_PLEASE_TOKEN`으로 저장합니다.
4. [Git 작업 규칙](GIT_WORKFLOW.md)에 따라 `main` 보호 규칙을 설정합니다.
5. Git 문서의 레이블과 `autorelease: pending`, `autorelease: tagged`, `autorelease: snapshot` 레이블을 자동 생성되지 않은 경우 만듭니다.

workflow는 `GITHUB_TOKEN`으로 fallback하지만 그 토큰이 만든 PR은 다른 workflow를 시작하지 않습니다. 완전한 릴리스 PR CI 자동화를 위해 전용 토큰이 필요합니다.

## 지원 아티팩트

| Runner | Rust target | 압축 형식 |
| --- | --- | --- |
| Ubuntu x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Ubuntu ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

모든 압축 파일에는 `tcade` 실행 파일, 영어 README, 라이선스가 들어갑니다. `SHA256SUMS`가 모든 압축 파일을 검증합니다.

## 유지관리자 릴리스 체크리스트

릴리스 PR 병합 전:

- 포함된 모든 PR이 CI와 리뷰를 통과했는지 확인합니다.
- 생성된 changelog를 사용자 관점의 릴리스 노트로 읽습니다.
- `Cargo.toml`과 manifest의 버전이 같은지 확인합니다.
- breaking change와 마이그레이션이 명확한지 확인합니다.
- 문서와 번역이 릴리스를 설명하는지 확인합니다.
- 생성된 Conventional Commit 제목으로 병합합니다.

병합 후:

- `vX.Y.Z` 태그가 `main`의 릴리스 커밋을 가리키는지 확인합니다.
- 5개 압축 파일과 `SHA256SUMS`가 첨부됐는지 확인합니다.
- 아티팩트 하나를 내려받아 체크섬을 검증하고 `tcade --version` 및 한 게임을 smoke test합니다.
- 아티팩트 빌드 실패 시 실패 job을 다시 실행합니다. 업로드 단계는 같은 릴리스의 아티팩트를 안전하게 교체합니다.
- 모든 아티팩트가 준비된 뒤 릴리스를 알립니다.

## 롤백과 hotfix

GitHub Release와 공개 태그는 변경하지 않는 역사 기록입니다. 문제를 숨기기 위해 공개 버전을 이동하거나 삭제하지 않습니다.

일반 결함은 `fix:` PR로 되돌리거나 수정하고 새 릴리스 PR을 병합해 patch를 게시합니다. 보안 결함은 [보안 정책](SECURITY.md)에 따라 비공개로 수정과 advisory를 준비한 뒤 patch를 게시합니다.

소스와 태그는 올바르지만 바이너리가 손상됐다면 아티팩트 job을 다시 실행하고 체크섬을 검증합니다. 소스가 잘못됐다면 기존 기록을 교체하지 말고 새 patch 버전을 게시합니다.

## 수동 복구

GitHub 이벤트를 놓친 경우 `workflow_dispatch`로 Release Please를 다시 실행할 수 있습니다. Actions UI에서 실패 job을 재실행할 수도 있습니다. 수동 태그나 릴리스는 최후 수단이며 `vX.Y.Z` 형식, `main`에 포함된 커밋, 생성된 changelog, 같은 5개 아티팩트와 체크섬을 유지해야 합니다.
