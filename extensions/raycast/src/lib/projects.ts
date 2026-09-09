import { readdirSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";
import { parse } from "yaml";
import { CMUXINATOR_CONFIG_DIR, expandHome } from "./paths";

export interface ProjectWorkspace {
  name: string;
  /** Expanded absolute cwd, if declared. */
  cwd?: string;
  /** Configured surface names across all panes. */
  surfaceNames: string[];
}

export interface Project {
  name: string;
  file: string;
  workspaces: ProjectWorkspace[];
  /** Every directory the project touches: workspace cwds plus `cd <path>` targets in commands. */
  cwds: string[];
}

interface RawSurface {
  name?: string;
  command?: string;
}
interface RawPane {
  surfaces?: RawSurface[];
}
interface RawWorkspace {
  name?: string;
  cwd?: string;
  panes?: RawPane[];
}
interface RawProject {
  name?: string;
  workspaces?: RawWorkspace[];
}

const CD_RE = /(?:^|&&|;|\|\|)\s*cd\s+(?:"([^"]+)"|'([^']+)'|(\S+))/g;

/** Directories a shell command `cd`s into, expanded. Best-effort text scan. */
export function cwdsFromCommand(command: string | undefined): string[] {
  if (!command) return [];
  const out: string[] = [];
  for (const m of command.matchAll(CD_RE)) {
    const raw = m[1] ?? m[2] ?? m[3];
    if (raw) out.push(expandHome(raw));
  }
  return out;
}

export function parseProject(file: string, text: string): Project {
  const raw = (parse(text) ?? {}) as RawProject;
  const name = raw.name?.trim() || basename(file).replace(/\.ya?ml$/, "");
  const cwds = new Set<string>();
  const workspaces: ProjectWorkspace[] = (raw.workspaces ?? []).map((ws, i) => {
    const cwd = ws.cwd ? expandHome(ws.cwd) : undefined;
    if (cwd) cwds.add(cwd);
    const surfaceNames: string[] = [];
    for (const pane of ws.panes ?? []) {
      for (const surface of pane.surfaces ?? []) {
        if (surface.name) surfaceNames.push(surface.name);
        for (const dir of cwdsFromCommand(surface.command)) cwds.add(dir);
      }
    }
    return { name: ws.name?.trim() || `workspace-${i + 1}`, cwd, surfaceNames };
  });
  return { name, file, workspaces, cwds: [...cwds] };
}

export function loadProjects(dir: string = CMUXINATOR_CONFIG_DIR): Project[] {
  let entries: string[];
  try {
    entries = readdirSync(dir);
  } catch {
    return [];
  }
  const projects: Project[] = [];
  for (const entry of entries.sort()) {
    if (!/\.ya?ml$/.test(entry)) continue;
    const file = join(dir, entry);
    try {
      projects.push(parseProject(file, readFileSync(file, "utf8")));
    } catch {
      // Skip unparsable files; `cmuxinator validate` is the place to debug them.
    }
  }
  return projects;
}

/**
 * Resolve a user-typed name: exact match first, then a unique prefix match,
 * then a unique substring match. Case-insensitive.
 */
export function findProject(projects: Project[], query: string): Project | undefined {
  const q = query.trim().toLowerCase();
  if (!q) return undefined;
  const exact = projects.find((p) => p.name.toLowerCase() === q);
  if (exact) return exact;
  const prefix = projects.filter((p) => p.name.toLowerCase().startsWith(q));
  if (prefix.length === 1) return prefix[0];
  if (prefix.length > 1) return undefined;
  const contains = projects.filter((p) => p.name.toLowerCase().includes(q));
  return contains.length === 1 ? contains[0] : undefined;
}
