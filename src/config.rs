use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::paths;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub name: String,
    pub cwd: Option<String>,
    pub layout: Option<LayoutPreset>,
    pub panes: Vec<Pane>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pane {
    pub surfaces: Option<Vec<Surface>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    #[serde(rename = "type", default)]
    pub surface_type: SurfaceType,
    pub name: Option<String>,
    pub command: Option<String>,
    pub url: Option<String>,
    pub focus: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceType {
    #[default]
    Terminal,
    Browser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutPreset {
    EvenHorizontal,
    EvenVertical,
    MainHorizontal,
    MainVertical,
    Tiled,
}

#[derive(Debug)]
pub struct LoadedProject {
    pub project: Project,
    pub path: PathBuf,
}

impl Project {
    pub fn validate(&self) -> Result<()> {
        paths::validate_name(&self.name, "project")?;

        if self.workspaces.is_empty() {
            return Err(anyhow!(
                "project {} must define at least one workspace",
                self.name
            ));
        }

        let mut workspace_names = HashSet::new();
        for workspace in &self.workspaces {
            workspace.validate()?;
            if !workspace_names.insert(workspace.name.as_str()) {
                return Err(anyhow!(
                    "project {} defines duplicate workspace name {}",
                    self.name,
                    workspace.name
                ));
            }
        }

        Ok(())
    }
}

impl Workspace {
    pub fn layout_preset(&self) -> LayoutPreset {
        self.layout.unwrap_or(LayoutPreset::EvenHorizontal)
    }

    fn validate(&self) -> Result<()> {
        paths::validate_name(&self.name, "workspace")?;

        if let Some(cwd) = &self.cwd {
            paths::expand_cwd(cwd)
                .with_context(|| format!("invalid cwd for workspace {}", self.name))?;
        }

        if self.panes.is_empty() {
            return Err(anyhow!(
                "workspace {} must define at least one pane",
                self.name
            ));
        }

        let mut focused_surfaces = 0;
        for (pane_index, pane) in self.panes.iter().enumerate() {
            pane.validate(&self.name, pane_index)?;
            if let Some(surfaces) = &pane.surfaces {
                focused_surfaces += surfaces
                    .iter()
                    .filter(|surface| surface.focus.unwrap_or(false))
                    .count();
            }
        }

        if focused_surfaces > 1 {
            return Err(anyhow!(
                "workspace {} has multiple surfaces with focus: true",
                self.name
            ));
        }

        Ok(())
    }
}

impl Pane {
    fn validate(&self, workspace_name: &str, pane_index: usize) -> Result<()> {
        let Some(surfaces) = &self.surfaces else {
            return Ok(());
        };

        if surfaces.is_empty() {
            return Err(anyhow!(
                "workspace {} pane {} must not use surfaces: []",
                workspace_name,
                pane_index + 1
            ));
        }

        for (surface_index, surface) in surfaces.iter().enumerate() {
            surface.validate(workspace_name, pane_index, surface_index)?;
        }

        Ok(())
    }
}

impl Surface {
    fn validate(
        &self,
        workspace_name: &str,
        pane_index: usize,
        surface_index: usize,
    ) -> Result<()> {
        let location = format!(
            "workspace {} pane {} surface {}",
            workspace_name,
            pane_index + 1,
            surface_index + 1
        );

        if let Some(name) = &self.name {
            paths::validate_name(name, "surface")
                .with_context(|| format!("invalid name for {location}"))?;
        }

        match self.surface_type {
            SurfaceType::Terminal => {
                if self.url.is_some() {
                    return Err(anyhow!("terminal {location} must not define url"));
                }
            }
            SurfaceType::Browser => {
                if self.command.is_some() {
                    return Err(anyhow!("browser {location} must not define command"));
                }
                let Some(url) = &self.url else {
                    return Err(anyhow!("browser {location} must define url"));
                };
                if url.trim().is_empty() {
                    return Err(anyhow!("browser {location} must define a non-empty url"));
                }
            }
        }

        Ok(())
    }
}

pub fn load_project(project: &str) -> Result<LoadedProject> {
    let path = paths::project_path(project)?;
    let source = fs::read_to_string(&path)
        .with_context(|| format!("failed to read project file {}", path.display()))?;
    let project = serde_yaml::from_str::<Project>(&source)
        .with_context(|| format!("failed to parse project file {}", path.display()))?;

    Ok(LoadedProject { project, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_project(yaml: &str) -> Project {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn terminal_is_default_surface_type() {
        let project = parse_project(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - surfaces:
          - name: shell
            command: pi
"#,
        );

        let surface = project.workspaces[0].panes[0].surfaces.as_ref().unwrap()[0].surface_type;
        assert_eq!(surface, SurfaceType::Terminal);
        project.validate().unwrap();
    }

    #[test]
    fn accepts_browser_surface_with_url() {
        let project = parse_project(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - surfaces:
          - type: browser
            name: preview
            url: http://localhost:3000
"#,
        );

        project.validate().unwrap();
    }

    #[test]
    fn rejects_browser_surface_without_url() {
        let project = parse_project(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - surfaces:
          - type: browser
            name: preview
"#,
        );

        assert!(project.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_workspace_names() {
        let project = parse_project(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - {}
  - name: dev
    panes:
      - {}
"#,
        );

        assert!(project.validate().is_err());
    }

    #[test]
    fn rejects_multiple_focused_surfaces() {
        let project = parse_project(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - surfaces:
          - name: one
            focus: true
          - name: two
            focus: true
"#,
        );

        assert!(project.validate().is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        let result = serde_yaml::from_str::<Project>(
            r#"
name: dex
unknown: true
workspaces:
  - name: dev
    panes:
      - {}
"#,
        );

        assert!(result.is_err());
    }
}
