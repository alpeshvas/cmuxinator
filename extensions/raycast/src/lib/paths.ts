import { accessSync, constants, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

export function expandHome(path: string): string {
  if (path === "~") return homedir();
  if (path.startsWith("~/")) return join(homedir(), path.slice(2));
  return path;
}

export function isExecutable(path: string): boolean {
  try {
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/** First executable among `preferred` (if set) and the fallback candidates. */
export function resolveBinary(
  name: string,
  preferred: string | undefined,
  candidates: string[],
): string {
  const ordered = [preferred?.trim(), ...candidates]
    .filter((c): c is string => !!c)
    .map(expandHome);
  for (const candidate of ordered) {
    if (isExecutable(candidate)) return candidate;
  }
  throw new Error(`${name} binary not found. Tried: ${ordered.join(", ")}`);
}

export const CMUX_CANDIDATES = [
  "/opt/homebrew/bin/cmux",
  "/usr/local/bin/cmux",
  "~/.local/bin/cmux",
];
export const CMUXINATOR_CANDIDATES = [
  "~/.cargo/bin/cmuxinator",
  "/opt/homebrew/bin/cmuxinator",
  "/usr/local/bin/cmuxinator",
  "~/.local/bin/cmuxinator",
];

export const CMUXINATOR_CONFIG_DIR = join(homedir(), ".config", "cmuxinator");
const LAST_SOCKET_FILE = join(homedir(), ".local", "state", "cmux", "last-socket-path");

/** cmux writes the socket path of the running app here; the CLI needs it when run outside cmux. */
export function readCmuxSocketPath(): string | undefined {
  try {
    const value = readFileSync(LAST_SOCKET_FILE, "utf8").trim();
    return value.length > 0 ? value : undefined;
  } catch {
    return undefined;
  }
}
