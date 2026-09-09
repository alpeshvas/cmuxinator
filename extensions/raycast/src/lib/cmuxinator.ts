import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { cmuxEnv } from "./cmux";

const execFileAsync = promisify(execFile);

export interface StartResult {
  windowId?: string;
  stdout: string;
}

/** Run `cmuxinator start <project>` and pick the created window id out of its output. */
export async function startProject(
  cmuxinatorBinary: string,
  cmuxBinaryDir: string,
  project: string,
  socketPassword?: string,
): Promise<StartResult> {
  const env = cmuxEnv(socketPassword);
  // cmuxinator shells out to `cmux` by name; Raycast's PATH lacks Homebrew.
  env.PATH = [cmuxBinaryDir, env.PATH ?? "/usr/bin:/bin"].join(":");

  const { stdout } = await execFileAsync(cmuxinatorBinary, ["start", project], {
    env,
    timeout: 60_000,
    maxBuffer: 4 * 1024 * 1024,
  });
  const windowId = /Created window ([0-9A-Fa-f-]{36})/.exec(stdout)?.[1];
  return { windowId, stdout };
}
