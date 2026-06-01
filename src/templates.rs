pub fn project_template(project: &str) -> String {
    format!(
        r#"# cmuxinator project config
# Canonical command: cmuxinator start {project}
# Optional shell alias: alias cmx='cmuxinator'
name: {project}
workspaces:
  - name: agents
    cwd: ~/codebase/{project}
    layout: even-horizontal
    panes:
      - surfaces:
          - name: agent
            command: pi
            focus: true
          - name: shell
      - surfaces:
          - name: second-agent
            command: pi

  - name: dev
    cwd: ~/codebase/{project}
    layout: even-vertical
    panes:
      - surfaces:
          - name: app
            command: npm run dev
      - surfaces:
          - type: browser
            name: preview
            url: http://localhost:3000
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_project_name_and_browser_example() {
        let template = project_template("dex");
        assert!(template.contains("name: dex"));
        assert!(template.contains("type: browser"));
        assert!(template.contains("alias cmx='cmuxinator'"));
    }
}
