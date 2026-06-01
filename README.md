# cmuxinator

A Rust CLI for starting cmux projects from tmuxinator-style YAML files.

A cmuxinator project is one config file under `~/.config/cmuxinator/` that opens a new cmux window and creates the project's cmux workspaces inside it. cmux creates a placeholder workspace when a new window is opened; cmuxinator closes that placeholder after creating the project workspaces.

## Install

```sh
cargo install --path ~/codebase/cmuxinator
```

Optional short alias:

```sh
alias cmx='cmuxinator'
```

## Commands

```sh
cmuxinator new dex              # scaffold ~/.config/cmuxinator/dex.yml
cmuxinator list                 # list projects
cmuxinator validate dex         # parse and validate config, check cmux is available
cmuxinator dry-run dex          # print generated cmux calls and layout JSON
cmuxinator start dex            # create a new cmux window and open all project workspaces
cmuxinator start dex agents     # create a new cmux window and open selected workspace(s)
```

## Config example

```yaml
name: dex
workspaces:
  - name: agents
    cwd: ~/codebase/dex-ios
    layout: even-horizontal
    panes:
      - surfaces:
          - name: agent
            command: pi
            focus: true
          - name: shell
      - surfaces:
          - name: magellan-agent
            command: cd ~/codebase/magellan && pi

  - name: dev
    cwd: ~/codebase/dex-ios
    layout: even-vertical
    panes:
      - surfaces:
          - name: app
            command: npm run dev
      - surfaces:
          - type: browser
            name: preview
            url: http://localhost:3000
```

Terminal is the default surface type. Browser surfaces require `type: browser` and `url`.
