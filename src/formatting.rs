use anyhow::Result;

use crate::cmux::NewWorkspaceSpec;

const DRY_RUN_WINDOW_HANDLE: &str = "<new-window>";

pub fn print_dry_run(
    project_name: &str,
    specs: &[NewWorkspaceSpec],
    create_group: bool,
) -> Result<()> {
    println!("cmuxinator dry-run: {} workspace(s)", specs.len());
    println!();
    println!("# create project window");
    println!("cmux rpc window.create '{{}}'");
    println!("cmux list-workspaces --window {DRY_RUN_WINDOW_HANDLE} --json");

    for (index, spec) in specs.iter().enumerate() {
        println!();
        println!("# workspace: {}", spec.name);
        println!("{}", command_shape(spec, DRY_RUN_WINDOW_HANDLE, index == 0));
        println!("{}", spec.pretty_layout_json()?);
    }

    println!();
    println!("# close placeholder workspace(s) created by cmux window.create");
    println!(
        "cmux close-workspace --workspace <placeholder-workspace> --window {DRY_RUN_WINDOW_HANDLE}"
    );

    if create_group && specs.len() > 1 {
        println!();
        println!("# group project workspaces under a collapsible sidebar header");
        println!(
            "cmux workspace-group create --name {} --from <created-workspace-refs> --window {}",
            project_name, DRY_RUN_WINDOW_HANDLE
        );
    }

    Ok(())
}

fn command_shape(spec: &NewWorkspaceSpec, window: &str, focus: bool) -> String {
    let mut parts = vec![
        "cmux".to_string(),
        "new-workspace".to_string(),
        "--name".to_string(),
        spec.name.clone(),
    ];

    if let Some(cwd) = &spec.cwd {
        parts.push("--cwd".to_string());
        parts.push(cwd.clone());
    }

    parts.push("--layout".to_string());
    parts.push("<json>".to_string());
    parts.push("--window".to_string());
    parts.push(window.to_string());
    parts.push("--focus".to_string());
    parts.push(focus.to_string());
    parts.join(" ")
}

pub fn print_project_list(projects: &[String]) {
    for project in projects {
        println!("{project}");
    }
}

pub fn print_validation_success(project_name: &str, path: &std::path::Path) {
    println!("Project {project_name} is valid ({})", path.display());
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn dry_run_command_shape_includes_window_and_cwd_when_present() {
        let spec = NewWorkspaceSpec {
            name: "dev".to_string(),
            cwd: Some("/tmp/app".to_string()),
            layout: json!({ "pane": { "surfaces": [{ "type": "terminal" }] } }),
        };

        assert_eq!(
            command_shape(&spec, "window:3", true),
            "cmux new-workspace --name dev --cwd /tmp/app --layout <json> --window window:3 --focus true"
        );
    }
}
