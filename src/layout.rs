use serde_json::{Map, Value, json};

use crate::config::{LayoutPreset, Pane, Surface, SurfaceType, Workspace};

#[derive(Clone, Copy)]
enum SplitDirection {
    Horizontal,
    Vertical,
}

impl SplitDirection {
    fn as_str(self) -> &'static str {
        match self {
            SplitDirection::Horizontal => "horizontal",
            SplitDirection::Vertical => "vertical",
        }
    }

    fn opposite(self) -> Self {
        match self {
            SplitDirection::Horizontal => SplitDirection::Vertical,
            SplitDirection::Vertical => SplitDirection::Horizontal,
        }
    }
}

pub fn build_workspace_layout(workspace: &Workspace) -> Value {
    let pane_nodes = workspace.panes.iter().map(pane_node).collect::<Vec<_>>();

    match workspace.layout_preset() {
        LayoutPreset::EvenHorizontal => split_balanced(&pane_nodes, SplitDirection::Horizontal),
        LayoutPreset::EvenVertical => split_balanced(&pane_nodes, SplitDirection::Vertical),
        LayoutPreset::MainHorizontal => split_main(&pane_nodes, SplitDirection::Horizontal),
        LayoutPreset::MainVertical => split_main(&pane_nodes, SplitDirection::Vertical),
        LayoutPreset::Tiled => split_tiled(&pane_nodes, SplitDirection::Horizontal),
    }
}

fn pane_node(pane: &Pane) -> Value {
    let surfaces = match &pane.surfaces {
        Some(surfaces) => surfaces.iter().map(surface_value).collect(),
        None => vec![json!({ "type": "terminal" })],
    };

    json!({
        "pane": {
            "surfaces": surfaces,
        },
    })
}

fn surface_value(surface: &Surface) -> Value {
    let mut value = Map::new();

    match surface.surface_type {
        SurfaceType::Terminal => {
            value.insert("type".to_string(), json!("terminal"));
            if let Some(command) = &surface.command {
                value.insert("command".to_string(), json!(command));
            }
        }
        SurfaceType::Browser => {
            value.insert("type".to_string(), json!("browser"));
            if let Some(url) = &surface.url {
                value.insert("url".to_string(), json!(url));
            }
        }
    }

    if let Some(name) = &surface.name {
        value.insert("name".to_string(), json!(name));
    }
    if let Some(focus) = surface.focus {
        value.insert("focus".to_string(), json!(focus));
    }

    Value::Object(value)
}

fn split_balanced(nodes: &[Value], direction: SplitDirection) -> Value {
    match nodes.len() {
        0 => json!({ "pane": { "surfaces": [{ "type": "terminal" }] } }),
        1 => nodes[0].clone(),
        len => {
            let left_count = len / 2;
            let split = left_count as f64 / len as f64;
            json!({
                "direction": direction.as_str(),
                "split": split,
                "children": [
                    split_balanced(&nodes[..left_count], direction),
                    split_balanced(&nodes[left_count..], direction),
                ],
            })
        }
    }
}

fn split_main(nodes: &[Value], direction: SplitDirection) -> Value {
    match nodes.len() {
        0 => json!({ "pane": { "surfaces": [{ "type": "terminal" }] } }),
        1 => nodes[0].clone(),
        _ => json!({
            "direction": direction.as_str(),
            "split": 0.6,
            "children": [
                nodes[0].clone(),
                split_balanced(&nodes[1..], direction),
            ],
        }),
    }
}

fn split_tiled(nodes: &[Value], direction: SplitDirection) -> Value {
    match nodes.len() {
        0 => json!({ "pane": { "surfaces": [{ "type": "terminal" }] } }),
        1 => nodes[0].clone(),
        len => {
            let left_count = len / 2;
            let split = left_count as f64 / len as f64;
            json!({
                "direction": direction.as_str(),
                "split": split,
                "children": [
                    split_tiled(&nodes[..left_count], direction.opposite()),
                    split_tiled(&nodes[left_count..], direction.opposite()),
                ],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Project;

    use super::*;

    fn workspace_from_yaml(yaml: &str) -> Workspace {
        let project: Project = serde_yaml::from_str(yaml).unwrap();
        project.validate().unwrap();
        project.workspaces.into_iter().next().unwrap()
    }

    #[test]
    fn builds_single_pane_layout() {
        let workspace = workspace_from_yaml(
            r#"
name: dex
workspaces:
  - name: dev
    panes:
      - surfaces:
          - name: shell
"#,
        );

        let layout = build_workspace_layout(&workspace);
        assert_eq!(
            layout,
            json!({ "pane": { "surfaces": [{ "type": "terminal", "name": "shell" }] } })
        );
    }

    #[test]
    fn builds_two_pane_even_horizontal_layout() {
        let workspace = workspace_from_yaml(
            r#"
name: dex
workspaces:
  - name: dev
    layout: even-horizontal
    panes:
      - surfaces:
          - name: left
      - surfaces:
          - name: right
"#,
        );

        let layout = build_workspace_layout(&workspace);
        assert_eq!(layout["direction"], "horizontal");
        assert_eq!(layout["split"], 0.5);
        assert_eq!(layout["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn builds_three_pane_even_layout_with_equal_top_split() {
        let workspace = workspace_from_yaml(
            r#"
name: dex
workspaces:
  - name: dev
    layout: even-horizontal
    panes:
      - surfaces:
          - name: one
      - surfaces:
          - name: two
      - surfaces:
          - name: three
"#,
        );

        let layout = build_workspace_layout(&workspace);
        assert_eq!(layout["direction"], "horizontal");
        assert_eq!(layout["split"], json!(1.0 / 3.0));
        assert_eq!(layout["children"][1]["split"], 0.5);
    }

    #[test]
    fn builds_four_pane_even_vertical_layout() {
        let workspace = workspace_from_yaml(
            r#"
name: dex
workspaces:
  - name: dev
    layout: even-vertical
    panes:
      - surfaces:
          - name: one
      - surfaces:
          - name: two
      - surfaces:
          - name: three
      - surfaces:
          - name: four
"#,
        );

        let layout = build_workspace_layout(&workspace);
        assert_eq!(layout["direction"], "vertical");
        assert_eq!(layout["split"], 0.5);
        assert_eq!(layout["children"][0]["split"], 0.5);
        assert_eq!(layout["children"][1]["split"], 0.5);
    }

    #[test]
    fn emits_browser_surface() {
        let workspace = workspace_from_yaml(
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

        let layout = build_workspace_layout(&workspace);
        assert_eq!(layout["pane"]["surfaces"][0]["type"], "browser");
        assert_eq!(
            layout["pane"]["surfaces"][0]["url"],
            "http://localhost:3000"
        );
    }
}
