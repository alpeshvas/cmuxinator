# cmuxinator for Raycast

Two commands for jumping to [cmuxinator](https://github.com/alpeshvas/cmuxinator) projects in cmux:

- **Open Cmux Project** `<project>` – focuses the cmux window that is running the project, or starts it with `cmuxinator start <project>` if none is. Give this command the alias `cmux` in Raycast so `cmux ho-agents` does what you expect. Unique prefixes work too (`cmux dex`, `cmux ho-a`).
- **Cmux Projects** `[project]` – lists every `~/.config/cmuxinator/*.yml` project with its live window, plus actions to focus, start, open another window, or open the YAML. With an argument that uniquely names a project it skips the list and opens it directly, so this command can carry the `cmux` alias too.

Raycast text arguments have no autocomplete, so suggestions come from the list: when
**Open Cmux Project** gets a name that matches nothing or several projects (`ho` → `ho-agents`,
`hooter`), it opens **Cmux Projects** pre-filtered with what you typed instead of failing.

## How windows are matched

cmuxinator does not tag the windows it creates and every project here uses the same
workspace titles (`agents`, `dev`), so the extension scores each live cmux window against
the project YAML:

| signal                                                                               | weight |
| ------------------------------------------------------------------------------------ | ------ |
| a configured surface name (`din-djarin`, `agent-1`, …) appears as a surface title    | 3 each |
| a configured workspace name appears as a workspace title                             | 2 each |
| a workspace's live cwd is inside a project directory (`cwd:` or `cd …` in a command) | 1 each |

A window counts as running the project when at least one surface name matched, or when
every workspace title matched and one cwd is inside the project. Highest score wins.

## One-time setup: let Raycast talk to cmux

cmux's control socket defaults to `cmuxOnly`, which rejects any process that was not
started inside a cmux terminal, including Raycast. Switch it to password mode:

```jsonc
// ~/.config/cmux/cmux.json
"automation": {
  "socketControlMode": "password",
  "socketPassword": "<random string>"
},
```

then run `cmux reload-config` inside cmux. The `cmux` CLI reads the saved password itself,
so nothing else is needed. If you prefer to keep the password out of `cmux.json`, set it in
cmux **Settings → Automation** instead and paste the same value into the extension's
"cmux socket password" preference; it is passed to `cmux` and `cmuxinator` as
`CMUX_SOCKET_PASSWORD`.

If cmux itself is not running, the extension launches it, waits for the socket to answer,
and then looks for the project among the restored windows before falling back to
`cmuxinator start`. Untick "Start the project…" in preferences to disable both behaviours.

Binaries are auto-detected at `/opt/homebrew/bin/cmux` and `~/.cargo/bin/cmuxinator`;
override them in the extension preferences if yours live elsewhere.

## Development

```sh
npm install
npm run dev      # imports the extension into Raycast and rebuilds on change
npm test         # matcher / YAML parsing tests
npm run lint
```

Runtime diagnostics are appended to `~/Library/Logs/cmuxinator-raycast.log`.
