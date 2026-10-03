use super::*;

#[tokio::test]
async fn shared_project_selector_loads_rules_and_skills_for_separate_conversations() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    register(&host, parent.path(), true).await;
    let mut tasks = Vec::new();
    for name in ["repo-one", "repo-two"] {
        let project = parent.path().join(name);
        let skill = project.join(".agents/skills/project-proof");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(project.join("AGENTS.md"), name).unwrap();
        std::fs::write(
            skill.join("SKILL.md"),
            format!("---\nname: project-proof\ndescription: {name} coding guidance\n---\n{name}"),
        )
        .unwrap();
        let accepted = host
            .call_persisted(
                "agent_user_message",
                turn_context(name, &agent, "agent_user_message", name, name),
                json!({"content":format!("[{name}] Inspect the project")}),
            )
            .await
            .unwrap();
        assert_eq!(accepted["projectFolder"], "@project");
        assert_eq!(accepted["projectContext"]["ruleCount"], 1);
        let mut context = OperationContext::new(name, &agent, "fs_read_text");
        context.task_id = accepted["taskId"].as_str().map(str::to_owned);
        tasks.push(context.task_id.clone());
        let rules = host
            .dispatch("project_context", context.clone(), json!({}))
            .await
            .unwrap();
        assert!(rules.to_string().contains(name));
        let skills = host
            .dispatch("skills_list", context.clone(), json!({}))
            .await
            .unwrap();
        assert_eq!(skills[0]["name"], "project-proof");
        assert!(skills[0]["description"].as_str().unwrap().contains(name));
        let read = host
            .dispatch("fs_read_text", context.clone(), json!({"path":"AGENTS.md"}))
            .await
            .unwrap();
        assert_eq!(read["content"], name);
        let stored = host
            .repository
            .task(&TaskId::new(context.task_id.as_deref().unwrap()).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.project_folder, None, "sharing must remain revocable");
    }
    assert_ne!(tasks[0], tasks[1]);
}

#[tokio::test]
async fn shared_project_selector_survives_followup_but_not_revocation() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    let project = parent.path().join("repo-one");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("AGENTS.md"), "project guidance").unwrap();
    register(&host, parent.path(), true).await;
    let mut accepted = serde_json::Value::Null;
    for (turn, content) in [
        ("first", "[repo-one] Inspect"),
        ("followup", "Continue"),
        ("wip", "[WIP] continue"),
        ("markdown", "[repo-one](https://example.com) follow up"),
        ("file", "[src/a.rs] fix this"),
        ("mention", "@bot fix arr[0]"),
        ("traversal-note", "[../outside] is an invalid path"),
    ] {
        accepted = host
            .call_persisted(
                "agent_user_message",
                turn_context(
                    turn,
                    &agent,
                    "agent_user_message",
                    turn,
                    "selector-followup",
                ),
                json!({"content":content}),
            )
            .await
            .unwrap();
        assert_eq!(accepted["projectFolder"], "@project");
    }
    sqlx::query("UPDATE workspace_projects SET allow_all_conversations=0,global_access_path=NULL WHERE id='shared'")
        .execute(host.repository.pool()).await.unwrap();
    let mut context = OperationContext::new("revoked-read", &agent, "fs_read_text");
    context.task_id = accepted["taskId"].as_str().map(str::to_owned);
    assert!(
        host.dispatch(
            "fs_read_text",
            context.clone(),
            json!({"path":project.join("AGENTS.md")})
        )
        .await
        .is_err()
    );
    assert!(
        host.dispatch("project_context", context.clone(), json!({}))
            .await
            .is_err()
    );
    assert!(
        host.dispatch("skills_list", context, json!({}))
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn new_shared_project_selector_keeps_parent_stable_after_creation() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    register(&host, parent.path(), true).await;
    let accepted = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                "new-project",
                &agent,
                "agent_user_message",
                "new-project",
                "new-project",
            ),
            json!({"content":"[새 프로젝트: demo] Create a calculator"}),
        )
        .await
        .unwrap();
    let task = accepted["taskId"].as_str().unwrap();
    let before = <RuntimeHost as chatcmd_mcp::RuntimeApi>::project_folder(&host, Some(task))
        .await
        .unwrap();
    std::fs::create_dir(parent.path().join("demo")).unwrap();
    let after = <RuntimeHost as chatcmd_mcp::RuntimeApi>::project_folder(&host, Some(task))
        .await
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(
        after,
        Some(parent.path().canonicalize().unwrap().display().to_string())
    );
}

#[tokio::test]
async fn shared_project_selector_child_retains_the_delegated_project() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    for name in ["repo-one", "repo-two"] {
        std::fs::create_dir(parent.path().join(name)).unwrap();
    }
    register(&host, parent.path(), true).await;
    sqlx::query(
        "INSERT INTO settings(key,value_json,updated_at_ms) VALUES('ui_subagentConcurrency','2',0)",
    )
    .execute(host.repository.pool())
    .await
    .unwrap();
    let accepted = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                "parent-first",
                &agent,
                "agent_user_message",
                "parent-first",
                "selector-parent",
            ),
            json!({"content":"[repo-one] Inspect"}),
        )
        .await
        .unwrap();
    let mut context = OperationContext::new("child-start", &agent, "agent_subagent_start");
    context.task_id = accepted["taskId"].as_str().map(str::to_owned);
    context.turn_id = accepted["turnId"].as_str().map(str::to_owned);
    let registered = host
        .register_subagent(&context, "Reader", "Read project rules", None)
        .await
        .unwrap();
    let child = registered["childTaskId"].as_str().unwrap();
    let expected = parent
        .path()
        .join("repo-one")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    assert_eq!(
        <RuntimeHost as chatcmd_mcp::RuntimeApi>::project_folder(&host, Some(child))
            .await
            .unwrap(),
        Some(expected.clone())
    );
    let mut child_context = OperationContext::new("child-message", &agent, "agent_user_message");
    child_context.task_id = Some(child.to_owned());
    child_context.turn_id = Some("child-first".to_owned());
    let synced = host.call_persisted("agent_user_message", child_context.clone(),
        json!({"content":format!("Read rules\nCMDGPT_SUBAGENT_ID={}",registered["subagentId"].as_str().unwrap())})
    ).await.unwrap();
    child_context.mcp_session_id = synced["sessionId"].as_str().map(str::to_owned);
    for index in 0..40 {
        child_context.turn_id = Some(format!("child-note-{index}"));
        host.save_user_message(&child_context, &format!("Review arr[{index}]"))
            .await
            .unwrap();
    }
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    host.call_persisted(
        "agent_user_message",
        turn_context(
            "parent-second",
            &agent,
            "agent_user_message",
            "parent-second",
            "selector-parent",
        ),
        json!({"content":"[repo-two] Inspect"}),
    )
    .await
    .unwrap();
    assert_eq!(
        <RuntimeHost as chatcmd_mcp::RuntimeApi>::project_folder(&host, Some(child))
            .await
            .unwrap(),
        Some(expected)
    );
    sqlx::query("UPDATE workspace_projects SET allow_all_conversations=0,global_access_path=NULL WHERE id='shared'").execute(host.repository.pool()).await.unwrap();
    assert_eq!(
        <RuntimeHost as chatcmd_mcp::RuntimeApi>::project_folder(&host, Some(child))
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn shared_project_selector_rejects_traversal_and_ambiguous_names() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    std::fs::create_dir(parent.path().join("repo-one")).unwrap();
    register(&host, parent.path(), true).await;
    let second = TempDir::new().unwrap();
    std::fs::create_dir(second.path().join("repo-one")).unwrap();
    let path = second.path().canonicalize().unwrap().display().to_string();
    sqlx::query("INSERT INTO workspace_projects(id,name,path,canonical_path,allow_all_conversations,global_access_path,created_at_ms,updated_at_ms) VALUES('shared-second','Second',?,?,1,?,0,0)")
        .bind(&path).bind(&path).bind(&path).execute(host.repository.pool()).await.unwrap();
    let ignored = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                "traversal",
                &agent,
                "agent_user_message",
                "traversal",
                "traversal",
            ),
            json!({"content":"[../outside] Inspect"}),
        )
        .await
        .unwrap();
    assert!(ignored["projectFolder"].is_null());
    let error = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                "ambiguous",
                &agent,
                "agent_user_message",
                "ambiguous",
                "ambiguous",
            ),
            json!({"content":"[repo-one] Inspect"}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "project_selector_ambiguous");
    let followup = host
        .call_persisted(
            "agent_user_message",
            turn_context(
                "after-error",
                &agent,
                "agent_user_message",
                "after-error",
                "ambiguous",
            ),
            json!({"content":"Continue without a project"}),
        )
        .await
        .unwrap();
    assert!(followup["projectFolder"].is_null());
}

#[cfg(unix)]
#[tokio::test]
async fn shared_project_selector_cannot_follow_an_escaping_symlink() {
    let (host, agent, _configured) = test_host().await;
    let parent = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), parent.path().join("escape")).unwrap();
    register(&host, parent.path(), true).await;
    let accepted = host
        .call_persisted(
            "agent_user_message",
            turn_context("escape", &agent, "agent_user_message", "escape", "escape"),
            json!({"content":"[escape] Inspect"}),
        )
        .await
        .unwrap();
    assert!(accepted["projectFolder"].is_null());
}
