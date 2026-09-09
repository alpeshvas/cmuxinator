import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readCmuxSocketPath } from "./paths";

const execFileAsync = promisify(execFile);

export interface CmuxSurface {
  id: string;
  title: string;
  type: string;
}
export interface CmuxWorkspace {
  id: string;
  title: string;
  currentDirectory?: string;
  surfaces: CmuxSurface[];
}
export interface CmuxWindow {
  id: string;
  ref: string;
  index: number;
  key: boolean;
  workspaces: CmuxWorkspace[];
}

/**
 * Environment for running the cmux CLI outside a cmux terminal: the socket path cmux
 * last announced, and an optional socket password for "password" control mode.
 */
export function cmuxEnv(socketPassword?: string): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env, CMUX_QUIET: "1" };
  const socket = readCmuxSocketPath();
  if (socket && !env.CMUX_SOCKET_PATH) env.CMUX_SOCKET_PATH = socket;
  const password = socketPassword?.trim();
  if (password) env.CMUX_SOCKET_PASSWORD = password;
  return env;
}

export class CmuxClient {
  constructor(
    private readonly binary: string,
    private readonly socketPassword?: string,
  ) {}

  private env(): NodeJS.ProcessEnv {
    return cmuxEnv(this.socketPassword);
  }

  async run(args: string[]): Promise<string> {
    const { stdout } = await execFileAsync(this.binary, args, {
      env: this.env(),
      timeout: 8000,
      maxBuffer: 16 * 1024 * 1024,
    });
    return stdout;
  }

  async isRunning(): Promise<boolean> {
    try {
      await this.run(["ping"]);
      return true;
    } catch {
      return false;
    }
  }

  /** Poll `cmux ping` until the socket answers or the timeout elapses. */
  async waitUntilRunning(timeoutMs: number, intervalMs = 400): Promise<boolean> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (await this.isRunning()) return true;
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
    return false;
  }

  /** All windows with their workspaces, surfaces and live cwd. */
  async windows(): Promise<CmuxWindow[]> {
    const tree = JSON.parse(await this.run(["tree", "--all", "--json", "--id-format", "both"])) as {
      windows?: TreeWindow[];
    };
    const windows: CmuxWindow[] = [];
    for (const w of tree.windows ?? []) {
      const cwdById = await this.workspaceDirectories(w.id);
      windows.push({
        id: w.id,
        ref: w.ref,
        index: w.index,
        key: !!w.key,
        workspaces: (w.workspaces ?? []).map((ws) => ({
          id: ws.id,
          title: ws.title ?? "",
          currentDirectory: cwdById.get(ws.id),
          surfaces: (ws.panes ?? []).flatMap((p) =>
            (p.surfaces ?? []).map((s) => ({ id: s.id, title: s.title ?? "", type: s.type ?? "" })),
          ),
        })),
      });
    }
    return windows;
  }

  private async workspaceDirectories(windowId: string): Promise<Map<string, string>> {
    const map = new Map<string, string>();
    try {
      const out = JSON.parse(
        await this.run([
          "workspace",
          "list",
          "--window",
          windowId,
          "--json",
          "--id-format",
          "both",
        ]),
      ) as { workspaces?: { id?: string; current_directory?: string | null }[] };
      for (const ws of out.workspaces ?? []) {
        if (ws.id && ws.current_directory) map.set(ws.id, ws.current_directory);
      }
    } catch {
      // cwd is only a tie-breaker; a failure here must not block focusing.
    }
    return map;
  }

  async focusWindow(windowId: string): Promise<void> {
    await this.run(["focus-window", "--window", windowId]);
  }
}

interface TreeSurface {
  id: string;
  title?: string;
  type?: string;
}
interface TreePane {
  surfaces?: TreeSurface[];
}
interface TreeWorkspace {
  id: string;
  title?: string;
  panes?: TreePane[];
}
interface TreeWindow {
  id: string;
  ref: string;
  index: number;
  key?: boolean;
  workspaces?: TreeWorkspace[];
}

/** Launch the cmux app if it is not running; returns once `open` has handed off to launchd. */
export async function launchCmuxApp(): Promise<void> {
  await execFileAsync("/usr/bin/open", ["-b", "com.cmuxterm.app"], { timeout: 15_000 });
}

/** Bring the cmux app to the foreground (focus-window alone only reorders cmux's own windows). */
export async function activateCmuxApp(): Promise<void> {
  try {
    await execFileAsync(
      "/usr/bin/osascript",
      ["-e", 'tell application id "com.cmuxterm.app" to activate'],
      {
        timeout: 5000,
      },
    );
  } catch {
    // Non-fatal: the window was still focused inside cmux.
  }
}
