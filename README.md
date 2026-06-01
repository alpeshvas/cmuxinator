# cmuxinator

A Rust CLI for starting cmux projects from tmuxinator-style YAML files.

A cmuxinator project is one config file under `~/.config/cmuxinator/` that opens a new cmux window, creates the project's cmux workspaces inside it, and groups those workspaces under a collapsible project header when there is more than one workspace. cmux creates a placeholder workspace when a new window is opened; cmuxinator closes that placeholder after creating the project workspaces.

## Install

With Homebrew:

```sh
brew tap alpeshvas/cmuxinator
brew install cmuxinator
```

cmuxinator shells out to `cmux`; install cmux separately if needed:

```sh
brew install --cask cmux
```

From source:

```sh
cargo install --git https://github.com/alpeshvas/cmuxinator
```

For local development:

```sh
cargo install --path ~/codebase/cmuxinator
```

Optional short alias:

```sh
alias cmx='cmuxinator'
```

## Shell completion

cmuxinator can generate dynamic shell completions, including subcommands, flags, project names from `~/.config/cmuxinator/*.yml`, and the selected project's workspace names for `start` / `dry-run`.

Add the line for your shell to your shell startup file:

```sh
# zsh
source <(COMPLETE=zsh cmuxinator)

# bash
source <(COMPLETE=bash cmuxinator)

# fish
COMPLETE=fish cmuxinator | source
```

For the optional zsh alias, add this after sourcing completions:

```sh
alias cmx='cmuxinator'
compdef _clap_dynamic_completer_cmuxinator cmx
```

Re-source completions after upgrading cmuxinator.

## Commands

```sh
cmuxinator new dex              # scaffold ~/.config/cmuxinator/dex.yml
cmuxinator list                 # list projects
cmuxinator validate dex         # parse and validate config, check cmux is available
cmuxinator dry-run dex          # print generated cmux calls, group command, and layout JSON
cmuxinator start dex            # create a new cmux window, open workspaces, group them
cmuxinator start dex --no-group # open workspaces without creating a workspace group
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
