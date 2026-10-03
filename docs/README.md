# ChatCMD documentation

이 포크의 주된 사용 흐름은 Workspace Gateway를 통해 dot에 개발 작업을 맡기고 Slack에서 요청·결과를 주고받는 구성이에요. 처음 연결할 때와 일상 작업을 요청할 때의 문서를 나눠 안내해요. 원본 ChatCMD의 기술 문서는 유지해요.

## Users and operators

- **[AI에게 설치 맡기기](../README.md#처음이라면-ai에게-설치-맡기기)**: 이전 대화 없이 넘길 설치 요청문
- **[설치·연결](DOT_SLACK_SETUP.md)**: 준비물, 빌드, 공유 폴더·권한, 공개 MCP, 웹·dot·Slack 검증과 완료 기준
- **[Slack 작업 요청](WORKSPACE_USAGE.md)**: 매번 보내는 짧은 예시, 폴더명의 의미, 기존·새 폴더, 후속 요청과 PR
- **[운영·문제 해결](GATEWAY_OPERATIONS.md)**: 업데이트, 접속 코드, 연결·권한 오류와 실제 검증 결과
- [Windows 빌드 및 Secure MCP Tunnel 대안 (한국어)](PLUGIN_SETUP.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [MCP method reference](mcp_method.md)
- [Diagnostic logs](logs.md)
- [Local transport protocol](ENCRYPTION_PROTOCOL.md)
- [Workspace path safety and traversal policy](workspace-path-safety.md)

## Contributors and maintainers

- [Architecture](ARCHITECTURE.md)
- [Development guide](DEVELOPMENT.md)
- [Release guide](RELEASING.md)
- [Open-source publication checklist](OPEN_SOURCE_CHECKLIST.md)
- [Engineering plans](../plan/00-README.md)

Community policies are stored at the repository root: [Contributing](../CONTRIBUTING.md), [Security](../SECURITY.md), [Support](../SUPPORT.md), [Governance](../GOVERNANCE.md), [Roadmap](../ROADMAP.md), and [Code of Conduct](../CODE_OF_CONDUCT.md).

The website documentation at [chatcmd.net/docs](https://chatcmd.net/docs) may include product screenshots and version-specific walkthroughs. Source-controlled documentation is authoritative for the behavior of the checked-out revision.
