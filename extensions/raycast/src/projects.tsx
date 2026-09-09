import {
  Action,
  ActionPanel,
  Color,
  closeMainWindow,
  getPreferenceValues,
  Icon,
  Keyboard,
  List,
  showHUD,
  showToast,
  Toast,
} from "@raycast/api";
import { useEffect, useState } from "react";
import { CmuxClient, type CmuxWindow } from "./lib/cmux";
import { matchAll, type WindowMatch } from "./lib/matcher";
import { openProject } from "./lib/open";
import { CMUX_CANDIDATES, resolveBinary } from "./lib/paths";
import { loadProjects, type Project } from "./lib/projects";

interface Preferences {
  cmuxPath?: string;
  cmuxinatorPath?: string;
  socketPassword?: string;
  autoStart: boolean;
}

interface State {
  projects: Project[];
  matches: Map<string, WindowMatch>;
  cmuxRunning: boolean;
  loading: boolean;
  error?: string;
}

async function loadState(prefs: Preferences): Promise<State> {
  const projects = loadProjects();
  let windows: CmuxWindow[] = [];
  let cmuxRunning = false;
  let error: string | undefined;
  try {
    const cmux = new CmuxClient(
      resolveBinary("cmux", prefs.cmuxPath, CMUX_CANDIDATES),
      prefs.socketPassword,
    );
    cmuxRunning = await cmux.isRunning();
    if (cmuxRunning) windows = await cmux.windows();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  }
  return { projects, matches: matchAll(projects, windows), cmuxRunning, loading: false, error };
}

export default function Command() {
  const prefs = getPreferenceValues<Preferences>();
  const [state, setState] = useState<State>({
    projects: [],
    matches: new Map(),
    cmuxRunning: false,
    loading: true,
  });

  const refresh = () => {
    setState((s) => ({ ...s, loading: true }));
    loadState(prefs).then(setState);
  };

  useEffect(refresh, []);

  const open = async (project: Project, force: "auto" | "start") => {
    try {
      await closeMainWindow({ clearRootSearch: true });
      const outcome = await openProject(project, {
        ...prefs,
        autoStart: force === "start" ? true : prefs.autoStart,
      });
      if (outcome.kind === "focused") await showHUD(`cmux: ${project.name}`);
      else if (outcome.kind === "started") await showHUD(`cmux: started ${project.name}`);
      else await showHUD(`cmux: ${project.name} is not running`);
    } catch (error) {
      await showToast({
        style: Toast.Style.Failure,
        title: `Could not open ${project.name}`,
        message: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const running = state.projects.filter((p) => state.matches.has(p.name));
  const stopped = state.projects.filter((p) => !state.matches.has(p.name));

  const item = (project: Project) => {
    const match = state.matches.get(project.name);
    return (
      <List.Item
        key={project.name}
        title={project.name}
        subtitle={project.workspaces.map((w) => w.name).join(" · ")}
        icon={match ? { source: Icon.Terminal, tintColor: Color.Green } : Icon.Terminal}
        accessories={
          match
            ? [
                {
                  tag: { value: match.window.ref, color: Color.Green },
                  tooltip: match.reasons.join("\n"),
                },
              ]
            : [{ text: state.cmuxRunning ? "not running" : "cmux not running" }]
        }
        actions={
          <ActionPanel>
            <Action
              title={match ? "Focus Window" : "Start with Cmuxinator"}
              icon={match ? Icon.Window : Icon.Play}
              onAction={() => open(project, match ? "auto" : "start")}
            />
            {match && (
              <Action
                title="Start Another Window"
                icon={Icon.Plus}
                shortcut={Keyboard.Shortcut.Common.New}
                onAction={async () => {
                  // Bypass matching: always create a fresh window for this project.
                  const { startProject } = await import("./lib/cmuxinator");
                  const { resolveBinary: resolve, CMUXINATOR_CANDIDATES } =
                    await import("./lib/paths");
                  const { dirname } = await import("node:path");
                  try {
                    await closeMainWindow({ clearRootSearch: true });
                    const cmuxBinary = resolve("cmux", prefs.cmuxPath, CMUX_CANDIDATES);
                    await startProject(
                      resolve("cmuxinator", prefs.cmuxinatorPath, CMUXINATOR_CANDIDATES),
                      dirname(cmuxBinary),
                      project.name,
                      prefs.socketPassword,
                    );
                    await showHUD(`cmux: started ${project.name}`);
                  } catch (error) {
                    await showToast({
                      style: Toast.Style.Failure,
                      title: `Could not start ${project.name}`,
                      message: error instanceof Error ? error.message : String(error),
                    });
                  }
                }}
              />
            )}
            <Action.Open
              title="Open Project YAML"
              icon={Icon.Document}
              target={project.file}
              shortcut={Keyboard.Shortcut.Common.Open}
            />
            <Action
              title="Refresh"
              icon={Icon.ArrowClockwise}
              shortcut={Keyboard.Shortcut.Common.Refresh}
              onAction={refresh}
            />
          </ActionPanel>
        }
      />
    );
  };

  return (
    <List isLoading={state.loading} searchBarPlaceholder="Search cmuxinator projects…">
      {state.error && (
        <List.Section title="Error">
          <List.Item title={state.error} icon={{ source: Icon.Warning, tintColor: Color.Red }} />
        </List.Section>
      )}
      <List.Section title="Running" subtitle={`${running.length}`}>
        {running.map(item)}
      </List.Section>
      <List.Section title="Not running" subtitle={`${stopped.length}`}>
        {stopped.map(item)}
      </List.Section>
    </List>
  );
}
