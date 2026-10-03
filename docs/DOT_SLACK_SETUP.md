# 설치·연결 가이드

이 가이드의 목표는 **클라우드 dot이 등록된 MCP 플러그인을 통해 개발 호스트에서 코딩하고, Slack에서 요청과 결과를 주고받는 구성**이에요. dot의 **Your computer → Allow access**는 켜지 않아요. ChatCmd와 MCP 연결만으로 파일·Git·명령 실행을 제공해요.

이 문서는 최초 설치와 연결 검증을 다뤄요. 이미 연결했다면 [작업 요청 가이드](WORKSPACE_USAGE.md)를 읽으세요. 업데이트와 오류 진단은 [운영·문제 해결 가이드](GATEWAY_OPERATIONS.md)에 있어요.

예시는 개인용 단일 호스트 구성이에요. 실제 접속 주소·비밀번호·쿠키·DB는 공개 저장소에 넣지 마세요. `<ACCESS_CODE>`와 `<PUBLIC_ORIGIN>`은 설명용 자리표시자예요.

## 0. 설치를 맡은 AI가 먼저 확인할 것

이 문서 하나로 최초 설치부터 Slack 검증까지 진행해요. 사용자가 [README의 설치 요청문](../README.md#처음이라면-ai에게-설치-맡기기)을 보냈다면 그 요청의 부모 폴더가 대상이에요. 사용자에게 dot 공통 지침을 저장시키는 단계는 없어요.

1. OS, 실제 사용자 계정, 작업 폴더와 기존 ChatCmd 설치·프로세스·사용 포트를 확인해요. 기존 DB와 인증 정보를 보존하고, 다른 작업·스레드·프로세스는 중지하거나 취소하지 않아요. 실행 중인 Gateway를 교체해야 한다면 작업이 끝날 때까지 유지하고 설치 준비를 먼저 해요.
2. 아래 준비물을 확인하고 없는 도구를 OS에 맞게 설치해요. 이미 설치·인증된 도구는 재사용해요. GitHub 계정이나 공개 주소를 임의로 바꾸지 않아요.
3. 소스는 **`pjy1368/ChatCmd`의 `feat/shared-project-selection`**을 사용해요. 원본의 릴리스나 포크의 기본 브랜치에 이름 선택 기능이 있다고 가정하지 않아요.
4. 1–7단계를 순서대로 실행해요. 호스트 명령과 관리 화면은 설치 AI가 처리하고, 비공개 인증·조직 승인처럼 직접 수행할 수 없는 단계만 사용자에게 넘긴 뒤 이어서 진행해요. AI에게 필요한 호스트·브라우저 도구가 없다면 그 제한을 먼저 알려요.
5. 끝의 [연결 완료 기준](#연결-완료-기준)을 실제 결과로 채우고, 사용자 폴더명에 맞춘 [Slack 요청 예시](WORKSPACE_USAGE.md)를 전달해요. 진행 중인 단계는 완료로 표시하지 않아요.

| 준비물 | 필요한 상태 |
|---|---|
| 개발 호스트와 도구 | macOS/Linux의 Git, stable Rust, Node.js·npm, CA 다운로드용 curl, 계산기 검증용 Python 3. GitHub PR을 맡길 때는 Git/`gh` 인증, worktree 도구는 해당 프로젝트 방식에 맞게 준비 |
| 공유할 부모 폴더 | 이미 존재하는 실제 경로. 예: `~/desktop/project`. `~`를 실제 계정의 절대 경로로 확장 |
| ChatGPT와 dot | 플러그인을 등록할 계정·작업공간과 사용할 dot. dot이 없다면 먼저 생성; 개발자 모드 제공 여부 확인 |
| ngrok | 설치·계정 인증, 이 문서의 mTLS 정책을 사용할 수 있는 계정 |
| Slack | 연결할 워크스페이스·사용자와 DM 또는 채널. 앱 연결에 조직 승인이 필요한지 확인 |

macOS/Linux는 아래 명령을 사용해요. Windows는 [Windows 설치·Secure MCP Tunnel 안내](PLUGIN_SETUP.md)로 서버·터널·ChatGPT 연결을 준비한 뒤 이 문서의 6–7단계로 돌아와 dot과 Slack을 검증해요. 공개 HTTPS가 필요한 요청이라면 Secure MCP Tunnel은 별도 연결 방식이라는 점을 사용자에게 설명해요.

이름 태그와 지침은 서버가 MCP 연결 시 제공해요. 설치 AI는 현재 버전의 서버 지침에 `SHARED PROJECT SELECTION`이 있는지 확인해요. 사용자는 도구 이름과 내부 `taskId`를 외우거나 긴 프롬프트를 반복할 필요가 없어요.

## 연결을 이해하기

| 구성 요소 | 역할 | 설정 위치 |
|---|---|---|
| ChatCmd | 파일·Git·명령 실행, 작업 기록, 권한 확인 | 호스트의 관리 화면 |
| MCP 전용 프록시 | 한 접속 경로만 전달하고 관리 화면을 차단 | 호스트의 별도 프로세스 |
| HTTPS 터널 | 외부 접근과 OpenAI 클라이언트 인증 | ngrok 및 mTLS 정책 |
| Workspace Gateway 플러그인 | MCP 도구를 ChatGPT와 dot에 제공 | ChatGPT Plugins |
| dot | 도구를 선택해 개발 작업 수행 | dot 대화와 프로필 |
| Slack | dot에게 요청하고 결과를 받는 연락 수단 | dot 프로필의 연락 수단 |

**공유 폴더 등록만으로 연결이 끝나지는 않아요.** 서버 실행, 프로필 권한, 공개 HTTPS 경로, ChatGPT 플러그인 등록, 실제 호출 확인이 모두 필요해요. URL을 dot 대화에 보내는 것으로 플러그인 등록을 대신할 수 없어요.

## 1. 소스 빌드와 실행

Git, 현재 stable Rust, Node.js와 npm을 준비해요. Node는 `web/package-lock.json`의 `engines`를 확인해 선택해요. 이 리비전의 빌드·테스트 의존성은 예를 들어 Node 24.15 이상인 24.x로 맞출 수 있어요. macOS에는 Xcode Command Line Tools, Linux에는 컴파일러가 필요해요. [Rust 설치](https://www.rust-lang.org/tools/install), [Node 다운로드](https://nodejs.org/en/download), [개발 안내](DEVELOPMENT.md)를 참고해요. Windows 빌드 명령은 [별도 안내](PLUGIN_SETUP.md)에 있어요.

다음은 macOS/Linux의 새 설치 예시예요. 프로젝트 선택 기능을 포함한 브랜치를 명시해요. 각 단계가 성공한 다음 단계로 진행하세요.

```sh
git clone --branch feat/shared-project-selection https://github.com/pjy1368/ChatCmd.git
cd ChatCmd/web
npm ci
npm run build
cd ..
cargo build --locked --release --features embedded-web
```

`embedded-web`는 관리 화면을 실행 파일에 포함해요. 실행할 때 Vite 서버를 별도로 켜지 않아요. 데이터와 초기 작업 루트는 소스 밖에 두는 예시예요.

```sh
mkdir -p ../chatcmd-state ../gateway-bootstrap
chmod 700 ../chatcmd-state
CHATCMD_BIND=127.0.0.1 CHATCMD_PORT=8080 \
CHATCMD_DB_PATH="$(cd ../chatcmd-state && pwd)/chatcmd.db" \
CHATCMD_WORKSPACE_ROOTS="$(cd ../gateway-bootstrap && pwd)" \
./target/release/chat-cmd-client
```

이 서버용 터미널은 계속 실행 상태로 두고, 다음 명령은 새 터미널에서 소스 폴더를 작업 디렉터리로 실행해요. 이미 8080이 사용 중이면 기존 서비스를 종료하지 말고 빈 포트를 선택해 실행·프록시 설정을 함께 맞춰요.

관리 화면은 `http://127.0.0.1:8080`에서 열어요. 초기 비밀번호 설정이나 로그인 안내가 나오면 완료해요. 위 빈 초기 루트와 별개로, 다음 단계에서 실제 개발 부모 폴더를 명시적으로 공유해요. **초기 루트는 `/`나 홈 전체 대신 작은 전용 폴더로 두세요.** 루트 전체를 지정하면 복구·인덱싱 탐색에서 시작 오류가 날 수 있어요. 실제 확인한 `/` 시작 실패는 [운영 가이드](GATEWAY_OPERATIONS.md#문제-해결)에 기록해요.

| 환경 변수 | 이 구성에서의 용도 |
|---|---|
| `CHATCMD_BIND` | `127.0.0.1`: 관리 서버를 호스트에서만 접근 |
| `CHATCMD_PORT` | `8080`: ChatCmd 포트 |
| `CHATCMD_DB_PATH` | 재실행 후에도 유지할 SQLite 파일 경로 |
| `CHATCMD_WORKSPACE_ROOTS` | 이미 존재하는 초기 작업 루트. 여러 개면 `;`로 구분 |

빌드 파일이 바뀌어도 기존 프로세스가 새 코드로 바뀌지는 않아요. 기존 작업을 중지하지 말고, 해당 서버의 실행 작업이 끝난 뒤 새 실행 파일로 교체해요.

## 2. 개발 폴더를 공유하기

예를 들어 Git 저장소와 일반 작업 폴더가 하나의 부모 아래 있다고 가정해요.

```text
~/desktop/project/
├── ch-dropwizard/
├── another-repo/
└── calculator-demo/
```

1. 관리 화면의 **Projects**에서 새 프로젝트를 만들거나 기존 프로젝트를 편집해요.
2. 이름은 `Coding Projects`, 경로는 부모 폴더의 **실제 절대 경로**로 지정해요. UI에는 위 `~` 표기 대신 확장된 경로를 넣어요.
3. **Allow access in all conversations**를 켜고 **Save / Save changes**로 저장해요.
4. 사용하지 않는 기존 프로젝트의 전체 대화 공유는 해제해요. 새 프로젝트 생성을 이름으로 선택하려면 공유 부모를 하나만 두세요.
5. 이후 `workspace_roots`를 호출해 실제 별칭을 확인해요. 별칭 이름은 응답을 따르고, 문서의 예시 이름을 추측해 사용하지 않아요.

`Coding Projects`는 관리 화면에서 공유 경로를 구별하는 표시 이름이에요. 요청에 쓰는 `[폴더명]`과는 달라요. 작업 대상으로는 이 부모 바로 아래의 실제 폴더 이름을 사용해요. 자세한 예시는 [작업 요청 가이드](WORKSPACE_USAGE.md)에 있어요.

부모를 공유하면 그 아래 다른 작업 폴더에도 접근할 수 있어요. 대상 폴더 선택을 접근 격리로 해석하지 마세요.

## 3. 읽기·쓰기·명령 실행 권한 설정

빠른 개인용 작업을 위한 설정이에요. 쓰기·삭제·명령 실행을 사전 작업 승인 없이 허용하는 구성이에요.

1. **Settings → Execution → Default execution mode → Allow all**을 선택해 저장해요.
2. **Approve new conversations**를 꺼서 새 대화의 시작 승인을 생략해요.
3. 이미 생성된 작업은 **Access permissions → Allow everything**도 확인해요. 전역 기본값은 기존 작업을 자동 변경하지 않아요.
4. **Plugin list → Create new Plugin connection**에서 `Workspace Gateway`라는 접속 프로필을 만들어요. **Per-tool permissions → Select all**, **Allow connections**를 켜고 저장해요.
5. 생성 직후 한 번 표시되는 전체 접속 링크를 비밀 저장소에 보관해요. ChatGPT 쪽 플러그인의 도구 권한도 별도로 허용해요.

```text
http://127.0.0.1:8080/mcp/<ACCESS_CODE>
```

세 가지 권한은 독립적이에요: **ChatGPT의 호출 허용**, **ChatCmd 프로필의 도구·폴더 범위**, **ChatCmd 작업의 실행·대화 승인**이에요. 하나를 열었다고 다른 설정이 자동으로 열리지 않아요. 모드 변경으로 기존 대기 요청이 취소되었다면 필요한 요청을 다시 실행해요.

명령은 **ChatCmd를 실행한 OS 계정 권한**으로 실행돼요. 프로젝트 범위가 셸 명령의 파일·네트워크 접근을 OS 샌드박스로 막아주지는 않아요. GitHub 작업에는 호스트에 Git/`gh` 설치와 계정 인증도 필요해요.

전용 파일 도구는 승인된 경로 범위를 확인해요. 도구 전체 허용이 모든 물리 경로의 파일 접근을 자동으로 허용하는 것은 아니에요. OS 사용자 권한과 MCP 경로 권한을 구분하고, 접근 거절이 나면 해당 권한을 확인해요. 공개 입구의 mTLS를 제거해 해결하지 않아요.

## 4. MCP 경로만 공개하기

구성은 **ChatCmd `8080` → MCP 전용 프록시 `8081` → ngrok HTTPS**예요. 관리 화면과 `/api/*`를 공개 터널로 전달하지 않아요. 아래 프록시 예시는 기존 운영 프록시의 정확한 경로 제한과 스트리밍 전달 방식을 사용해요. 별도 npm 패키지는 필요 없어요.

### 4-1. 비밀 설정과 프록시

앞에서 만든 `chatcmd-state` 폴더에 `connection.json`을 편집기로 만들어요. 접속 링크의 실제 값을 넣되 터미널 기록이나 채팅에 출력하지 마세요.

```json
{
  "endpoint": "http://127.0.0.1:8080/mcp/<ACCESS_CODE>",
  "proxyPort": 8081
}
```

같은 폴더에 다음 내용을 `mcp-proxy.cjs`로 저장해요. `endpoint`는 생성된 정확한 주소를 사용하고, 다른 포트를 선택한다면 코드의 대상 포트도 맞춰요.

```js
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
const config = JSON.parse(fs.readFileSync(path.join(__dirname, 'connection.json'), 'utf8'));
const endpoint = new URL(config.endpoint);
if (endpoint.origin !== 'http://127.0.0.1:8080' || endpoint.search || endpoint.hash ||
    !endpoint.pathname.startsWith('/mcp/')) throw new Error('Invalid MCP endpoint');
const methods = new Set(['GET', 'POST', 'DELETE']);
const hop = new Set(['connection', 'keep-alive', 'proxy-authenticate',
  'proxy-authorization', 'te', 'trailer', 'transfer-encoding', 'upgrade']);
function headers(input, blocked) {
  const denied = new Set([...hop, ...blocked,
    ...String(input.connection || '').toLowerCase().split(',').map(x => x.trim())]);
  return Object.fromEntries(Object.entries(input).filter(([key]) => !denied.has(key)));
}
http.createServer((req, res) => {
  if (req.url !== endpoint.pathname || !methods.has(req.method)) {
    res.writeHead(404).end();
    return;
  }
  const forwarded = headers(req.headers, ['host', 'cookie', 'authorization', 'x-chatcmdclient']);
  forwarded.host = endpoint.host;
  const upstream = http.request({ hostname: '127.0.0.1', port: 8080,
    path: endpoint.pathname, method: req.method, headers: forwarded }, reply => {
    res.writeHead(reply.statusCode, headers(reply.headers, ['set-cookie']));
    res.flushHeaders();
    reply.on('error', () => res.destroy());
    reply.pipe(res);
  });
  req.on('aborted', () => upstream.destroy());
  res.on('close', () => upstream.destroy());
  upstream.on('error', () => {
    if (!res.headersSent) res.writeHead(502).end();
    else res.destroy();
  });
  req.pipe(upstream);
}).listen(config.proxyPort, '127.0.0.1');
```

소스 폴더에서 별도 터미널로 실행하는 예시예요.

```sh
chmod 600 ../chatcmd-state/connection.json
node ../chatcmd-state/mcp-proxy.cjs
```

프록시 터미널도 계속 실행 상태로 둬요. 서버와 프록시가 각각 8080·8081을 듣는지 확인한 뒤 터널 단계로 진행해요.

이 프로세스는 요청 경로·본문을 로그에 남기지 않아요. 관리 API·관리 쿠키를 외부로 전달하지 않고, MCP 세션 헤더와 응답 스트림을 전달해요. 접속 코드를 교체하면 `connection.json`도 변경하고 프록시를 재시작해야 해요.

### 4-2. OpenAI mTLS 정책

[OpenAI 인증 문서](https://developers.openai.com/plugins/build/auth)의 **OpenAI Connectors mTLS intermediate CA**를 사용해요. 자체 서명 인증서를 새로 만드는 단계가 아니에요. 클라이언트 인증서의 체인과 SAN `mtls.prod.connectors.openai.com`을 확인하고, 교체될 수 있는 leaf 인증서 지문은 고정하지 않아요.

소스 폴더에서 공개 CA를 받아요. 다운로드가 성공했는지 확인한 뒤 다음 단계로 진행하세요.

```sh
curl --fail --location \
  https://developers.openai.com/plugins/mtls/openai-connectors-mtls-ca.pem \
  --output ../chatcmd-state/openai-connectors-mtls-ca.pem
```

`chatcmd-state`에서 다음 Node 명령을 실행해 정책 파일을 만들어요. 인증서는 공개 CA이고, 정책에는 접속 코드가 들어가지 않아요.

```sh
cd ../chatcmd-state
node <<'JS'
const fs = require('node:fs');
const ca = fs.readFileSync('openai-connectors-mtls-ca.pem', 'utf8');
if (!ca.includes('-----BEGIN CERTIFICATE-----')) throw new Error('Invalid CA file');
const policy = {
  on_tcp_connect: [{ actions: [{ type: 'terminate-tls', config: {
    mutual_tls_certificate_authorities: [ca]
  } }] }],
  on_http_request: [{ expressions: [
    '!("mtls.prod.connectors.openai.com" in actions.ngrok.terminate_tls.client.san.dns_names)'
  ], actions: [{ type: 'custom-response', config: {
    status_code: 403, content: 'Forbidden'
  } }] }]
};
fs.writeFileSync('ngrok-openai-policy.json', JSON.stringify(policy, null, 2));
JS
```

### 4-3. HTTPS 터널 실행

[ngrok 공식 설치 안내](https://ngrok.com/docs/getting-started/)에 따라 설치와 계정 인증을 완료해요. 인증 토큰은 ngrok의 개인 설정에 저장하고 Git에 넣지 않아요. 앞 단계의 `chatcmd-state`에서 실행해요.

```sh
ngrok http http://127.0.0.1:8081 \
  --traffic-policy-file ngrok-openai-policy.json \
  --inspect=false
```

터널은 세 번째 터미널에서 실행 상태로 유지해요. `chatcmd-state`가 현재 작업 디렉터리인지 확인하세요. ChatCmd·프록시·터널 중 하나가 종료되면 공개 MCP 호출은 완료할 수 없어요.

[ngrok TLS 정책](https://ngrok.com/docs/gateway/traffic-policy/actions/terminate-tls)의 mTLS 기능과 정책 사용 가능 여부는 계정에서 확인해요. 사용 불가 오류가 나면 정책을 빼고 공개하는 대신, 지원되는 구성을 준비하거나 [Secure MCP Tunnel 대안](PLUGIN_SETUP.md)을 사용해요.

ngrok이 표시하는 HTTPS origin에 기존 접속 링크의 **같은 `/mcp/<ACCESS_CODE>` 경로**를 붙여 ChatGPT 등록 주소를 만들어요.

```text
<PUBLIC_ORIGIN>/mcp/<ACCESS_CODE>
```

ChatCmd의 Custom tunnel 저장 화면을 이 MCP 전용 구성에 사용하지 않아요. 해당 화면의 `/api/ping` 검사와 달리, 이 공개 경로는 관리 API를 차단하고 mTLS를 요구해요. 접속 주소는 ChatGPT에 직접 등록해요.

**mTLS는 OpenAI 호출 클라이언트를 확인해요. 특정 사용자나 특정 dot만 확인하지는 않아요.** 같은 주소를 가진 다른 OpenAI 연결도 접근할 수 있어요. 사용자별 제한에는 별도의 OAuth 사용자 인증·권한 검증이 필요하며, 이 가이드의 접속 코드 방식에는 구현되어 있지 않아요. [OpenAI의 mTLS와 사용자 인증 구분](https://developers.openai.com/plugins/build/auth)을 참고하세요.

브라우저나 일반 `curl`에는 OpenAI 클라이언트 인증서가 없으므로 공개 주소 연결 거절이 정상일 수 있어요. mTLS 적용 후의 성공 검증은 실제 ChatGPT 호출로 해요. 승인 없이 CA/SAN 정책을 느슨하게 바꾸지 마세요.

## 5. 웹 ChatGPT에 플러그인 등록

[OpenAI 공식 연결 절차](https://developers.openai.com/plugins/deploy/connect-chatgpt)를 따라요. 화면과 기능 제공 여부는 계정·작업공간 정책에 따라 달라질 수 있어요.

1. **Settings → Security and login → Developer mode**를 켜요.
2. **ChatGPT Plugins → `+`**에서 새 연결을 만들어요.
3. 이름은 `Workspace Gateway`, 설명은 아래 예시를 사용해요.
4. **Connection**에 전체 공개 MCP 주소를 입력해요. 이 구성에서는 별도 OAuth를 제공하지 않으므로 인증 선택이 나오면 **None / No authentication**을 선택해요.
5. 연결을 생성하고 발견된 도구 목록과 권한을 확인해요. 새 대화의 도구 메뉴에서 플러그인을 추가해요.

```text
승인된 작업 공간의 파일을 조회·생성·수정하고 Git과 명령 실행 세션을 관리하는 MCP 서비스예요.
```

이름·설명·추가 지침에는 작업 공간과 개발 작업 중심의 표현을 사용해요. 이 문구가 dot의 실행 여부를 보장하거나 플랫폼 제한을 바꾸는 것은 아니에요. **None은 접속 코드가 불필요하다는 뜻이 아니에요.** 전체 URL이 프로필 자격 증명이고, 공개 입구에서는 mTLS도 확인해요.

### 먼저 읽기 호출 검증

새 웹 대화에서 다음을 요청해요.

```text
Workspace Gateway의 workspace_roots 도구를 실제 호출하고 반환된 작업 경로 목록을 알려주세요.
호출할 수 없다면 도구 미표시, 인증·권한 오류, 승인 대기를 구분해주세요.
추측한 결과는 제시하지 마세요. 파일 변경이나 명령 실행은 하지 마세요.
```

관리 화면의 해당 작업 기록에서 실제 호출과 결과를 확인해요. 반환되는 경로는 `@project` 등 가상 별칭일 수 있어요. 별칭을 호스트의 물리 경로로 임의 변환하지 않아요.

### 쓰기와 실행 검증

아직 없는 테스트 폴더 이름으로 **아래 요청 하나를 그대로 보내요.** 공통 지침을 먼저 저장할 필요는 없어요. 같은 이름이 이미 있다면 다른 이름을 정해요.

```text
[새 프로젝트: calculator-demo] Workspace Gateway 도구로 Python 계산기 샘플을 만들어주세요.
기존 파일은 먼저 확인하고 다른 작업 폴더는 변경하지 마세요.
다른 진행 중인 작업은 중지하거나 취소하지 마세요.
계산기와 unittest를 생성하고 테스트, 7 + 5, 12 / 3을 실제 실행해주세요.
각 실행의 출력과 종료 코드를 확인하고 생성 파일과 결과를 알려주세요.
실행 세션이 계속 진행 중이면 기다린 뒤 종료 상태까지 확인해주세요.
```

태그는 자동으로 폴더를 만들지 않아요. dot이 폴더 생성 도구부터 호출해야 해요. 통과 기준은 파일 생성, 실제 테스트 출력, 계산 결과 `12`·`4`, 각 종료 코드 확인이에요. 세션 생성이나 `pending`만으로 성공이라고 하지 않아요. 결과 조회가 취소되었다면 취소 원인을 추측하지 않고 세션 ID로 상태를 다시 확인해요.

설치 AI는 관리 화면의 `agent_user_message` 결과에서 이 요청의 `sharedProjectSelection`과 `projectFolder`도 확인해요. 새 폴더 요청은 공유 부모를 선택해야 해요. 생성 후 `[calculator-demo] Workspace Gateway로 생성한 파일을 읽고 목록을 알려주세요. 파일은 변경하지 마세요.`를 보내 이번에는 생성한 하위 폴더가 선택되는지 확인해요. 읽기만 성공했는데 쓰기·실행까지 검증했다고 보고하지 않아요.

## 6. dot 플러그인 연결과 호출 검증

dot이 사용할 계정에 Workspace Gateway를 설치·활성화해요. 먼저 dot의 ChatGPT 대화에서 위 `workspace_roots` 읽기 요청을 그대로 실행해요. 도구가 보이고 실제 서버 응답이 반환되어야 연결 완료예요.

dot은 설치·활성화된 지원 플러그인을 사용하고 기존 앱 권한을 적용받아요. **Your computer → Allow access는 계속 꺼둬요.** [dot의 컴퓨터·앱 연결 공식 안내](https://learn.chatgpt.com/docs/dots/computers-and-apps)를 참고하세요.

dot의 자체 클라우드 파일과 Gateway 호스트 파일은 서로 다른 환경이에요. 호스트 프로젝트 작업에는 Workspace Gateway 도구를 사용하도록 지정해요. 네이티브 컴퓨터 연결이 필요한 개인 스킬과, MCP의 `skills_list`·`skill_read`로 읽는 프로젝트 스킬은 별개예요.

## 7. Slack 연락 수단 연결과 호출 검증

1. dot 프로필에서 **Add**를 열고 **Slack** 연락 수단을 선택해요.
2. 안내에 따라 원하는 워크스페이스와 사용자 계정을 연결해요. 조직의 앱 설치 승인이 필요하면 해당 절차를 완료해요.
3. Slack에서 연결한 dot에게 DM을 보내거나 dot을 채널에 추가해요. 채널에서는 실제 dot을 멘션하고 스레드로 요청해요. 문서의 `@dot`을 일반 문자로 입력하는 것과 실제 멘션은 달라요.
4. 먼저 웹에서 사용한 `workspace_roots` 읽기 요청으로 실제 MCP 응답을 확인해요.
5. [웹 쓰기·실행 검증](#쓰기와-실행-검증)을 다른 새 폴더 이름으로 Slack에서도 진행해요. 관리 화면에서 파일 생성·실행 출력·종료 코드를 대조해요.

[OpenAI의 Slack 연락 수단 안내](https://learn.chatgpt.com/docs/dots/channels)에 따른 구성이에요. Slack 검색 플러그인 설치와 dot의 연락 수단 연결은 다른 설정이에요. ChatCmd용 Slack 앱을 별도로 만들거나 Slack 봇 토큰을 ChatCmd에 넣지 않아요.

공유 채널에서 비밀 주소, 인증 정보, 민감한 코드 내용을 공개하지 않도록 요청해요. 일상 작업에는 다음처럼 한 번의 요청을 보내면 돼요. `ch-dropwizard`와 할 일은 실제 폴더·작업으로 바꾸세요.

```text
@dot [ch-dropwizard] Workspace Gateway로 ○○ 버그를 수정하고 관련 테스트를 실행해주세요.
기존 변경은 보존하고, 다른 진행 중인 작업은 중지하거나 취소하지 마세요.
변경 파일과 실제 테스트 결과를 이 스레드에 알려주세요.
```

**Slack 스레드는 요청·답변을 정리하는 통로예요.** 2026-10-03에는 두 스레드가 같은 ChatCmd `taskId`를 받았어요. 이것만으로 dot의 내부 대화가 같거나 병렬 작업이 불가능하다고 결론 내리지 않아요. 설치 완료를 위해 작업 ID가 달라야 할 필요도 없어요. 동시 작업의 기본 경로 격리는 검증되지 않았으므로 각 요청의 대상과 실행 경로를 확인해요. [실제 검증 결과](GATEWAY_OPERATIONS.md#실제-검증-결과)를 참고하세요.

## 연결 완료 기준

설치 AI는 아래 결과를 **확인됨 / 실패 / 미확인**으로 구분해 전달해요. 사용자에게 인증·로그인을 넘겼거나 응답을 기다리는 단계는 미확인이에요.

| 확인할 항목 | 완료 증거 |
|---|---|
| 서버·접속·권한 | 실행 버전·소스 브랜치, 실제 DB·공유 부모, 프로필 도구·작업 승인 설정, 공개 관리 API 차단과 mTLS 적용 확인. 비밀값은 제외 |
| 웹 ChatGPT | 실제 roots 응답, 이름 선택, 테스트 폴더·파일 생성, 테스트·계산 출력과 종료 코드 |
| dot | 플러그인을 실제 호출해 서버 응답 반환. 자체 클라우드 실행을 Gateway 검증으로 대체하지 않음 |
| Slack | 실제 dot 멘션/DM → Gateway 읽기·쓰기·실행 → 해당 스레드의 결과를 서버 기록과 대조. 기존 폴더 태그의 후속 읽기도 확인 |
| 사용자에게 넘길 사용법 | 실제 폴더명으로 된 짧은 Slack 요청 예시, 서버·프록시·터널의 실행 위치와 재시작 명령, 오류 확인 위치 |

서버·프록시·터널은 이 예시에서는 계속 실행 중인 세 프로세스예요. 실행에 사용한 절대 경로와 환경 설정을 비밀 파일에 보관하고, 재시작은 **ChatCmd → 프록시 → ngrok → ChatGPT 실제 읽기 호출** 순서로 해요. 자동 시작 서비스는 별도로 설정한 경우에만 구성 완료로 표시해요. 호스트가 잠들거나 꺼지면 Gateway 호출은 지연·실패할 수 있지만 dot의 클라우드 작업과 Slack 응답은 계속될 수 있어요.

- [작업 요청 가이드](WORKSPACE_USAGE.md): 매번 보내는 짧은 Slack 요청, 기존·새 폴더, 후속 요청과 PR
- [운영·문제 해결 가이드](GATEWAY_OPERATIONS.md): 업데이트, 코드 회전, 권한·연결 오류와 실제 검증 결과
