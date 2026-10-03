use chatcmd_core::{ExecutionMode, TaskExecutionMode, TaskId, TaskStore as _};
use chatcmd_runtime::OperationContext;
use serde_json::json;
use tempfile::TempDir;

use crate::runtime_host::user_message_tests::{test_host, turn_context};
use crate::runtime_host::*;

async fn context(host: &RuntimeHost, agent: &str, id: &str) -> OperationContext {
    let accepted = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                &format!("user-{id}"),
                agent,
                "agent_user_message",
                &format!("turn-{id}"),
                id,
            ),
            json!({"content":"Inspect this workspace"}),
        )
        .await
        .unwrap();
    let mut context = OperationContext::new(format!("read-{id}"), agent, "fs_read_text");
    context.task_id = accepted["taskId"].as_str().map(str::to_owned);
    context.turn_id = accepted["turnId"].as_str().map(str::to_owned);
    context.mcp_session_id = accepted["sessionId"].as_str().map(str::to_owned);
    context
}

async fn register(host: &RuntimeHost, directory: &std::path::Path, enabled: bool) {
    let canonical = directory
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    sqlx::query("INSERT INTO workspace_projects(id,name,path,canonical_path,allow_all_conversations,global_access_path,created_at_ms,updated_at_ms) VALUES('shared','Shared',?,?,?, ?,0,0)")
        .bind(&canonical).bind(&canonical).bind(enabled).bind(enabled.then_some(&canonical))
        .execute(host.repository.pool()).await.unwrap();
}

async fn roots(host: &RuntimeHost, context: &OperationContext) -> serde_json::Value {
    host.dispatch("workspace_roots", context.clone(), json!({}))
        .await
        .unwrap()
}

#[path = "shared_project_selection_tests.rs"]
mod selection;

#[tokio::test]
async fn opt_in_adds_alias_to_existing_conversations_and_revokes_reads_and_writes() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    std::fs::write(external.path().join("hello.txt"), "hello").unwrap();
    let a = context(&host, &agent, "first").await;
    let b = context(&host, &agent, "second").await;
    register(&host, external.path(), false).await;
    let before = roots(&host, &a).await;
    let path = external.path().join("hello.txt");
    assert!(
        host.dispatch("fs_read_text", a.clone(), json!({"path":path}))
            .await
            .is_err()
    );
    sqlx::query("UPDATE workspace_projects SET allow_all_conversations=1,global_access_path=path WHERE id='shared'")
        .execute(host.repository.pool()).await.unwrap();
    let after = roots(&host, &a).await;
    assert_eq!(
        after.as_array().unwrap().len(),
        before.as_array().unwrap().len() + 1
    );
    assert_eq!(after, roots(&host, &b).await);
    assert!(
        !after
            .to_string()
            .contains(external.path().to_str().unwrap())
    );
    let view = host.virtual_workspace_view(&a).await;
    let alias = view.project_path(external.path());
    assert!(alias.starts_with('@'));
    assert!(
        host.dispatch(
            "fs_read_text",
            a.clone(),
            json!({"path":format!("{alias}/hello.txt")})
        )
        .await
        .is_ok()
    );
    assert!(
        host.dispatch(
            "fs_write_text",
            b.clone(),
            json!({"path":format!("{alias}/new.txt"),"content":"new"})
        )
        .await
        .is_ok()
    );
    sqlx::query("UPDATE workspace_projects SET allow_all_conversations=0,global_access_path=NULL WHERE id='shared'")
        .execute(host.repository.pool()).await.unwrap();
    assert_eq!(roots(&host, &a).await, before);
    assert!(
        host.dispatch("fs_read_text", a.clone(), json!({"path":path}))
            .await
            .is_err()
    );
    assert!(
        host.dispatch(
            "fs_read_text",
            b.clone(),
            json!({"path":format!("{alias}/hello.txt")})
        )
        .await
        .is_err()
    );
    assert!(
        host.dispatch(
            "fs_write_text",
            b,
            json!({"path":external.path().join("new.txt"),"content":"denied"})
        )
        .await
        .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(external.path().join("new.txt")).unwrap(),
        "new"
    );
}

#[tokio::test]
async fn shared_roots_are_visible_with_a_task_project_and_delete_revokes_only_shared_scope() {
    let (host, agent, _root) = test_host().await;
    let bound = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let context = context(&host, &agent, "bound").await;
    sqlx::query("UPDATE tasks SET project_folder=? WHERE id=?")
        .bind(bound.path().to_str().unwrap())
        .bind(context.task_id.as_deref())
        .execute(host.repository.pool())
        .await
        .unwrap();
    register(&host, external.path(), true).await;
    let roots = roots(&host, &context).await;
    assert_eq!(roots[0], "@project");
    assert_eq!(roots.as_array().unwrap().len(), 2);
    sqlx::query("DELETE FROM workspace_projects WHERE id='shared'")
        .execute(host.repository.pool())
        .await
        .unwrap();
    assert_eq!(self::roots(&host, &context).await, json!(["@project"]));
    assert!(
        host.dispatch("fs_stat", context.clone(), json!({"path":external.path()}))
            .await
            .is_err()
    );
    assert!(
        host.dispatch("fs_stat", context, json!({"path":"@project"}))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn opt_in_never_bypasses_tool_allowlists_or_execution_deny() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    register(&host, external.path(), true).await;
    let context = context(&host, &agent, "policy").await;
    assert!(host.authorize_tool(&agent, "fs_write_text").await.is_err());
    host.repository
        .set_execution_mode(&TaskExecutionMode {
            task_id: TaskId::new(context.task_id.as_deref().unwrap()).unwrap(),
            mode: ExecutionMode::Deny,
            updated_at_ms: 0,
        })
        .await
        .unwrap();
    let error = host
        .authorize_execution(&context, "fs_read_text", &json!({"path":external.path()}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "policy_denied");
}

#[tokio::test]
async fn public_results_mask_shared_paths_and_path_traversal_remains_denied() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    std::fs::write(external.path().join("hello.txt"), "hello").unwrap();
    register(&host, external.path(), true).await;
    let context = context(&host, &agent, "masked").await;
    host.repository
        .set_execution_mode(&TaskExecutionMode {
            task_id: TaskId::new(context.task_id.as_deref().unwrap()).unwrap(),
            mode: ExecutionMode::Allow,
            updated_at_ms: 0,
        })
        .await
        .unwrap();
    let alias = host
        .virtual_workspace_view(&context)
        .await
        .project_path(external.path());
    let result = host
        .call_persisted(
            "fs_read_text",
            context.clone(),
            json!({"path":format!("{alias}/hello.txt")}),
        )
        .await
        .unwrap();
    assert!(
        !result
            .to_string()
            .contains(external.path().to_str().unwrap())
    );
    assert!(
        host.dispatch(
            "fs_stat",
            context,
            json!({"path":format!("{alias}/../outside")})
        )
        .await
        .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn shared_symlink_cannot_escape_or_retarget_an_approved_directory() {
    use std::os::unix::fs::symlink;
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    let other = TempDir::new().unwrap();
    let granted = external.path().join("granted");
    std::fs::create_dir(&granted).unwrap();
    std::fs::write(other.path().join("secret"), "secret").unwrap();
    register(&host, &granted, true).await;
    let context = context(&host, &agent, "symlink").await;
    symlink(other.path(), granted.join("escape")).unwrap();
    assert!(
        host.dispatch(
            "fs_read_text",
            context.clone(),
            json!({"path":granted.join("escape/secret")})
        )
        .await
        .is_err()
    );
    std::fs::rename(&granted, external.path().join("old")).unwrap();
    symlink(other.path(), &granted).unwrap();
    assert!(host.shared_project_scopes().await.unwrap().is_empty());
    assert!(
        host.dispatch(
            "fs_read_text",
            context,
            json!({"path":granted.join("secret")})
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn duplicate_shared_paths_have_one_alias_and_no_task_receives_no_roots() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    register(&host, external.path(), true).await;
    sqlx::query("INSERT INTO workspace_projects(id,name,path,canonical_path,allow_all_conversations,global_access_path,created_at_ms,updated_at_ms) SELECT 'duplicate','Duplicate',path,'different-key',1,global_access_path,0,0 FROM workspace_projects WHERE id='shared'")
        .execute(host.repository.pool()).await.unwrap();
    assert_eq!(host.shared_project_scopes().await.unwrap().len(), 1);
    assert_eq!(
        roots(
            &host,
            &OperationContext::new("no-task", &agent, "workspace_roots")
        )
        .await,
        json!([])
    );
}

#[tokio::test]
async fn pending_approval_cannot_resurrect_a_revoked_shared_path() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    std::fs::write(external.path().join("hello.txt"), "hello").unwrap();
    register(&host, external.path(), true).await;
    let mut context = context(&host, &agent, "pending").await;
    context.request_id = "pending-shared-read".to_owned();
    let alias = host
        .virtual_workspace_view(&context)
        .await
        .project_path(external.path());
    let runner = host.clone();
    let call = tokio::spawn(async move {
        runner
            .call_persisted(
                "fs_read_text",
                context,
                json!({"path":format!("{alias}/hello.txt")}),
            )
            .await
    });
    let mut pending = false;
    for _ in 0..100 {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM approvals WHERE id='pending-shared-read' AND state='pending'",
        )
        .fetch_one(host.repository.pool())
        .await
        .unwrap();
        if count == 1 {
            pending = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(
        pending,
        "read must still require approval after project opt-in"
    );
    sqlx::query("UPDATE workspace_projects SET allow_all_conversations=0,global_access_path=NULL WHERE id='shared'")
        .execute(host.repository.pool()).await.unwrap();
    sqlx::query("UPDATE approvals SET state='approved',decision_json='{}',resolved_at_ms=1 WHERE id='pending-shared-read'")
        .execute(host.repository.pool()).await.unwrap();
    assert!(
        call.await.unwrap().is_err(),
        "approval cannot restore revoked filesystem scope"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn existing_terminal_cannot_accept_input_after_shared_scope_revocation() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    register(&host, external.path(), true).await;
    let context = context(&host, &agent, "terminal").await;
    let session = host
        .dispatch(
            "shell_create",
            context.clone(),
            json!({
                "workingDirectory":external.path(),"executable":"/bin/sh",
            }),
        )
        .await
        .unwrap();
    let id = session["sessionId"].as_str().unwrap();
    sqlx::query("DELETE FROM workspace_projects WHERE id='shared'")
        .execute(host.repository.pool())
        .await
        .unwrap();
    let result = host
        .dispatch(
            "shell_write",
            context.clone(),
            json!({"sessionId":id,"text":"echo should-not-run"}),
        )
        .await;
    assert_eq!(result.unwrap_err().code, "policy_denied");
    assert!(
        host.dispatch("shell_read", context.clone(), json!({"sessionId":id}))
            .await
            .is_err()
    );
    host.dispatch("shell_close", context, json!({"sessionId":id,"force":true}))
        .await
        .unwrap();
}

#[tokio::test]
async fn shared_alias_never_remaps_when_another_same_named_folder_is_removed() {
    let (host, agent, _root) = test_host().await;
    let external = TempDir::new().unwrap();
    let first = external.path().join("one/game");
    let second = external.path().join("two/game");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    register(&host, &first, true).await;
    sqlx::query("INSERT INTO workspace_projects(id,name,path,canonical_path,allow_all_conversations,global_access_path,created_at_ms,updated_at_ms) VALUES('second','Second',?,?,1,?,0,0)")
        .bind(second.to_str().unwrap()).bind(second.to_str().unwrap()).bind(second.to_str().unwrap())
        .execute(host.repository.pool()).await.unwrap();
    let context = context(&host, &agent, "stable-alias").await;
    let before = host.virtual_workspace_view(&context).await;
    let first_alias = before.project_path(&first);
    let second_alias = before.project_path(&second);
    assert_ne!(first_alias, second_alias);
    sqlx::query("DELETE FROM workspace_projects WHERE id='shared'")
        .execute(host.repository.pool())
        .await
        .unwrap();
    let after = host.virtual_workspace_view(&context).await;
    assert_eq!(after.project_path(&second), second_alias);
    assert!(!after.aliases().contains(&first_alias));
}
