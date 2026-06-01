use std::process::Command;

use anyhow::{Context, Result, anyhow};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct NewWorkspaceSpec {
    pub name: String,
    pub cwd: Option<String>,
    pub layout: Value,
}

impl NewWorkspaceSpec {
    pub fn layout_json(&self) -> Result<String> {
        serde_json::to_string(&self.layout).context("failed to serialize cmux layout JSON")
    }

    pub fn pretty_layout_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.layout).context("failed to serialize cmux layout JSON")
    }
}

pub fn ensure_available() -> Result<()> {
    let output = Command::new("cmux")
        .arg("--help")
        .output()
        .context("failed to execute cmux; is the upstream cmux binary in PATH?")?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux binary exists but `cmux --help` failed: {}",
            stderr_or_status(&output)
        ));
    }

    Ok(())
}

pub fn new_window() -> Result<String> {
    let output = Command::new("cmux")
        .arg("rpc")
        .arg("window.create")
        .arg("{}")
        .output()
        .context("failed to execute cmux rpc window.create")?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux rpc window.create failed: {}",
            stderr_or_status(&output)
        ));
    }

    parse_window_create_id(&output.stdout)
}

pub fn workspace_refs(window: &str) -> Result<Vec<String>> {
    let output = Command::new("cmux")
        .arg("list-workspaces")
        .arg("--window")
        .arg(window)
        .arg("--json")
        .output()
        .with_context(|| format!("failed to execute cmux list-workspaces for {window}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux list-workspaces failed for {window}: {}",
            stderr_or_status(&output)
        ));
    }

    parse_workspace_refs(&output.stdout)
}

pub fn new_workspace(spec: &NewWorkspaceSpec, window: &str, focus: bool) -> Result<()> {
    let layout_json = spec.layout_json()?;
    let mut command = Command::new("cmux");
    command.arg("new-workspace").arg("--name").arg(&spec.name);

    if let Some(cwd) = &spec.cwd {
        command.arg("--cwd").arg(cwd);
    }

    let output = command
        .arg("--layout")
        .arg(layout_json)
        .arg("--window")
        .arg(window)
        .arg("--focus")
        .arg(if focus { "true" } else { "false" })
        .output()
        .with_context(|| format!("failed to execute cmux new-workspace for {}", spec.name))?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux new-workspace failed for {}: {}",
            spec.name,
            stderr_or_status(&output)
        ));
    }

    Ok(())
}

pub fn close_workspace(workspace_ref: &str, window: &str) -> Result<()> {
    let output = Command::new("cmux")
        .arg("close-workspace")
        .arg("--workspace")
        .arg(workspace_ref)
        .arg("--window")
        .arg(window)
        .output()
        .with_context(|| format!("failed to execute cmux close-workspace for {workspace_ref}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux close-workspace failed for {workspace_ref}: {}",
            stderr_or_status(&output)
        ));
    }

    Ok(())
}

pub fn create_workspace_group(name: &str, workspace_refs: &[String], window: &str) -> Result<()> {
    if workspace_refs.len() < 2 {
        return Ok(());
    }

    let output = Command::new("cmux")
        .arg("workspace-group")
        .arg("create")
        .arg("--name")
        .arg(name)
        .arg("--from")
        .arg(workspace_refs.join(","))
        .arg("--window")
        .arg(window)
        .output()
        .with_context(|| format!("failed to execute cmux workspace-group create for {name}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "cmux workspace-group create failed for {name}: {}",
            stderr_or_status(&output)
        ));
    }

    Ok(())
}

fn parse_window_create_id(output: &[u8]) -> Result<String> {
    let value: Value =
        serde_json::from_slice(output).context("failed to parse window.create JSON")?;
    value
        .get("window_id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("window.create response did not contain window_id"))
}

fn parse_workspace_refs(output: &[u8]) -> Result<Vec<String>> {
    let value: Value =
        serde_json::from_slice(output).context("failed to parse cmux list-workspaces JSON")?;
    let workspaces = value
        .get("workspaces")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("cmux list-workspaces JSON did not contain workspaces[]"))?;

    Ok(workspaces
        .iter()
        .filter_map(|workspace| workspace.get("ref").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect())
}

fn stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        format!("exit status {}", output.status)
    } else {
        stderr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_window_id_from_window_create_json() {
        let window_id = parse_window_create_id(
            br#"{
                "window_id":"977EEEBE-E2CE-420B-89BC-BB8208DE43A3",
                "window_ref":"window:5"
            }"#,
        )
        .unwrap();

        assert_eq!(window_id, "977EEEBE-E2CE-420B-89BC-BB8208DE43A3");
    }

    #[test]
    fn parses_workspace_refs_from_list_workspaces_json() {
        let refs = parse_workspace_refs(
            br#"{
                "window_ref":"window:3",
                "workspaces":[{"ref":"workspace:7"},{"ref":"workspace:8"}]
            }"#,
        )
        .unwrap();

        assert_eq!(refs, vec!["workspace:7", "workspace:8"]);
    }
}
