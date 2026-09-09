import { appendFileSync, mkdirSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const LOG_DIR = join(homedir(), "Library", "Logs");
export const LOG_FILE = join(LOG_DIR, "cmuxinator-raycast.log");

/** Append a line to ~/Library/Logs/cmuxinator-raycast.log. Raycast swallows console output for no-view commands. */
export function log(message: string, detail?: unknown): void {
  try {
    mkdirSync(LOG_DIR, { recursive: true });
    const extra =
      detail === undefined
        ? ""
        : " " +
          (detail instanceof Error
            ? `${detail.message}\n${detail.stack ?? ""}`
            : JSON.stringify(detail));
    appendFileSync(LOG_FILE, `${new Date().toISOString()} ${message}${extra}\n`);
  } catch {
    // Logging must never break the command.
  }
}
