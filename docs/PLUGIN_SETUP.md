# Windows 설치와 ChatGPT 연결

> 이 문서는 Windows 빌드와 **OpenAI Secure MCP Tunnel**을 사용하는 대안이에요. 이 포크의 기본 목적과 공개 HTTPS MCP → dot → Slack 설정은 [dot·Slack 상세 가이드](DOT_SLACK_SETUP.md)를 먼저 참고하세요. 아래 터널 연결 방식은 공개 HTTPS 주소 등록 방식과 달라요.

새 PC에서 **소스 빌드 → ChatCMD 실행 → Secure MCP Tunnel → ChatGPT MCP 앱 → 작업 폴더 확인**까지 진행하는 설치 안내입니다. 각 사용자가 자신의 PC와 계정에서 설정합니다. 이 저장소는 소스만 공유하며, 실행 파일 배포나 공개 플러그인 패키지 제출을 요구하지 않습니다.

모든 명령은 **Windows PowerShell**에서 실행합니다. 예시는 `%LOCALAPPDATA%\ChatCMD`를 설치 기준 폴더로 사용합니다. 관리자 권한은 일반 실행에 필요하지 않습니다.

- `source`: Git 소스와 빌드 산출물
- `run\ChatCMD.exe`: 실제로 실행할 파일
- `tunnel`: 공식 터널 패키지 전체
- `backups`: 업데이트 전 실행 파일과 데이터 백업
- 기본 데이터베이스: `%LOCALAPPDATA%\ChatCmdClient\data\chatcmd.db` (설치 폴더와 별도)

ChatCMD와 터널은 **같은 PC에서 모두 실행 중**이어야 합니다. `127.0.0.1`은 그 프로그램이 실행되는 PC 자신입니다. 서버나 터널을 종료하거나 PC가 절전 상태가 되면 연결을 사용할 수 없습니다.

## 1. 준비 프로그램 설치

다음 공식 배포처에서 설치하고 새 PowerShell 창을 엽니다.

1. [Git for Windows](https://git-scm.com/downloads/win)
2. [Node.js](https://nodejs.org/en/download): 최신 Node 22 LTS 패치 버전(22.22.2 이상), npm 포함
3. [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/): **Desktop development with C++ / C++를 사용한 데스크톱 개발** 워크로드, MSVC 도구 및 Windows SDK 포함
4. [Rust / rustup](https://www.rust-lang.org/tools/install): Windows MSVC **stable** 툴체인과 Cargo

Cargo.toml의 Rust 하한만 맞추기보다 현재 stable을 사용합니다. 잠긴 의존성의 요구 버전이 더 높을 수 있습니다.

```powershell
& {
    git --version
    node --version
    npm.cmd --version
    rustup update stable
    if ($LASTEXITCODE -ne 0) { throw 'Rust update failed' }
    rustup default stable
    if ($LASTEXITCODE -ne 0) { throw 'Rust toolchain selection failed' }
    cargo --version
}
```

## 2. 소스 다운로드와 빌드

처음 설치할 때 한 번 실행합니다. 같은 `source` 폴더가 이미 있으면 아래 새 설치 명령 대신 [업데이트](#9-업데이트와-백업)를 따릅니다. 각 명령은 실패하면 중단하도록 구성했습니다.

```powershell
& {
    $ErrorActionPreference = 'Stop'
    $Root = Join-Path $env:LOCALAPPDATA 'ChatCMD'
    New-Item -ItemType Directory -Force -Path $Root | Out-Null
    if (Test-Path "$Root\source") { throw 'Source already exists; use the update section' }
    git clone --branch feat/shared-project-selection https://github.com/pjy1368/ChatCmd.git "$Root\source"
    if ($LASTEXITCODE -ne 0) { throw 'Clone failed' }
    Set-Location "$Root\source\web"
    npm.cmd ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    npm.cmd run build
    if ($LASTEXITCODE -ne 0) { throw 'Web build failed' }
    Set-Location "$Root\source"
    cargo build --locked --release --features embedded-web
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    New-Item -ItemType Directory -Force -Path "$Root\run" | Out-Null
    Copy-Item -LiteralPath "$Root\source\target\release\chat-cmd-client.exe" -Destination "$Root\run\ChatCMD.exe"
}
```

`embedded-web`는 관리 화면을 실행 파일에 포함합니다. Cargo 산출물 이름은 `chat-cmd-client.exe`이고, 이 안내에서는 실행용 복사본을 `ChatCMD.exe`로 통일합니다. Node와 Rust는 빌드할 때 사용하며 실행할 때 별도 Vite 서버를 켜지 않습니다.

## 3. ChatCMD 실행과 안전한 승인 설정

```powershell
& {
    $Root = Join-Path $env:LOCALAPPDATA 'ChatCMD'
    Start-Process -FilePath "$Root\run\ChatCMD.exe" -WorkingDirectory "$Root\run"
    Start-Process 'http://127.0.0.1:8080'
}
```

첫 실행에서 데이터베이스와 스키마가 생성됩니다. 같은 PC에서 ChatCMD를 중복 실행하지 않습니다. 브라우저를 닫아도 앱은 트레이에 남을 수 있습니다. 종료할 때는 트레이의 **Quit**을 사용합니다. 다음 실행도 위 명령이나 `run\ChatCMD.exe` 바로가기를 사용합니다.

**Settings → Execution**에서 다음을 확인하고 저장합니다.

- **Default execution mode → Ask for approval**: 처음 사용하는 사용자에게 권장합니다. 새 데이터베이스의 기본값은 승인 모드이며, 기존 설치에서도 현재 값을 직접 확인합니다.
- **Approve new conversations**: 켜 두면 새로운 ChatGPT 대화의 시작을 별도로 승인합니다.
- **Allow all**은 선택 사항입니다. 허용된 도구와 경로 범위에서 **파일 쓰기·삭제·명령 실행까지** 사전 작업 승인 없이 진행될 수 있습니다. 읽기 전용 설정이 아닙니다.

**승인 횟수를 줄이는 선택 설정:** 개인 PC에서 연결된 앱과 작업을 신뢰한다면 **Default execution mode → Allow all**과 **Approve new conversations 끄기**를 함께 선택하고 저장할 수 있습니다. 기존 대화는 **Access permissions → Allow everything**도 별도로 확인합니다. ChatGPT 쪽의 앱 도구 권한은 그쪽 설정에서 따로 검토합니다. 새 대화 승인을 끄면 다음 새 대화가 시작 승인을 기다리지 않아 편하지만, 새 세션을 검토하는 보호 단계가 없어집니다. Allow all과 함께 쓰면 허용 범위의 쓰기·삭제·명령 실행이 확인 없이 진행될 수 있으므로 필수 설치 조건은 아닙니다.

승인 설정은 세 층으로 나뉩니다.

1. **ChatGPT 앱/플러그인 권한**: ChatGPT가 도구 호출을 승인하는 정책
2. **ChatCMD 프로필의 도구 선택과 프로젝트 경로**: 실제로 사용할 수 있는 도구 및 파일 범위
3. **ChatCMD 실행 모드와 새 대화 승인**: 로컬 동작을 실행하기 전 승인할지 결정

한쪽의 허용이 다른 쪽을 자동으로 바꾸지는 않습니다. 기존 대화는 개별 **Access permissions → Approval / Allow everything** 설정이 전역 기본값보다 우선할 수 있습니다. 전역 기본값 변경만으로 기존 대화 설정이나 이미 대기 중인 승인 요청이 없어지지 않습니다. 대화별 실행 모드 변경은 대기 중인 작업 승인을 취소하고 활성 범위 승인을 회수할 수 있으므로 필요한 요청을 다시 실행합니다.

**터미널/명령은 ChatCMD를 실행한 Windows 사용자 권한으로 실행됩니다. 프로젝트 경로 설정은 OS 수준의 파일·네트워크 샌드박스가 아닙니다.**

## 4. ChatCMD MCP 접속 프로필 생성

1. 관리 화면에서 **Plugin list → Create new Plugin connection**을 엽니다.
2. 알아보기 쉬운 이름을 입력합니다.
3. 필요한 도구만 선택합니다. 처음에는 읽기 중심으로 시작하고 파일 수정·터미널 기능은 필요할 때 추가합니다.
4. **Allow connections**를 켜고 저장합니다.
5. 생성 직후 **Save this connection link** 창에 표시되는 로컬 MCP 접속 주소를 개인 비밀 저장소에 보관합니다.
6. **I saved the connection link**를 눌러 닫습니다. 주소 형식은 다음과 같습니다.

```text
http://127.0.0.1:8080/mcp/<token>
```

`<token>`은 설명용 자리표시자입니다. 실행할 때는 방금 발급한 실제 주소를 입력합니다. 주소 자체가 자격 증명이며 별도 Authorization 헤더를 넣지 않습니다. 실제 주소를 Git, 문서, 채팅, 스크린샷, 공유 바로가기에 넣지 않습니다. 이후 터널은 **이 전체 주소**로 연결합니다. 주소를 분실했다면 프로필의 `••• → Create new access code`에서 교체를 확인합니다. 이전 주소는 무효화되므로 터널에도 새 주소를 입력합니다.

## 5. OpenAI 터널과 런타임 준비

[OpenAI Secure MCP Tunnel 공식 안내](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels)를 참고해 다음을 준비합니다.

1. [Platform 터널 설정](https://platform.openai.com/settings/organization/tunnels)에서 사용할 조직을 선택하고 터널을 생성합니다. 터널 ID를 보관합니다.
2. 사용할 ChatGPT 작업공간을 터널에 연결합니다. 생성·수정에는 조직의 Tunnels Read + Manage, 실행과 앱 선택에는 Read + Use 권한이 필요합니다.
3. [조직 API 키 설정](https://platform.openai.com/settings/organization/api-keys)에서 해당 터널을 실행할 수 있는 런타임 API 키를 준비합니다. 키는 비밀 저장소에 보관합니다.
4. [공식 최신 릴리스](https://github.com/openai/tunnel-client/releases/latest)의 Assets에서 CPU에 맞는 **Windows runtime-cloudflared** 패키지를 받습니다. Intel/AMD 64비트는 `windows-amd64`, ARM Windows는 `windows-arm64` 패키지를 선택합니다.
5. 압축의 **전체 내용**을 `%LOCALAPPDATA%\ChatCMD\tunnel`에 풉니다. 중첩 폴더가 생겼다면 실행 파일이 아래 위치에 오도록 옮깁니다. `cloudflared.exe`, manifest 등 동봉 파일도 함께 유지합니다.

```text
%LOCALAPPDATA%\ChatCMD\tunnel\tunnel-client-runtime-cloudflared.exe
```

이 안내는 위 **runtime-cloudflared 실행 파일의 `run` 명령**을 사용합니다. 일반 CLI용 프로필 초기화 절차와 섞지 않습니다. 별도 Cloudflare 계정이나 공개 도메인은 이 연결 절차에 필요하지 않습니다. Secure MCP Tunnel은 비공개 MCP 연결용이며 공개 앱 디렉터리 배포 절차와는 별개입니다.

## 6. 비밀값을 가려 입력하고 터널 실행

ChatCMD 관리 화면이 열리는 상태에서 **새 PowerShell 창**에 아래 블록을 붙여 넣습니다. 명령을 먼저 실행한 뒤 API 키, 터널 ID, MCP 주소를 차례로 입력합니다. API 키와 MCP 주소 입력은 화면에 표시되지 않습니다.

```powershell
& {
    $ErrorActionPreference = 'Stop'
    $Root = Join-Path $env:LOCALAPPDATA 'ChatCMD'
    Set-Location "$Root\tunnel"
    if (-not (Test-Path '.\tunnel-client-runtime-cloudflared.exe')) { throw 'Runtime executable missing' }
    $Key = Read-Host 'Control plane API key' -AsSecureString
    $Mcp = $null
    $McpUrl = $null
    try {
        $env:CONTROL_PLANE_API_KEY = [System.Net.NetworkCredential]::new('', $Key).Password
        $TunnelId = Read-Host 'Tunnel ID'
        $Mcp = Read-Host 'ChatCMD local MCP URL' -AsSecureString
        $McpUrl = [System.Net.NetworkCredential]::new('', $Mcp).Password
        if ([string]::IsNullOrWhiteSpace($env:CONTROL_PLANE_API_KEY) -or [string]::IsNullOrWhiteSpace($TunnelId)) { throw 'Key and tunnel ID are required' }
        if ($McpUrl -notmatch '^http://127\.0\.0\.1:8080/mcp/[^/?#\s]+$') { throw 'Use the local ChatCMD MCP URL on port 8080' }
        .\tunnel-client-runtime-cloudflared.exe run `
            --control-plane.api-key env:CONTROL_PLANE_API_KEY `
            --control-plane.tunnel-id $TunnelId `
            --mcp.server-url $McpUrl `
            --health.listen-addr 127.0.0.1:0
        if ($LASTEXITCODE -ne 0) { throw 'Tunnel runtime exited with an error' }
    } finally {
        Remove-Item Env:CONTROL_PLANE_API_KEY -ErrorAction SilentlyContinue
        $McpUrl = $null
        $Key.Dispose()
        if ($null -ne $Mcp) { $Mcp.Dispose() }
    }
}
```

- 키는 이 PowerShell과 자식 프로세스에만 전달하며 사용자 환경변수나 파일에 영구 저장하지 않습니다. 다음 실행 때 다시 입력합니다.
- 가려 입력하면 명령 기록에 비밀값을 직접 적지 않아도 됩니다. 실행 중 키는 프로세스 환경에, MCP 주소는 런타임 명령줄에 존재할 수 있으므로 같은 사용자 권한의 프로그램·관리자로부터 숨기는 보안 경계는 아닙니다. 화면 공유와 프로세스/로그 수집에도 주의합니다.
- `127.0.0.1:0`은 사용 가능한 로컬 health 포트를 선택합니다. 실행 로그에 표시된 상태 주소의 `/ui`를 열어 연결 상태를 확인합니다. ChatCMD의 `8080` 포트와 별도입니다.
- 이 창을 유지합니다. 중지는 `Ctrl+C`입니다. 재부팅 후에는 **ChatCMD 실행 → 위 터널 실행 → 연결 확인** 순서로 다시 시작합니다.
- 이 저장소가 터널 자동 시작·백그라운드 감독·자동 재시작을 제공하는 것은 아닙니다. 한 터널을 중복 실행하지 않습니다.

## 7. ChatGPT에 비공개 MCP 앱 연결

1. 해당 계정/작업공간의 ChatGPT 설정에서 **Security and sign-in / Security and login → Developer mode**를 켭니다. 조직 계정은 관리자가 기능을 허용해야 합니다.
2. [ChatGPT Plugins](https://chatgpt.com/plugins)의 추가 버튼에서 개발자 모드 앱을 만듭니다. 화면에 따라 Apps, Plugins 등으로 표시될 수 있습니다.
3. 이름을 입력하고 **Connection → Tunnel**을 선택합니다.
4. 5단계의 터널을 선택하거나 같은 **터널 ID**를 입력합니다. 로컬 MCP 주소나 API 키를 이 칸에 넣지 않습니다.
5. 현재 ChatCMD의 토큰 경로 방식은 **No authentication / AuthNone**을 선택합니다. 인증이 사라지는 뜻이 아니라 터널이 전달하는 로컬 주소의 토큰으로 ChatCMD가 인증한다는 뜻입니다.
6. 개발자 경고와 권한을 검토하고 생성·연결합니다. ChatCMD와 터널은 계속 실행합니다.
7. 대화에서 새 앱을 선택합니다. 필요하면 ChatGPT를 새로고침합니다.

앱 생성 성공과 실제 도구 호출 성공은 별도입니다. 아래 단계에서 실제 폴더 목록을 확인합니다. 소스 설치와 직접 MCP 연결에는 별도의 플러그인 ZIP이나 공개 제출이 필요하지 않습니다.

## 8. 프로젝트 등록과 실제 연결 확인

1. ChatCMD에서 **Projects → +**를 열고 **Name**에 프로젝트 이름을 입력한 뒤 **Project folder → Choose folder**로 작업 폴더를 선택합니다.
2. 이미 등록된 프로젝트는 우클릭 후 **Edit project**를 엽니다.
3. 모든 대화에서 이 폴더를 사용하려면 **Allow access in all conversations**를 체크하고 **Save / Save changes**를 누릅니다. 새 프로젝트와 기존 프로젝트 모두 기본은 체크 해제입니다. 특정 대화에서만 사용할 폴더는 체크하지 않고 해당 대화에 프로젝트를 연결합니다.
4. ChatGPT에서 앱을 선택하고 “workspace_roots로 접근 가능한 작업 폴더를 확인해 줘”라고 요청합니다. 새 대화 승인 요청이 뜨면 ChatCMD에서 확인합니다.
5. 공유를 켠 프로젝트는 반환된 `@shared-…` 별칭으로 확인합니다. “반환된 별칭을 사용해 workspace_list로 폴더 목록을 읽어 줘”라고 요청합니다. 별칭을 임의로 만들어 입력하지 않습니다.
6. ChatCMD에서 작업과 도구 결과를 확인합니다. 쓰기 권한은 필요한 경우에만 추가하고, 실제 작업 파일 대신 테스트 폴더로 먼저 확인합니다.

`workspace_roots`와 `workspace_list`는 등록 도구가 아닙니다. 폴더 공유를 켜도 프로필의 도구 제한과 실행 승인 정책은 유지됩니다. **Approve**는 개별 승인, **Allow similar**는 제한된 유사 작업에 대한 범위·기간이 있는 승인으로, 모든 쓰기와 명령 실행을 허용하는 옵션이 아닙니다.

공유 접근만 회수하려면 프로젝트 편집에서 체크를 해제하고 저장합니다. 이후 호출부터 적용되며 이미 실행 중인 동작을 되돌리지는 않습니다. 별도로 해당 대화에 허용한 경로는 유지될 수 있습니다. 프로젝트 삭제는 연결된 대화에 영향을 줄 수 있으므로 권한 회수 대신 사용하지 않습니다.

## 9. 업데이트와 백업

이 소스 전용 배포는 **Git 업데이트 후 직접 다시 빌드**합니다. 내장 업데이트 버튼이나 upstream 바이너리를 이 설치 절차 대신 사용하지 않습니다.

1. 진행 중인 작업을 마칩니다.
2. 터널 창에서 `Ctrl+C`, ChatCMD 트레이에서 **Quit**을 선택합니다.
3. 아래 명령을 실행합니다. 소스 변경 사항이 있으면 중단하므로 먼저 별도로 보존합니다. 강제 reset이나 덮어쓰기를 하지 않습니다.

```powershell
& {
    $ErrorActionPreference = 'Stop'
    $Root = Join-Path $env:LOCALAPPDATA 'ChatCMD'
    if (Get-Process -Name ChatCMD,chat-cmd-client -ErrorAction SilentlyContinue) { throw 'Quit ChatCMD before updating' }
    Set-Location "$Root\source"
    $Changes = git status --porcelain
    if ($LASTEXITCODE -ne 0) { throw 'Git status failed' }
    if ($Changes) { throw 'Preserve local changes before updating' }
    $Backup = Join-Path "$Root\backups" (Get-Date -Format 'yyyyMMdd-HHmmss-fff')
    New-Item -ItemType Directory -Path $Backup | Out-Null
    Copy-Item -LiteralPath "$Root\run" -Destination "$Backup\run" -Recurse
    $Data = Join-Path $env:LOCALAPPDATA 'ChatCmdClient\data'
    if (Test-Path $Data) { Copy-Item -LiteralPath $Data -Destination "$Backup\data" -Recurse }
    git switch feat/shared-project-selection
    if ($LASTEXITCODE -ne 0) { throw 'Branch selection failed' }
    git pull --ff-only origin feat/shared-project-selection
    if ($LASTEXITCODE -ne 0) { throw 'Source update failed' }
    Set-Location "$Root\source\web"
    npm.cmd ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    npm.cmd run build
    if ($LASTEXITCODE -ne 0) { throw 'Web build failed' }
    Set-Location "$Root\source"
    cargo build --locked --release --features embedded-web
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    Copy-Item -LiteralPath "$Root\source\target\release\chat-cmd-client.exe" -Destination "$Root\run\ChatCMD.exe" -Force
    Start-Process -FilePath "$Root\run\ChatCMD.exe" -WorkingDirectory "$Root\run"
}
```

- 위 백업은 기본 데이터 위치 기준입니다. **CHATCMD_DB_PATH를 따로 설정했다면 실행 전에 실제 DB와 같은 디렉터리의 관련 파일을 앱이 종료된 상태에서 별도 백업**하고, 재실행 때 같은 경로를 유지합니다. 기존 `CHATCMD_BIND`, `CHATCMD_PORT` 등 사용자 실행 설정도 유지합니다. 이 안내의 터널 명령은 기본 주소/포트 기준입니다.
- 데이터베이스에는 프로젝트, 설정, 대화 기록, 접속 관련 정보가 들어 있습니다. DB를 삭제하거나 새 DB로 바꾸지 않습니다. 백업도 민감한 자료이므로 공유하거나 Git에 올리지 않습니다.
- 빌드가 실패하면 실행용 파일은 교체하지 않습니다. 기존 파일과 데이터 백업을 보존합니다. 새 버전이 DB를 마이그레이션한 뒤 이전 버전으로 돌아갈 때는 해당 버전과 짝이 맞는 백업이 필요할 수 있습니다.
- 관리 화면에서 프로젝트/설정을 확인한 뒤 6단계로 터널을 다시 실행하고 8단계의 폴더 조회를 반복합니다. 서버 주소·포트·MCP 토큰이 바뀌면 터널의 입력 주소도 갱신합니다.

## 선택 사항: 서브에이전트 활성화

서브에이전트를 사용하려면 **Settings → Execution → Sub-agent count**를 확인합니다. 기본값 **0은 비활성화**이므로 **1–5 중 필요한 동시 실행 수**로 바꾸고 저장합니다. 예를 들어 처음에는 1로 시작하고 병렬 작업이 필요할 때 늘릴 수 있습니다. Task concurrency나 Session concurrency와는 별도 설정입니다.

이 숫자는 실행 상한이며 도구 권한·승인 정책을 우회하지 않습니다. 프로필에 서브에이전트 도구가 허용되어 있어야 하고, 브라우저 fallback을 사용할 때는 아래 확장도 필요합니다. 전체 동작은 [Sub-Agent reports](subagent-reports.md)를 참고합니다.

## 선택 사항: ChatGPT 브라우저 확장

일반 MCP 연결에는 필요하지 않습니다. ChatCMD에서 ChatGPT 웹 대화를 열거나 승인 UI를 연결하려면 [확장 README](../chatgpt-extension/README.md)의 권한 설명을 먼저 읽습니다.

1. Chrome의 `chrome://extensions/` 또는 Edge의 `edge://extensions/`를 엽니다.
2. **Developer mode → Load unpacked**에서 `%LOCALAPPDATA%\ChatCMD\source\chatgpt-extension`을 선택합니다.
3. 같은 브라우저 프로필에서 ChatGPT에 로그인하고 ChatGPT와 ChatCMD 페이지를 새로고침합니다.
4. 소스 업데이트 후 확장 관리 화면에서 **Reload**합니다.

확장은 비공식 DOM 브리지이므로 ChatGPT 화면 변경의 영향을 받을 수 있습니다.

## 다른 연결 방식과 참고 문서

로컬 MCP 클라이언트는 4단계에서 만든 주소를 Streamable HTTP 서버로 직접 사용할 수 있습니다. 공개 HTTPS 리버스 프록시를 별도로 운영하는 경우에만 ChatCMD의 **Custom Tunnel / private domain**을 사용합니다. 이 가이드의 Secure MCP Tunnel에서는 공개 도메인을 ChatCMD에 추가하지 않습니다. 공개 프록시는 관리 UI와 같은 리스너를 노출할 수 있으므로 별도의 접근 통제가 필요합니다.

- [설정 변수와 Gateway 실행](DOT_SLACK_SETUP.md#1-소스-빌드와-실행)
- [문제 해결](TROUBLESHOOTING.md)
- [승인 범위](approval-grants.md)
- [작업 경로 안전 정책](workspace-path-safety.md)
- [공식 Secure MCP Tunnel 안내](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels)
