# 운영·문제 해결 가이드

이미 연결한 Workspace Gateway의 업데이트, 접속 코드 관리와 장애 진단을 다뤄요. 처음 연결할 때는 [설치·연결](DOT_SLACK_SETUP.md), 일상 작업을 요청할 때는 [작업 요청](WORKSPACE_USAGE.md)을 참고하세요.

## 운영과 업데이트

ChatCmd, 프록시, ngrok은 모두 실행 중이어야 해요. 이 가이드는 자동 시작 서비스를 설치하지 않아요. 상시 운영하려면 OS 서비스 관리자로 세 프로세스와 비밀 환경 설정을 관리하고, 재부팅 후 실제 도구 호출을 재검증해요.

업데이트는 자신의 포크를 기준으로 해요. 먼저 소스 저장소 폴더로 이동해요. 내장 업데이트 기능의 배포 저장소가 이 포크로 변경되었다고 가정하지 않아요.

1. 해당 Gateway의 진행 중인 작업·실행 세션이 끝날 때까지 기다려요. 업데이트를 위해 다른 프로젝트나 프로세스를 중지·취소하지 않아요. 사용할 작업이 없을 때 ChatCmd를 정상 종료하고 데이터 폴더를 백업해요. 실행 중인 SQLite DB 파일 하나만 복사하지 마세요.
2. `git status --short`로 기존 변경을 확인해요. 보존할 변경이 있다면 자동 덮어쓰기를 하지 않아요.
3. 아래 명령을 단계별로 실행하고 성공을 확인해요.
4. 기존 DB 경로·초기 루트로 새 바이너리를 실행해요. 접속 코드가 그대로라면 프록시 주소도 유지해요.
5. ChatGPT 플러그인의 **Refresh** 후 새 대화에서 읽기·프로젝트 선택·쓰기·실행을 다시 확인해요.

```sh
git switch feat/shared-project-selection
git pull --ff-only origin feat/shared-project-selection
cd web
npm ci
npm run build
cd ..
cargo build --locked --release --features embedded-web
```

공개 origin이 바뀌면 ChatGPT의 등록 주소를 수정해요. 접속 코드를 회전하면 프로필 링크·프록시 설정·ChatGPT 주소를 함께 갱신해요. 분실·유출 시 **Plugin list → 해당 프로필 `•••` → Create new access code**로 교체해요. 긴급 차단은 **Allow connections**를 끄는 방식으로 해요.

## 문제 해결

| 관찰된 증상 | 확인할 위치와 조치 |
|---|---|
| 도구가 보이지 않음 | ChatGPT 플러그인 등록·활성화·도구 목록과 dot의 계정 확인. 필요하면 Refresh 후 새 대화 |
| 초기 연결 실패·502 | ChatCmd·프록시·터널 프로세스, 포트, 공개 origin과 전체 MCP 경로 확인 |
| `/`를 초기 루트로 지정한 서버의 시작 실패 | 2026-10-03 별도 검증 서버에서 복구 탐색 중 `Bad file descriptor`로 시작 실패. `CHATCMD_WORKSPACE_ROOTS`는 작은 기존 폴더로 두고 실제 개발 부모는 Projects에서 공유. 운영 서버에 실패 설정을 적용하지 않음 |
| 401·404 | 프로필 활성화, 정확한 접속 경로, 코드 회전 여부 확인. 관리 API의 404는 공개 차단 의도 |
| mTLS 연결 거절·403 | CA 다운로드·정책 적용·SAN 확인. 일반 브라우저 거절만으로 실패 판정하지 않음 |
| `conversation_approval_denied`·`notStarted` | 새 대화 승인과 해당 작업 상태 확인. 응답만으로 미승인과 승인 시간 만료를 구분하지 않음 |
| 실행 승인 대기·`policy_denied` | 프로필 도구 선택, 해당 작업 실행 모드, 현재 공유 폴더·세션 작업 경로 확인 |
| `project_selector_ambiguous` | 같은 이름의 여러 공유 부모 또는 새 프로젝트용 부모가 여러 개. 명시적 작업 프로젝트 선택 필요 |
| 태그를 썼는데 이전 작업 폴더 유지 | 미수락 태그인지, 명시적 작업 프로젝트가 우선하는지 `agent_user_message` 결과와 `workspace_context` 확인 |
| “사용자 취소”·출력 조회 실패 | 클라이언트 취소·연결 중단·승인 변경 여부를 실제 기록과 대조. 자동으로 사용자의 직접 취소라고 단정하지 않음 |
| 서로 다른 Slack 스레드가 같은 `taskId` 반환 | 관찰된 작업 ID 공유이며 이것만으로 dot 대화 공유나 병렬 작업 불가를 판정하지 않음. 대상 폴더·명시적 cwd·별도 worktree 확인. 다른 작업은 중지·취소하지 않고 실제 충돌이 있을 때 이번 요청만 보류 |
| Slack에서만 응답·권한 문제 | 연락 수단 계정, 채널 초대·멘션, 플러그인 계정·권한 확인. 웹 dot 읽기 호출과 비교 |

서버 기록에는 호출이 없는데 클라이언트에서 실패했다면 서버 실행 성공을 추측하지 않아요. 호출 기록, 세션 ID, 종료 상태를 단계별로 비교하고, 공유용 기록에서는 자격 증명을 지워요. 추가 진단은 [Troubleshooting](TROUBLESHOOTING.md)을 참고해요.

## 실제 검증 결과

2026-10-03 기준, 이 브랜치의 프로젝트 선택은 분리된 HTTP/SSE MCP 환경에서 30회 호출·28개 검증 항목과 6개 기능 테스트를 통과했어요. 이후 실행 서버에 해당 버전을 적용하고 실제 Slack 두 스레드에서 dot에게 샘플 작업을 요청했어요. 응답을 서버 호출 기록·호스트 생성 파일·저장된 터미널 출력과 종료 코드에 대조했어요.

| 실제 Slack 검증 | 결과 |
|---|---|
| `workspace_roots` 읽기 | 호출·응답 성공 |
| 새 프로젝트 A: 계산기·테스트 파일 생성 | 성공. 최초 쓰기 2회는 부모 폴더 미생성으로 실패 후 복구 |
| A: unittest / `7 + 5` / `12 / 3` | 10개 OK / `12` / `4`, 세 실행 모두 종료 코드 0 |
| 새 프로젝트 B: 곱셈·테스트 파일 생성과 실행 | 3개 OK / `31 × 13 = 403`, 두 실행 모두 종료 코드 0 |
| 기존 프로젝트 이름 선택 후 파일 읽기 | 성공. 최초 범위 단위 `lines` 오류를 `line`으로 고친 뒤 성공 |
| 서로 다른 두 Slack 스레드의 작업 ID | 같은 `taskId` 반환. dot 내부 대화 상태와 동시 작업의 기본 경로 격리는 미확인 |

파일 생성·읽기·실행 성공과 독립된 병렬 작업의 검증은 별개예요. 위 테스트에서는 두 샘플이 순서대로 실행됐으며 동시 작업의 기본 경로 격리는 확인하지 못했어요. 같은 ID 자체를 오류로 보거나 다른 작업 중지의 근거로 삼지 않아요. 각 설치 환경에서는 [설치·연결 가이드](DOT_SLACK_SETUP.md)의 웹·Slack 검증을 진행해야 해요.

- [프로젝트 선택·도구 규약](mcp_method.md)
- [파일 경로와 traversal 정책](workspace-path-safety.md)
- [OpenAI: 플러그인 연결·테스트](https://developers.openai.com/plugins/deploy/connect-chatgpt)
- [OpenAI: dot의 컴퓨터·앱 연결](https://learn.chatgpt.com/docs/dots/computers-and-apps)
- [OpenAI: dot과 Slack 연락 수단](https://learn.chatgpt.com/docs/dots/channels)
