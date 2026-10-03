# Astra Workspace MCP Methods

Tên server công khai là `astra-workspace`, tiêu đề là `Astra Workspace`; catalog version hiện tại là 12. Mô tả công khai dùng tên `workspace_*`, `execution_*` và `repository_*`. Tên runtime cũ chỉ phục vụ tương thích nội bộ; quyền truy cập, xác thực, phê duyệt và tác động thật của thao tác không thay đổi. Khi catalog hash thay đổi, client phải tải lại schema thay vì tiếp tục dùng mô tả đã cache.

### Tạo ảnh

`generate_image(path, prompt?, jobId?, overwrite?, model?, waitMs?)` gửi prompt tới nhà cung cấp ảnh bên ngoài qua phiên trình duyệt đã xác thực, rồi ghi ảnh vào đích được cấp quyền. Phiên nhà cung cấp và image bridge phải sẵn sàng. Khi tạo mới cần `prompt`; khi kiểm tra job đã có dùng `jobId` với cùng `path`. Đích là file hoặc thư mục; thư mục thiếu được tạo và đuôi file theo định dạng ảnh thực tế. `overwrite` mặc định false; `waitMs` mặc định 0, tối đa 280000. `status=running` chưa phải hoàn tất: chỉ báo thành công sau khi kết quả trả về trạng thái hoàn tất và đường dẫn ảnh đã lưu. Tên cũ `workspace_generate_image` không còn nằm trong catalog mới.

Tài liệu này liệt kê các MCP tool/method mà `chatcmd-mcp` hiện expose để agent gọi về ChatCMD server/runtime.

Nguồn đối chiếu chính:

- `crates/chatcmd-mcp/src/lib.rs` — đăng ký rmcp router + schema/description của từng method; đây là source of truth runtime.
- `crates/chatcmd-mcp/src/tool_catalog.rs` — sinh canonical manifest, capability flags, metadata và `catalogHash` trực tiếp từ rmcp router; không duy trì danh sách tool thủ công thứ hai.

> Tổng số method phải lấy từ generated catalog/runtime thay vì hard-code trong tài liệu hoặc connector.

## Quy ước chung

Phần lớn method đều có các trường correlation chung do ChatCMD bổ sung:

- `taskId`: ID của ChatCMD task/conversation hiện tại.
- `turnId`: ID của user turn hiện tại. Mọi tool trong cùng một turn phải dùng cùng `turnId`.
- `agentId`: định danh agent gọi tool; server có thể ghi đè bằng authenticated identity.
- `requestId`: khóa idempotency do caller sinh; thường có thể bỏ qua để ChatCMD tự sinh.

Luồng agent bắt buộc:

1. `agent_user_message` là tùy chọn cho bookkeeping; khi chọn project bằng `[folder-name]` hoặc `[new project: name]`, gọi một lần gần đầu turn với nguyên văn yêu cầu của user.
2. Với mọi yêu cầu không-trivial, gọi `agent_progress` ngay sau đó để tóm tắt user yêu cầu gì và agent sẽ làm gì tiếp theo, trước `skills_list` hoặc tool substantive khác.
3. Với công việc project không tầm thường, gọi `skills_list`; nếu có skill phù hợp thì đọc bằng `skill_read` trước khi thao tác liên quan.
4. Trong lúc thực hiện, duy trì `agent_progress` theo checkpoint có ý nghĩa: thường sau khoảng 2–4 substantive operation hoặc sau một batch thao tác low-level liên quan chặt. Không cần callback theo từng tool; execution-session polling nhanh có thể gom cho đến khi trạng thái/output thay đổi đáng kể, còn lỗi/retry nên báo hướng xử lý trước khi đổi cách làm.
5. Nếu có sub-agent thì phải chờ chúng hoàn tất bằng `agent_subagent_wait`.
6. `agent_turn_complete` phải là tool cuối cùng, gọi đúng một lần ngay trước khi agent trả lời user.

### Shared project selection

Share a parent directory using the existing project management settings. A request
starting with `[folder-name]` selects a unique existing direct child directory of a
currently shared parent, whether or not it is a Git repository. The name is the
actual directory name, not the display name in the Projects management UI.
`[new project: name]` and `[새 프로젝트: name]` select the
single shared parent as the working directory for creating the named child. The
parent remains the task project after creation; use the child path explicitly for
its commands. The selector does not create a directory or initialize Git; the
agent must create the child directory before writing files. Once it exists, use
`[name]` in subsequent requests to select the child itself. See the
[workspace usage guide](WORKSPACE_USAGE.md) for the create-then-continue workflow.
One leading agent mention is accepted. A closing `]` must be followed
by whitespace or the end of the message; Markdown links are not selectors.

Call `agent_user_message` near the beginning with the exact user message, then use
its `projectFolder`, `workspace_context`, and relevant project skills. Other tools
remain directly callable. `sharedProjectSelection` reports a selection accepted
for this message. Unmatched tags, path-shaped brackets, and ordinary follow-ups do
not replace the last accepted selection. If an unmatched tag intends a different
project, clarify before modifying the previously selected project. Ambiguous
selections fail before that message is recorded and do not poison later turns.

The selection stores only a name and creation intent in the existing user-event
payload. It does not populate `tasks.project_folder` or grant paths. Every use
checks current shared scopes and canonical paths; revoked access and symlinks
outside a shared parent cannot keep a derived project accessible. Existing explicit
task project folders retain precedence. Delegated children inherit the accepted
selection as of their registration, even after ordinary child follow-ups. This
feature does not join conversations; Slack thread identity still depends on the
client's actual authenticated conversation context.

---

## 1. Execution target methods

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `execution_targets` | Không có tham số riêng | Liệt kê các execution device/máy hiện đang có thể dùng để thực thi. |
| `execution_target_get` | `deviceId` | Lấy thông tin chi tiết của một execution device cụ thể. |

---

## 2. Interactive execution session methods

Các method này quản lý interactive execution session chạy lâu dài trên connected execution runtime.

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `execution_session_create` | `workingDirectory?`, `executable?`, `arguments?`, `environment?`, `columns?`, `rows?` | Tạo một persistent interactive execution session sau khi execution policy cho phép. `workingDirectory` là field chuẩn; `cwd` và `initialWorkingDirectory` chỉ là alias tương thích. Shell chạy với quyền OS của tiến trình ChatCMD, không phải sandbox chỉ-đọc. |
| `execution_session_write` | `sessionId`, `text`, `appendNewLine?` | Gửi input literal vào terminal session. `input` là alias tương thích của `text`. |
| `execution_session_wait` | `sessionId`, `timeoutMs?` | Chờ execution session/process thay đổi hoặc kết thúc. Hết timeout không tự kill session. |
| `execution_session_read` | `sessionId`, `afterSequence?`, `maxEvents?` | Đọc output replayable của execution session theo sequence. `afterSequence` là cursor chuẩn; `startSequence`/`fromSequence` là alias tương thích. |
| `execution_session_signal` | `sessionId`, `signal` | Gửi signal portable tới terminal, ví dụ interrupt/terminate tùy signal runtime hỗ trợ. |
| `execution_session_resize` | `sessionId`, `columns`, `rows` | Resize kích thước interactive execution session. |
| `execution_session_close` | `sessionId`, `force?` | Đóng execution session; có thể force-close khi cần. |
| `execution_session_list` | Không có tham số riêng | Liệt kê các execution session hiện có. |
| `execution_session_inspect` | `sessionId` | Xem trạng thái/thông tin của một execution session. |

### Non-interactive command execution

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `execution_run` | `executable`, `cwd`, `arguments?`, `environment?`, `idempotencyKey?`, `maxStdoutBytes?`, `maxStderrBytes?`, `maxArtifactBytes?`, `timeoutMs?`, `killOnOutputLimit?` | Chạy đúng một process non-interactive sau authorization. Không shell interpolation trừ khi caller chọn shell làm executable. Result có `executionId`, `terminalState`, nullable `exitCode`/`signal`, timeout/cancel, timestamps, bounded stdout/stderr và artifact metadata. Tool success không đồng nghĩa exit 0. |

Retry cùng task/agent và idempotency key reuse execution đang chạy hoặc đã hoàn tất; cùng key nhưng
command khác trả `idempotency_conflict`. Execution lookup fail closed ngoài owner. Record hiện chỉ sống
trong process runtime, nên restart làm evidence ref cũ thành unresolved/unknown thay vì tự chạy lại.

---

## 3. Workspace methods

Các method `workspace_*` thao tác trực tiếp trong canonical workspace scope và tuân theo policy/path grant của ChatCMD.

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `workspace_roots` | Không có tham số riêng | Liệt kê các canonical workspace root mà agent được phép thao tác. |
| `workspace_context` | `targetPaths?`, `policy?`, `range?` | Đọc bounded rule bundle của project hiện tại với provenance/hash/scope. `CLAUDE.md` mặc định không tải; chỉ tải thành record riêng khi `policy.loadClaudeMd=true`, không merge ngầm với `AGENTS.md`. `range {path, offset, versionToken}` đọc chunk UTF-8 kế tiếp và fail nếu version cũ. Manifest chỉ được đọc metadata/prefix, không thực thi. |
| `workspace_list_legacy` | `path`, `offset?`, `limit?` | Legacy compatibility: trả trực tiếp mảng `FsEntry`, global sort theo tên rồi mới offset/limit; runtime cap `limit` ở 2.000. Với thư mục lớn nên dùng `workspace_list`. |
| `workspace_list` | `path`, `cursor?`, `limit?`, `sort?`, `metadata?`, `includeHidden?`, `budget?` | Cursor pagination bounded-work theo `sort=filesystem` (không hứa global alphabetical). `metadata` hỗ trợ `type`, `size`, `readonly`; mặc định `[]` để tránh stat. Result envelope v1 có `data.items`, `data.directoryVersion`, `data.sort`, `page.nextCursor/hasMore`, usage `entriesScanned/metadataCalls`, truncation/warnings khi cần. Cursor chỉ dùng lại cho cùng path/options; directory đổi thì continuation fail và phải restart. |
| `workspace_search` | `path`, `query`, `caseSensitive?`, `maxResults?`, `maxFileBytes?`, `includeIgnored?`, `exclude?` | Tìm kiếm **nội dung text** trong workspace. Khi tìm từ root nên dùng `path: "."`. |
| `workspace_find` | `path`, `pattern`, `patternMode?`, `caseSensitive?`, `entryTypes?`, `maxDepth?`, `includeIgnored?`, `includeHidden?`, `exclude?`, `extensions?`, `cursor?`, `limit?`, `budget?` (`maxResults?` legacy) | Tìm **đường dẫn/tên file hoặc thư mục** bằng traversal có early-stop và cursor. `patternMode=literal` tìm chuỗi trong filename; `glob` match path tương đối như `**/*.rs`; `regex` match regex trên path tương đối. Kết quả dùng `ToolResultEnvelope`, tiếp tục bằng `page.nextCursor` với cùng path/options. Bỏ `patternMode` giữ tương thích legacy `*foo*` literal-contains và trả warning. |
| `workspace_read_text_legacy` | `path`, `startLine?`, `lineCount?`, `maxCharacters?` | Adapter tương thích cho contract cũ; nội bộ dùng reader streaming/range, không còn tải toàn file vào RAM. Với file lớn/resumable nên dùng `workspace_read_text`. |
| `workspace_read_text` | `path`, `range { unit: line\|byte, start, limit }`, `maxBytes?`, `includeLineEndings?`, `expectedVersion?`, `budget { timeoutMs?, maxBytesRead? }?` | Reader streaming/range bounded-memory. Trả `range`, `nextStartLine`/`nextByteOffset`, `truncated` + `truncationReason`, `bytesRead`, `sizeBytes`, `versionToken`, UTF-8/BOM và newline metadata; `expectedVersion` chặn continuation stale khi file đã đổi. |
| `workspace_batch_read` | `requests[]`, `maxItems?`, `maxTotalOutputBytes?`, `concurrency?` | Đọc nhiều range qua reader v2, giữ thứ tự input, trả lỗi theo item và enforce aggregate output cap. |
| `workspace_write_text` | `path`, `content`, `overwrite?` | Ghi nguyên tử nội dung UTF-8 vào file; dùng cho tạo mới hoặc thay toàn bộ file. |
| `workspace_replace_text` | `path`, `oldText`, `newText`, `expectedOccurrences?` | Chỉnh sửa an toàn bằng exact text replacement. `oldText` phải khớp nội dung hiện tại. |
| `workspace_apply_edits` | `path`, `expectedVersion`, `coordinateSystem`, `edits`, `columnEncoding?`, `dryRun?`, `preserveLineEndings?`, `preserveBom?`, `budget?` | Sửa nhiều range UTF-8 không chồng lấn bằng streaming temp-file transaction; kiểm tra version trước xử lý và ngay trước atomic commit. |
| `workspace_write_bytes` | `path`, `base64`, `overwrite?` | Decode Base64 và ghi atomically dữ liệu binary/raw vào workspace. |
| `workspace_stat` | `path` | Xem metadata của một file/thư mục: loại entry, size, readonly, v.v. |
| `workspace_batch_stat` | `paths[]`, `versionStrength?`, `maxItems?`, `budget?` | Stat tối đa 500 path, giữ thứ tự và trả outcome riêng từng item. |
| `workspace_index_status` | `path` | Xem generation, freshness, schema, số entry và lỗi gần nhất của metadata index. |
| `workspace_index_rebuild` | `path` | Rebuild metadata index có cancellation và hard entry cap; không lưu content. |
| `workspace_create_directory` | `path` | Tạo thư mục trong workspace. |
| `workspace_copy` | `source`, `destination`, `conflictPolicy?`, `atomicPublish?`, `verify?`, `preserveMetadata?`, `followSymlinks?`, `dryRun?`, `expectedSourceVersion?`, `expectedDestinationVersion?`, `budget?`, `overwrite?` | Preflight bounded, copy vào sibling staging, verify rồi atomic publish; `overwrite` là adapter cũ cho `replace`. Symlink/reparse không được follow. |
| `workspace_move` | giống `workspace_copy` | Stage-copy → verify → publish trước khi xóa source; báo `completedWithSourceRemaining` nếu cleanup source lỗi. |
| `workspace_delete` | `path`, `recursive?`, `mode?`, `expectedVersion?`, `dryRun?`, `budget?` | `mode=quarantine` mặc định; permanent phải explicit. Root/grant root bị từ chối và traversal dùng no-follow. |

### Phân biệt nhanh `workspace_search` và `workspace_find`

- `workspace_search`: tìm **chuỗi trong nội dung file**.
- `workspace_find`: tìm **file/folder/path** theo tên/pattern.

---

## 4. Repository methods

Các Git method được thiết kế để tránh shell interpolation và truyền argument có kiểm soát.

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `repository_status` | `cwd?`, Git limits | Xem porcelain v2 và branch metadata của working tree. `path` cũ được chấp nhận như alias của `cwd`. |
| `repository_diff` | `cwd?`, `staged?`, `stat?`, `path?`, Git limits | Lấy Git diff; nên gọi `stat=true` trước khi yêu cầu patch lớn. Ext-diff và màu luôn bị tắt. |
| `repository_log` | `cwd?`, `count?`, `path?` | Xem lịch sử commit có giới hạn số lượng; có thể lọc theo path. |
| `repository_branches` | `cwd?`, Git limits | Liệt kê branch bằng format machine-readable. |
| `repository_show` | `revision`, `cwd?`, `path?` | Xem nội dung của một revision/commit đã được validate; có thể lọc theo path. |
| `repository_commit` | `message`, `cwd?`, `all?`, `paths?` | Tạo Git commit không qua shell interpolation. Phải chọn đúng một phạm vi: `paths` chuẩn hóa, không rỗng hoặc `all=true`; `all` mặc định `false`, chỉ commit thay đổi đã stage và fail closed nếu còn unstaged/untracked. Runtime từ chối preview cũ, staged path ngoài scope, đường dẫn mơ hồ và file được chọn có cả staged/unstaged changes. |

Mọi Git method nhận thêm `outputMode` (`inline` hoặc `inlineOrArtifact`),
`maxOutputBytes`, `maxStderrBytes`, `timeoutMs`, `maxRuntimeMs`,
`artifactMaxBytes` và `killOnLimit`. Mặc định timeout là 30 giây, stdout preview
512 KiB, stderr preview 128 KiB và artifact tối đa 256 MiB. Result giữ các field
cũ (`exitCode`, `stdout`, `stderr`, `truncated`) và bổ sung byte counters,
`truncationReason`, `artifactRef`, SHA-256, elapsed time, timeout/cancellation.
Git chạy với stdin/pager/credential prompt bị vô hiệu hóa; path luôn được truyền sau
`--`, không qua shell interpolation.

---

## 5. Execution process methods

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `execution_process_list` | Không có tham số riêng | Chỉ liệt kê process phát triển liên quan đến task được execution-runtime policy cho phép hiển thị; process nền không liên quan bị ẩn. |
| `execution_process_inspect` | `processId` | Xem chi tiết một process đã được execution-runtime process policy cho phép hiển thị. |
| `execution_process_stop` | `processId`, `entireTree?` | Chỉ kết thúc process đã được policy cho phép hiển thị; có thể dừng cả process tree khi `entireTree=true`. |

---

## 6. Skill methods

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `skills_list` | Không có tham số riêng | Khám phá các skill trong `.agents` và `.codex`. Với project work không tầm thường, method này phải được gọi sau `agent_user_message` và trước khi inspect/code nếu chưa biết skill phù hợp. |
| `skill_read` | `skillId` | Đọc đầy đủ instruction của một skill phù hợp. `id` là compatibility alias của `skillId`. |

---

## 7. Task methods

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `task_get` | Dùng `taskId` chung | Đọc state hiện tại của ChatCMD task. |
| `task_list` | Không có tham số riêng | Liệt kê các task. |
| `task_set_execution_mode` | `mode` | Adapter tương thích: MCP không được tự đổi quyền và luôn nhận lỗi `permission_change_requires_user`. Chỉ local UI đã xác thực có thể đổi execution mode. |
| `task_artifact_list` | Dùng `taskId` chung | Liệt kê các artifact được gắn với task. |
| `task_artifact_read` | `artifactId` | Đọc một task artifact cụ thể. |

---

## 8. Agent lifecycle / orchestration methods

Đây là nhóm method điều phối vòng đời một turn giữa ChatGPT/agent và ChatCMD server.

| Method | Tham số chính | Ý nghĩa |
|---|---|---|
| `agent_user_message` | `content` | Tùy chọn cho bookkeeping; để chọn project bằng tag, gọi một lần gần đầu turn. Đồng bộ nguyên văn user message lên ChatCMD và thiết lập/correlate `taskId` + `turnId`. `content` phải đúng nguyên văn message hiện tại. Không dùng method này cho progress/reflection/finding sau tool result; các cập nhật đó phải dùng `agent_progress`. |
| `agent_progress` | `message`, `suggestedTitle?` | **Rule phía AI cho mọi turn project không-trivial.** Ngay sau `agent_user_message` nên gửi progress tóm tắt yêu cầu + hành động kế tiếp. Sau các kết quả `workspace_*` có ý nghĩa (đặc biệt `workspace_find`, `workspace_search`, `workspace_read_text_legacy`, edit/write/delete), repository/execution, `execution_session_read`/`execution_session_wait` còn pending, sub-agent wait chưa xong, hoặc failure/non-zero, AI nên gửi progress mô tả kết quả quan sát được và bước tiếp theo trước khi tiếp tục. Đây không phải runtime gate: server không reject tool chỉ vì thiếu progress; các thao tác low-level liên quan chặt có thể gom thành một checkpoint để tránh làm chậm tiến độ và tránh callback MCP không cần thiết. Không gửi private chain-of-thought. |
| `agent_plan_question` | `question`, `options`, `questionKind?` | `questionKind` mặc định `clarification`; `executionConsent` dùng semantics consent do server định nghĩa. Lifecycle được audit durable; restart/disconnect/timeout/custom answer fail closed. Approved consent không đổi execution mode, không mint grant và mọi side effect vẫn qua C01 tool authorization. |
| `agent_subagent_start` | `name`, `request` | Tạo hoặc reuse child. `samplingTools`/`samplingText` là worker sampling; `extensionFallback` là child pending để browser extension claim nên parent không làm trùng; `existing` không spawn lại. Startup lỗi sau registration trả structured `status=failed` + `startupError`. |
| `agent_subagent_wait` | `timeoutMs?`, `subagentId?`, `reportOffset?`, `reportVersion?` | Chờ toàn bộ cây agent của parent turn và trả báo cáo công khai trong `subagents[].report.content`. `allFinished`/`allCompleted` chỉ là lifecycle; kiểm tra `workOutcome`, các bộ đếm lỗi và báo cáo thiếu. Nếu `allFinished=false` hoặc `reportPendingCount>0` thì tiếp tục gọi lại. Báo cáo dài trả `report.continuation` để truyền lại vào tool, không cần đọc lại repo. Xem [hợp đồng báo cáo sub-agent](subagent-reports.md). |
| `agent_turn_complete` | `content`, `suggestedTitle?`, `workOutcome?`, `verificationIntent?`, `verificationReason?`, `verificationScope?`, `criteria?`, `evidenceRefs?`, `blockers?`, `limitations?` | **Bắt buộc là MCP call cuối cùng.** Xác nhận turn đã hoàn tất và gửi đúng nội dung cuối cùng agent sẽ trả cho user. `workOutcome` là agent assessment; verification do server resolve từ `execution_run` execution IDs. Client cũ chỉ gửi `content` vẫn hợp lệ và được normalize thành legacy completed + `notRun`, không phải verified. |

Lưu ý: `workspace_find`, `workspace_search`, `workspace_read_text_legacy`, các tool sửa file, shell, Git... **không tạo thêm `agent_user_message`**. `agent_user_message` chỉ đại diện cho message thật của user ở đầu turn. Sau kết quả của các tool này, message cập nhật gửi cho user phải đi qua `agent_progress`.

---

## 9. Generated catalog, version và cache invalidation

`TOOL_NAMES` được sinh từ chính `McpServer::tool_router().list_all()` và sort deterministic. Không copy danh sách tool sang connector, UI, release script hoặc tài liệu.

Canonical manifest chứa `protocolVersion`, `catalogVersion` và với mỗi tool có `name`, normalized input schema, `resultSchema` cùng capability flags. Tool chưa migrate result contract có `resultSchema: null` và `resultSchemaVersion: null`; `workspace_list` và `workspace_find` quảng bá `resultSchemaVersion: 1` cùng generated JSON schema của `ToolResultEnvelope<...>` tương ứng. Trước khi hash SHA-256, object keys được sort và metadata chỉ để mô tả như `description`/`title` được bỏ khỏi contract; vì vậy đổi wording không làm invalid cache, còn đổi input/result schema hoặc capability sẽ làm đổi `catalogHash`.

Chi tiết semantics, cursor/error code, migration inventory và các ví dụ complete/paged/truncated/content-backed nằm tại `docs/tool_result_envelope.md`.

Contract coding-agent, completion quality, compatibility và rollout nằm tại
[`coding-agent-contract.md`](coding-agent-contract.md).

Metadata runtime gồm `appVersion`, `protocolVersion`, `catalogVersion`, `catalogHash`, `instructionsVersion`, `instructionsHash`, `buildId`. `catalogHash` chỉ theo contract cấu trúc; instruction bundle và behavior descriptions có hash/version riêng để wording không làm invalid schema cache. MCP initialize trả metadata dưới prefix `CHATCMD_CATALOG_METADATA=...` trong server instructions. HTTP host cũng expose endpoint authenticated `GET /mcp/{token}/catalog` để diagnostics lấy metadata + canonical manifest; token vẫn chỉ ở auth boundary và không được ghi vào structured catalog log.

Caller có thể gửi `clientCatalogHash` trong common tool arguments. Nếu hash khác server, request fail-fast với `error.code = "catalog_mismatch"`, kèm cả `clientCatalogHash`, `serverCatalogHash` và recovery instruction. Connector phải bỏ schema cache cũ, reconnect/initialize/list_tools lại và chỉ retry operation tối đa một lần sau refresh để tránh retry loop.

Tool-level structured error có `code`, `message`, `retryable`, `approvalRequired`, `phase`, `outcome` và `recovery`; `usage` nằm cạnh error khi runtime có số liệu. `outcome` phân biệt `notStarted`, `unchanged`, `unknown`; riêng unknown/partial side effect phải inspect state trước khi retry. Exit code khác 0, timeout hoặc cancellation của command vẫn là command outcome trong result, không bị đổi thành tool error giả.

Release gate cho catalog là `cargo test -p chatcmd-mcp --test release_catalog_smoke`: test spawn binary `catalog_smoke_server` qua stdio transport thật hai lần, gọi MCP initialize/list_tools, bắt buộc có các contract mới và so names + normalized schema với canonical manifest. Workflow adversarial chạy gate này trong platform matrix; desktop DEV packaging vẫn chỉ được khởi động thủ công bằng `workflow_dispatch`.

---

## 10. Luồng gọi mẫu

Một turn sửa code thông thường có thể có flow:

```text
agent_user_message
  -> agent_progress              (ngay lập tức: tóm tắt user yêu cầu gì + bước tiếp theo)
  -> skills_list
  -> skill_read                  (nếu có skill phù hợp)
  -> workspace_roots / workspace_find
  -> workspace_read_text_legacy / workspace_search    (có thể gom các read/search liên quan thành một batch)
  -> agent_progress              (báo finding chính sau batch inspect/search)
  -> workspace_replace_text / workspace_write_text
  -> agent_progress              (báo file vừa đổi gì và tác động chính nếu đáng báo)
  -> repository_diff / repository_status       (nếu cần kiểm tra thay đổi)
  -> agent_progress              (báo kết quả verify/Git đáng chú ý)
  -> agent_turn_complete
```

Cadence mặc định là khoảng **2–4 substantive operation hoặc hết một coherent batch**, không phải một progress cho mỗi tool call. Nếu có finding quan trọng, lỗi, hoặc chuyển phase thì có thể báo sớm hơn.

Một turn có sub-agent:

```text
agent_user_message
  -> agent_progress              (tóm tắt yêu cầu + kế hoạch chia việc)
  -> skills_list
  -> agent_subagent_start
  -> agent_progress              (báo child đã được dispatch hoặc lỗi dispatch + fallback)
  -> ... parent tiếp tục công việc, vẫn giữ cadence progress ...
  -> agent_subagent_wait
  -> agent_progress              (nếu child vẫn chưa xong, báo đang chờ gì)
  -> agent_subagent_wait         (lặp nếu allFinished=false)
  -> agent_turn_complete
```

Một turn chạy terminal dài:

```text
agent_user_message
  -> agent_progress              (tóm tắt command/workflow sắp chạy)
  -> execution_session_create
  -> execution_session_write
  -> execution_session_wait / execution_session_read     (có thể poll ngắn liên tiếp)
  -> agent_progress              (nếu vẫn chạy lâu: báo stage/output hiện tại + đang chờ gì)
  -> execution_session_wait / execution_session_read     (tiếp tục poll; không lặp message nếu trạng thái chưa đổi)
  -> agent_progress              (khi có thay đổi đáng kể, kết quả cuối, hoặc lỗi + hướng recovery)
  -> execution_session_close                 (nếu cần đóng session)
  -> agent_turn_complete
```

Nếu command/tool trả lỗi hoặc exit code khác 0, `agent_progress` phải xuất hiện **trước** lần retry, đổi command hoặc fallback tiếp theo; không được retry âm thầm.

---

## 11. Lưu ý khi bổ sung MCP method mới

Khi thêm/xóa/đổi tên MCP tool, cần đồng bộ ít nhất:

1. `crates/chatcmd-mcp/src/lib.rs` — thêm/sửa schema argument + tool description + handler trong rmcp router. `TOOL_NAMES` và canonical manifest sẽ tự sinh từ router này.
2. Runtime dispatch/handler tương ứng ở phía ChatCMD nếu method cần xử lý mới.
3. Xác định capability flags trong `tool_catalog.rs` nếu semantics mới không được rule hiện tại bao phủ.
4. Chạy invariant tests catalog/schema/dispatcher và `release_catalog_smoke`; mọi thay đổi contract hợp lệ phải làm `catalogHash` thay đổi.
5. Cập nhật tài liệu về semantics nếu cần, nhưng không copy lại full tool list để tránh drift.
