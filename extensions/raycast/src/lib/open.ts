import { dirname } from "node:path";
import { activateCmuxApp, CmuxClient } from "./cmux";
import { startProject } from "./cmuxinator";
import { log } from "./log";
import { findProjectWindow } from "./matcher";
import { CMUX_CANDIDATES, CMUXINATOR_CANDIDATES, resolveBinary } from "./paths";
import type { Project } from "./projects";

export interface OpenOptions {
  cmuxPath?: string;
  cmuxinatorPath?: string;
  socketPassword?: string;
  autoStart: boolean;
}

export type OpenOutcome =
  | { kind: "focused"; windowRef: string }
  | { kind: "started"; windowId?: string }
  | { kind: "not-running" };

/** Focus the project's window if cmux has one; otherwise start it with cmuxinator (if allowed). */
export async function openProject(project: Project, options: OpenOptions): Promise<OpenOutcome> {
  const cmuxBinary = resolveBinary("cmux", options.cmuxPath, CMUX_CANDIDATES);
  const cmux = new CmuxClient(cmuxBinary, options.socketPassword);

  const running = await cmux.isRunning();
  log("cmux", { binary: cmuxBinary, running });
  if (running) {
    const windows = await cmux.windows();
    const match = findProjectWindow(project, windows);
    log(
      "match",
      match ? { window: match.window.ref, score: match.score, reasons: match.reasons } : null,
    );
    if (match) {
      // Activate first: when Raycast closes, macOS restores cmux's previous key window,
      // which would override a focus-window issued before that restore lands.
      await activateCmuxApp();
      await cmux.focusWindow(match.window.id);
      return { kind: "focused", windowRef: match.window.ref };
    }
  }

  if (!options.autoStart) return { kind: "not-running" };

  const cmuxinatorBinary = resolveBinary(
    "cmuxinator",
    options.cmuxinatorPath,
    CMUXINATOR_CANDIDATES,
  );
  const result = await startProject(
    cmuxinatorBinary,
    dirname(cmuxBinary),
    project.name,
    options.socketPassword,
  );
  await activateCmuxApp();
  if (result.windowId) {
    try {
      await cmux.focusWindow(result.windowId);
    } catch {
      // The new window is normally already frontmost; focusing is best-effort.
    }
  }
  return { kind: "started", windowId: result.windowId };
}
