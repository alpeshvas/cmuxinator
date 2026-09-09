import {
  closeMainWindow,
  getPreferenceValues,
  LaunchProps,
  showHUD,
  showToast,
  Toast,
} from "@raycast/api";
import { openProject } from "./lib/open";
import { log } from "./lib/log";
import { findProject, loadProjects } from "./lib/projects";

interface Preferences {
  cmuxPath?: string;
  cmuxinatorPath?: string;
  socketPassword?: string;
  autoStart: boolean;
}

export default async function Command(props: LaunchProps<{ arguments: { project: string } }>) {
  const query = props.arguments.project?.trim() ?? "";
  log("open-project", { query });
  const projects = loadProjects();
  const project = findProject(projects, query);

  if (!project) {
    const names = projects.map((p) => p.name).join(", ") || "none found in ~/.config/cmuxinator";
    await showToast({
      style: Toast.Style.Failure,
      title: `No cmuxinator project matches "${query}"`,
      message: `Known projects: ${names}`,
    });
    return;
  }

  const prefs = getPreferenceValues<Preferences>();
  try {
    await closeMainWindow({ clearRootSearch: true });
    const outcome = await openProject(project, prefs);
    log("open-project outcome", outcome);
    if (outcome.kind === "focused") {
      await showHUD(`cmux: ${project.name}`);
    } else if (outcome.kind === "started") {
      await showHUD(`cmux: started ${project.name}`);
    } else {
      await showHUD(`cmux: ${project.name} is not running`);
    }
  } catch (error) {
    log(`open-project failed for ${project.name}`, error);
    await showToast({
      style: Toast.Style.Failure,
      title: `Could not open ${project.name}`,
      message: error instanceof Error ? error.message : String(error),
    });
  }
}
