<p align="center">
  <img src="../../desktop/src/assets/app-icon.png" width="88" alt="CueTuck">
</p>

# CueTuck · 唤词

**Your prompts, a shortcut away.**

[简体中文](../../README.md) · [English](README.en.md) · [日本語](README.ja.md) · **한국어** · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md)

자주 쓰는 프롬프트, 참고 이미지, 에이전트 Skills를 모아 두는 로컬 우선 데스크톱 작업 공간입니다. 검색하고 변수를 채운 뒤 사용하는 AI 도구에 결과를 복사하세요.

[미리 보기 버전 다운로드](https://github.com/sutao2/CueTuck/releases) · [문서](../../docs/INDEX.md) · [문제 신고](https://github.com/sutao2/CueTuck/issues)

![프롬프트 광장의 분류, 이미지 카드, 검색과 필터](../../docs/assets/readme/square.png)

## 주요 기능

| 기능 | 사용 방식 |
|---|---|
| 프롬프트 광장 | 분류, 모델, 키워드로 탐색하고 이미지·출처·작성자를 확인합니다. 다운로드와 즐겨찾기를 지원합니다. |
| 로컬 라이브러리 | 분류, 검색, 즐겨찾기, 본문과 참고 자료 편집. 카드와 목록 보기를 전환합니다. |
| 독립 실행기 | 설정 가능한 전역 단축키로 열어 검색, 변수 입력, 복사를 수행합니다. 빠른 생성, AI 개선, 명시적인 광장 검색도 가능합니다. |
| 번역과 AI | 광장의 중국어·원문·영어 버전을 전환합니다. 로컬 번역과 개선에는 직접 설정한 모델을 사용하며, 모델 목록을 불러와 선택합니다. |
| Skills 관리 | 공개 소스 탐색, 로컬 Skills 검색, 에이전트와 설치 범위별 관리, 설치, 백업 복원, 수동 업데이트 확인. |
| MCP 연결 | 호환 에이전트가 로컬 프롬프트를 검색·조회하고 변수를 적용합니다. 공개 광장 도구는 선택적으로 활성화합니다. |

## 정리부터 사용까지

### 로컬에 저장하는 템플릿

데스크톱 라이브러리는 SQLite를 사용합니다. `{{변수}}`와 참고 자료를 포함할 수 있습니다. 광장 즐겨찾기, 게시, 개인 라이브러리 동기화가 필요할 때 로그인하세요.

![예시 프롬프트와 모델 태그가 있는 로컬 라이브러리](../../docs/assets/readme/library.png)

### 입력하고 미리 보고 복사

이번 작업의 목표나 독자를 입력하고 완성된 본문을 확인합니다. 원본 템플릿은 계속 재사용할 수 있습니다.

![변수를 채운 프롬프트 미리 보기](../../docs/assets/readme/variables.png)

### 단축키로 바로 열기

실행기는 독립 창으로 열리며 키보드 조작을 지원합니다. 변수를 채우고 Enter로 복사하세요. 새 입력으로 프롬프트를 만들거나 AI 개선을 실행할 수도 있습니다.

![독립 실행기의 변수 입력 화면](../../docs/assets/readme/launcher.png)

> 스크린샷은 현재 소스의 브라우저 미리 보기입니다. 광장은 공개 콘텐츠이며 로컬 라이브러리와 실행기는 예시 데이터입니다. 네이티브 SQLite, 시스템 단축키, 파일 설치에는 데스크톱 앱이 필요합니다. 배포 패키지는 현재 소스보다 이전 버전일 수 있습니다.

## 설치와 업데이트

[GitHub Releases](https://github.com/sutao2/CueTuck/releases)에서 해당 파일을 받으세요.

| 플랫폼 | 패키지 | 지원 현황 |
|---|---|---|
| macOS · Apple Silicon | `.dmg` | arm64, 실제 Mac에서 검증 |
| Windows | `.exe` | x64, CI 설치·실행·제거 검사 통과 |
| Linux | — | 아직 검증하지 않음 |

**설정 → 업데이트**에서 버전을 확인하고 다운로드 진행률을 보며 설치를 승인할 수 있습니다. 현재 미리 보기 버전입니다. macOS 패키지는 임시 서명이며 Apple 공증을 받지 않았습니다. 시스템 안내는 [설치 문서](../../deploy/README.md)를 참고하세요.

## 시작하기

1. 로컬 프롬프트를 만들거나 광장에서 템플릿을 다운로드합니다.
2. **사용**을 선택하고 변수를 채운 뒤 AI 도구에 복사합니다.
3. 설정에서 단축키, 언어, 모양을 조정합니다.
4. **AI 및 모델**에서 서비스 URL과 API 키를 입력하고 모델 목록을 불러와 선택합니다.
5. 동기화, 광장 즐겨찾기, 게시가 필요하면 로그인하고 공개 닉네임을 설정합니다.

로컬 AI 설정은 오프라인 추론을 뜻하지 않습니다. 데스크톱 키는 시스템 자격 증명 저장소에 보관하며, 번역과 개선 시 관련 텍스트를 선택한 제공업체에 전송합니다. 게시 전 본문과 공개 첨부 파일을 확인하세요.

## Skills와 MCP

Codex, Claude Code, Cursor, Pi, OpenCode 등의 Skill 폴더를 전역 또는 프로젝트 범위로 관리합니다. 설치와 덮어쓰기는 확인 및 백업 절차를 거칩니다. Skill 설치가 MCP 의존성을 자동 설치하거나 스크립트를 실행하지는 않습니다. [Skills 명세](../../docs/specs/skills/spec.md).

MCP는 별도로 빌드하는 **Rust stdio 서비스**로, 실행에 Node.js나 uv가 필요하지 않습니다. 기본값은 로컬 읽기 전용 도구입니다. **설정 → 네트워크 및 프록시 → 에이전트 MCP 연결**에서 설정을 생성한 뒤 [연결 가이드](../../docs/how-to/mcp-clients.md)를 따르세요.

## 개발

Node.js 22+가 필요합니다. 데스크톱 개발에는 Rust와 플랫폼별 Tauri 빌드 의존성도 필요합니다.

```bash
git clone https://github.com/sutao2/CueTuck.git
cd CueTuck/desktop
npm ci
npm test
npm run dev
```

브라우저 미리 보기를 시작합니다. 네이티브 앱은 미리 보기를 종료한 뒤 `npm run tauri dev`로 실행합니다.

[로컬 개발](../../docs/how-to/local-dev.md) · [배포](../../deploy/README.md) · [기여 안내](../../CONTRIBUTING.md) · [테스트 기준](../../docs/reference/test-gates.md)

기술 문서는 주로 중국어입니다. 문제 신고에는 OS, 앱 버전, 재현 단계, 개인정보를 가린 스크린샷을 포함하고 키나 세션 토큰은 제외하세요.
