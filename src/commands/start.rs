use std::collections::HashSet;

use anyhow::{Context, Result, anyhow};

use crate::{
    cmux::{self, NewWorkspaceSpec},
    config::{self, Project},
    formatting, layout, paths,
};

struct StartPlan {
    project_name: String,
    specs: Vec<NewWorkspaceSpec>,
}

pub fn run(project: &str, selected_workspaces: &[String], create_group: bool) -> Result<()> {
    let plan = build_plan(project, selected_workspaces)?;
    cmux::ensure_available()?;

    let window = cmux::new_window()?;
    let placeholder_workspaces = cmux::workspace_refs(&window)?;
    println!("Created window {window}");

    let mut created_workspace_refs = Vec::new();
    for (index, spec) in plan.specs.iter().enumerate() {
        let before = cmux::workspace_refs(&window)?
            .into_iter()
            .collect::<HashSet<_>>();
        cmux::new_workspace(spec, &window, index == 0)
            .with_context(|| format!("failed to start workspace {}", spec.name))?;
        println!("Started workspace {}", spec.name);

        let new_ref = new_workspace_ref(&window, &before)
            .with_context(|| format!("failed to locate created workspace {}", spec.name))?;
        created_workspace_refs.push(new_ref);
    }

    for workspace_ref in placeholder_workspaces {
        cmux::close_workspace(&workspace_ref, &window)
            .with_context(|| format!("failed to close placeholder workspace {workspace_ref}"))?;
    }

    if create_group && created_workspace_refs.len() > 1 {
        cmux::create_workspace_group(&plan.project_name, &created_workspace_refs, &window)
            .with_context(|| format!("failed to create workspace group {}", plan.project_name))?;
        println!("Grouped workspaces under {}", plan.project_name);
    }

    Ok(())
}

pub fn dry_run(project: &str, selected_workspaces: &[String], create_group: bool) -> Result<()> {
    let plan = build_plan(project, selected_workspaces)?;
    formatting::print_dry_run(&plan.project_name, &plan.specs, create_group)
}

fn build_plan(project_name: &str, selected_workspaces: &[String]) -> Result<StartPlan> {
    let loaded = config::load_project(project_name)?;
    loaded.project.validate()?;
    let project_name = loaded.project.name.clone();
    let specs = select_specs(&loaded.project, selected_workspaces)?;
    Ok(StartPlan {
        project_name,
        specs,
    })
}

fn new_workspace_ref(window: &str, before: &HashSet<String>) -> Result<String> {
    let after = cmux::workspace_refs(window)?;
    let created = after
        .into_iter()
        .filter(|workspace_ref| !before.contains(workspace_ref))
        .collect::<Vec<_>>();

    match created.as_slice() {
        [workspace_ref] => Ok(workspace_ref.clone()),
        [] => Err(anyhow!("no new workspace appeared in window {window}")),
        _ => Err(anyhow!(
            "multiple new workspaces appeared in window {window}: {}",
            created.join(", ")
        )),
    }
}

fn select_specs(
    project: &Project,
    selected_workspaces: &[String],
) -> Result<Vec<NewWorkspaceSpec>> {
    let all_names = project
        .workspaces
        .iter()
        .map(|workspace| workspace.name.as_str())
        .collect::<HashSet<_>>();
    let selected_names = selected_workspaces
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    let unknown = selected_names
        .iter()
        .filter(|name| !all_names.contains(**name))
        .copied()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "project {} does not define workspace(s): {}",
            project.name,
            unknown.join(", ")
        ));
    }

    project
        .workspaces
        .iter()
        .filter(|workspace| {
            selected_names.is_empty() || selected_names.contains(workspace.name.as_str())
        })
        .map(|workspace| {
            let cwd = workspace
                .cwd
                .as_deref()
                .map(paths::expand_cwd)
                .transpose()
                .with_context(|| format!("invalid cwd for workspace {}", workspace.name))?;
            let layout = layout::build_workspace_layout(workspace);

            Ok(NewWorkspaceSpec {
                name: workspace.name.clone(),
                cwd,
                layout,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::config::Project;

    use super::*;

    #[test]
    fn selects_only_requested_workspaces() {
        let project: Project = serde_yaml::from_str(
            r#"
name: dex
workspaces:
  - name: agents
    panes:
      - {}
  - name: dev
    panes:
      - {}
"#,
        )
        .unwrap();
        project.validate().unwrap();

        let specs = select_specs(&project, &["dev".to_string()]).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "dev");
    }

    #[test]
    fn rejects_unknown_requested_workspaces() {
        let project: Project = serde_yaml::from_str(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - {}
"#,
        )
        .unwrap();
        project.validate().unwrap();

        assert!(select_specs(&project, &["agents".to_string()]).is_err());
    }
}
