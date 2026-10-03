use std::collections::BTreeSet;

use chatcmd_core::{TaskId, TaskStore as _};
use chatcmd_runtime::{RuntimeError, RuntimeResult};
use serde_json::{Value, json};

use super::{RuntimeHost, invalid, storage_error};

impl RuntimeHost {
    pub(super) async fn task_project_folder(
        &self,
        task_id: Option<&str>,
    ) -> RuntimeResult<Option<String>> {
        let Some(task_id) = task_id.filter(|value| !value.trim().is_empty()) else {
            return Ok(None);
        };
        let id = TaskId::new(task_id).map_err(|error| invalid("taskId", error))?;
        let folder = self
            .repository
            .task(&id)
            .await
            .map_err(storage_error)?
            .and_then(|task| task.project_folder)
            .filter(|folder| !folder.trim().is_empty());
        if folder.is_some() {
            return Ok(folder);
        }
        self.shared_task_project_folder(task_id).await
    }

    /// Record only a validated name, never a physical path or a lasting access grant.
    pub(super) async fn shared_project_selection(
        &self,
        content: &str,
    ) -> RuntimeResult<Option<Value>> {
        let Some((new_project, name)) = shared_project_selector(content) else {
            return Ok(None);
        };
        if self
            .resolve_shared_project_selection(new_project, name)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(json!({"newProject": new_project, "name": name})))
    }

    async fn shared_task_project_folder(&self, task_id: &str) -> RuntimeResult<Option<String>> {
        // Filter accepted selections before limiting: ordinary child messages cannot hide
        // the inherited selection. Preserve the existing delegation-time ancestor cutoff.
        let selection = sqlx::query_scalar::<_, String>(
            "WITH RECURSIVE selected_tasks(id,before_ms,depth) AS (SELECT ?,9223372036854775807,0 UNION ALL SELECT parent.parent_task_id,MIN(selected_tasks.before_ms,parent.created_at_ms),selected_tasks.depth+1 FROM subagent_runs parent JOIN selected_tasks ON parent.child_task_id=selected_tasks.id WHERE selected_tasks.depth<32) SELECT json_extract(event.payload_json,'$.sharedProjectSelection') FROM timeline_events event JOIN selected_tasks ON event.task_id=selected_tasks.id WHERE event.actor='user' AND event.kind='message' AND event.created_at_ms<=selected_tasks.before_ms AND json_type(event.payload_json,'$.sharedProjectSelection')='object' ORDER BY selected_tasks.depth,event.created_at_ms DESC,event.event_id DESC LIMIT 1",
        ).bind(task_id).fetch_optional(self.repository.pool()).await
            .map_err(|_| RuntimeError::new("storage_error", "project selection unavailable"))?;
        let Some(selection) = selection else {
            return Ok(None);
        };
        let selection: Value = serde_json::from_str(&selection)
            .map_err(|_| RuntimeError::new("storage_error", "project selection unavailable"))?;
        let name = selection["name"]
            .as_str()
            .ok_or_else(|| RuntimeError::new("storage_error", "project selection unavailable"))?;
        let new_project = selection["newProject"]
            .as_bool()
            .ok_or_else(|| RuntimeError::new("storage_error", "project selection unavailable"))?;
        // Revalidate even persisted names against currently approved scopes.
        if !valid_project_name(name) {
            return Ok(None);
        }
        self.resolve_shared_project_selection(new_project, name)
            .await
    }

    async fn resolve_shared_project_selection(
        &self,
        new_project: bool,
        name: &str,
    ) -> RuntimeResult<Option<String>> {
        let scopes = self.shared_project_scopes().await?;
        if new_project {
            // Keep the parent stable while the requested child is being created.
            return match scopes.as_slice() {
                [root] => Ok(Some(root.to_string_lossy().into_owned())),
                [] => Ok(None),
                _ => Err(RuntimeError::new(
                    "project_selector_ambiguous",
                    "new project requires one shared parent; specify the intended parent",
                )),
            };
        }
        let mut matches = BTreeSet::new();
        for root in scopes {
            let candidate = root.join(name);
            if let Ok(canonical) = candidate.canonicalize()
                && canonical.is_dir()
                && canonical.starts_with(&root)
            {
                matches.insert(canonical);
            }
        }
        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches
                .into_iter()
                .next()
                .map(|path| path.to_string_lossy().into_owned())),
            _ => Err(RuntimeError::new(
                "project_selector_ambiguous",
                "more than one shared parent contains this project; specify the intended project",
            )),
        }
    }
}

fn valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && !matches!(name, "." | "..")
        && !name.contains(['/', '\\', '[', ']', ':'])
        && !name.chars().any(char::is_control)
}

fn shared_project_selector(content: &str) -> Option<(bool, &str)> {
    let mut content = content.trim_start();
    // Slack can prefix the exact request with a single agent mention.
    if content.starts_with("<@") {
        let (mention, rest) = content.split_once('>')?;
        if mention[2..].is_empty() || mention[2..].chars().any(char::is_whitespace) {
            return None;
        }
        content = rest.trim_start();
    } else if content.starts_with('@') {
        let end = content.find(char::is_whitespace)?;
        if end == 1 {
            return None;
        }
        content = content[end..].trim_start();
    }
    let (selector, rest) = content.strip_prefix('[')?.split_once(']')?;
    // A Markdown link is ordinary text, not a project selector.
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let selector = selector.trim();
    let (new_project, name) = match selector
        .strip_prefix("새 프로젝트:")
        .or_else(|| selector.strip_prefix("new project:"))
    {
        Some(name) => (true, name.trim()),
        None => (false, selector),
    };
    valid_project_name(name).then_some((new_project, name))
}
