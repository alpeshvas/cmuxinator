import type { CmuxWindow } from "./cmux";
import type { Project } from "./projects";

export interface WindowMatch {
  window: CmuxWindow;
  score: number;
  /** Human-readable reasons, for debugging and the list UI. */
  reasons: string[];
}

const SCORE_SURFACE_NAME = 3;
const SCORE_WORKSPACE_TITLE = 2;
const SCORE_CWD = 1;

function normalize(s: string): string {
  return s.trim().toLowerCase();
}

function isUnder(path: string, root: string): boolean {
  const p = path.replace(/\/+$/, "");
  const r = root.replace(/\/+$/, "");
  return p === r || p.startsWith(r + "/");
}

/**
 * Score how well a live cmux window matches a cmuxinator project.
 *
 * cmuxinator does not tag the windows it creates, and every project here has
 * workspaces called "agents"/"dev", so titles alone can't tell projects apart.
 * Surface names (din-djarin, agent-1, ...) are distinctive, so they weigh most;
 * workspace titles and live cwds under a configured project directory break ties.
 */
export function scoreWindow(project: Project, window: CmuxWindow): WindowMatch {
  const reasons: string[] = [];
  let score = 0;

  const liveWorkspaceTitles = new Set(window.workspaces.map((ws) => normalize(ws.title)));
  const liveSurfaceTitles = new Set(
    window.workspaces.flatMap((ws) => ws.surfaces.map((s) => normalize(s.title))),
  );

  for (const ws of project.workspaces) {
    if (liveWorkspaceTitles.has(normalize(ws.name))) {
      score += SCORE_WORKSPACE_TITLE;
      reasons.push(`workspace "${ws.name}"`);
    }
    for (const surfaceName of ws.surfaceNames) {
      if (liveSurfaceTitles.has(normalize(surfaceName))) {
        score += SCORE_SURFACE_NAME;
        reasons.push(`surface "${surfaceName}"`);
      }
    }
  }

  for (const ws of window.workspaces) {
    if (ws.currentDirectory && project.cwds.some((root) => isUnder(ws.currentDirectory!, root))) {
      score += SCORE_CWD;
      reasons.push(`cwd ${ws.currentDirectory}`);
    }
  }

  return { window, score, reasons };
}

/**
 * A window counts as "running this project" if a distinctive surface name matched,
 * or every configured workspace title matched and at least one live cwd is inside the project.
 */
export function isConfidentMatch(project: Project, match: WindowMatch): boolean {
  const surfaceHits = match.reasons.filter((r) => r.startsWith("surface ")).length;
  if (surfaceHits > 0) return true;
  const workspaceHits = match.reasons.filter((r) => r.startsWith("workspace ")).length;
  const cwdHits = match.reasons.filter((r) => r.startsWith("cwd ")).length;
  return (
    project.workspaces.length > 0 && workspaceHits === project.workspaces.length && cwdHits > 0
  );
}

/** Best confident window for a project, or undefined if it doesn't look open. */
export function findProjectWindow(
  project: Project,
  windows: CmuxWindow[],
): WindowMatch | undefined {
  const ranked = windows
    .map((w) => scoreWindow(project, w))
    .filter((m) => isConfidentMatch(project, m))
    .sort((a, b) => b.score - a.score || a.window.index - b.window.index);
  return ranked[0];
}

/** Map every project to its window (if any), preferring the strongest claim when two projects want one window. */
export function matchAll(projects: Project[], windows: CmuxWindow[]): Map<string, WindowMatch> {
  const candidates: { project: Project; match: WindowMatch }[] = [];
  for (const project of projects) {
    for (const window of windows) {
      const match = scoreWindow(project, window);
      if (isConfidentMatch(project, match)) candidates.push({ project, match });
    }
  }
  candidates.sort((a, b) => b.match.score - a.match.score);
  const byProject = new Map<string, WindowMatch>();
  const takenWindows = new Set<string>();
  for (const { project, match } of candidates) {
    if (byProject.has(project.name) || takenWindows.has(match.window.id)) continue;
    byProject.set(project.name, match);
    takenWindows.add(match.window.id);
  }
  return byProject;
}
