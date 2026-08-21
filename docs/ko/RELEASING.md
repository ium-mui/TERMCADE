# 릴리스 절차

[English](../RELEASING.md) · [한국어](RELEASING.md) · [简体中文](../zh-CN/RELEASING.md)

TERMCADE는 GitHub Releases의 실행 파일 압축본으로 배포합니다. `.github/workflows/release.yml`이 `main`을 기준으로 릴리스를 빌드합니다.

## 자동화 흐름

```text
main의 Conventional Commits
  → Release Please가 버전 PR 준비
  → 버전 PR 검토 및 병합
  → GitHub가 vX.Y.Z 릴리스 생성
  → CI가 5개 플랫폼 압축본과 SHA256SUMS 생성
```

Release Please는 `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `.release-please-manifest.json`을 갱신합니다.

| 커밋 | 버전 변경 |
| --- | --- |
| `fix:` | Patch |
| `feat:` | Minor |
| `type!:` 또는 `BREAKING CHANGE:` | Major |
| `docs:`, `test:`, `chore:` | 단독으로는 버전 변경 없음 |

## 저장소 설정

`Release` 워크플로는 태그, 릴리스, 파일을 만들기 위한 `contents: write` 권한이 필요합니다. 릴리스 PR까지 자동으로 만들려면 contents, issues, pull requests 쓰기 권한이 있는 전용 토큰을 `RELEASE_PLEASE_TOKEN`이라는 Actions secret으로 추가합니다. 이 secret이 없으면 올바른 형식의 릴리스 PR이 병합된 뒤 릴리스는 게시할 수 있지만 다음 릴리스 PR을 자동으로 열지는 않습니다.

## 릴리스 파일

| 플랫폼 | Rust target | 압축 형식 |
| --- | --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc` | `.zip` |

각 압축본에는 실행 파일, 프로젝트 README, `LICENSE`가 들어갑니다. `SHA256SUMS`에는 모든 압축본의 체크섬이 들어갑니다.

## 릴리스 확인

버전 PR 병합 전:

- CI 통과 여부를 확인합니다.
- 생성된 버전과 changelog를 확인합니다.
- 필요한 경우 사용자 문서가 이번 릴리스를 반영하는지 확인합니다.

병합 후:

- `vX.Y.Z` 릴리스가 생성됐는지 확인합니다.
- 5개 압축본과 `SHA256SUMS`가 첨부됐는지 확인합니다.
- 압축본 하나를 내려받아 체크섬과 `tcade --version`을 확인합니다.

빌드나 업로드가 실패하면 워크플로를 수정한 뒤 실패한 작업을 다시 실행합니다. 기존 버전 태그는 이동하지 않으며, 배포된 소스를 바꿔야 하면 patch 릴리스를 만듭니다.
