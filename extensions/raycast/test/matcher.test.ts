import assert from "node:assert/strict";
import { test } from "node:test";
import type { CmuxWindow } from "../src/lib/cmux";
import { findProjectWindow, matchAll } from "../src/lib/matcher";
import { cwdsFromCommand, findProject, parseProject } from "../src/lib/projects";

const HOME = process.env.HOME!;

const dex = parseProject(
  "/x/dex.yml",
  `
name: dex
workspaces:
  - name: agents
    cwd: ~/codebase/dex-ios
    panes:
      - surfaces: [{ name: din-djarin, command: claude }]
      - surfaces: [{ name: grogu, command: pi }]
  - name: dev
    cwd: ~/codebase/dex-ios
    panes:
      - surfaces: [{ name: dex }]
      - surfaces: [{ name: magellan, command: "cd ~/codebase/magellan && exec $SHELL -l" }]
`,
);

const hoAgents = parseProject(
  "/x/ho-agents.yml",
  `
name: ho-agents
workspaces:
  - name: agents
    cwd: ~/codebase/ho-agents
    panes:
      - surfaces: [{ name: agent-1, command: claude }]
      - surfaces: [{ name: agent-2, command: claude }]
  - name: dev
    cwd: ~/codebase/ho-agents
    panes:
      - surfaces: [{ name: yazi, command: yazi }]
      - surfaces: [{ name: git }]
`,
);

const hooter = parseProject(
  "/x/hooter.yml",
  `
name: hooter
workspaces:
  - name: agents
    cwd: ~/codebase/hooter-core
    panes:
      - surfaces: [{ name: hooter-core-agent, command: pi }]
`,
);

function win(id: string, index: number, workspaces: [string, string, string[]][]): CmuxWindow {
  return {
    id,
    ref: `window:${index + 1}`,
    index,
    key: false,
    workspaces: workspaces.map(([title, cwd, surfaces], i) => ({
      id: `${id}-ws${i}`,
      title,
      currentDirectory: cwd,
      surfaces: surfaces.map((t, j) => ({ id: `${id}-s${i}${j}`, title: t, type: "terminal" })),
    })),
  };
}

// Mirrors the live cmux state this was built against: two windows, both titled agents/dev.
const dexWindow = win("W-DEX", 0, [
  ["agents", `${HOME}/codebase/dex-ios`, ["din-djarin", "grogu"]],
  ["dev", `${HOME}/codebase/dex-ios`, ["dex", "magellan"]],
]);
const hoWindow = win("W-HO", 1, [
  ["agents", `${HOME}/codebase/ho-agents`, ["agent-1", "agent-2"]],
  ["dev", `${HOME}/codebase/ho-agents`, ["yazi", "git"]],
]);

test("parseProject expands cwds and collects cd targets from commands", () => {
  assert.deepEqual(dex.cwds.sort(), [`${HOME}/codebase/dex-ios`, `${HOME}/codebase/magellan`].sort());
  assert.deepEqual(dex.workspaces[0].surfaceNames, ["din-djarin", "grogu"]);
  assert.deepEqual(cwdsFromCommand('cd "/a b" && pi; cd ~/c'), ["/a b", `${HOME}/c`]);
});

test("identical workspace titles are disambiguated by surface names and cwd", () => {
  assert.equal(findProjectWindow(hoAgents, [dexWindow, hoWindow])?.window.id, "W-HO");
  assert.equal(findProjectWindow(dex, [dexWindow, hoWindow])?.window.id, "W-DEX");
  assert.equal(findProjectWindow(hooter, [dexWindow, hoWindow]), undefined);
});

test("surfaces renamed by the terminal still match via workspace titles + cwd", () => {
  const renamed = win("W-HO2", 2, [
    ["agents", `${HOME}/codebase/ho-agents/src`, ["✳ claude", "zsh"]],
    ["dev", `${HOME}/codebase/ho-agents`, ["yazi: src", "git"]],
  ]);
  assert.equal(findProjectWindow(hoAgents, [dexWindow, renamed])?.window.id, "W-HO2");
  // Same titles but a cwd outside the project must not match.
  const foreign = win("W-X", 3, [["agents", `${HOME}/codebase/other`, ["a", "b"]], ["dev", `${HOME}/codebase/other`, ["c", "d"]]]);
  assert.equal(findProjectWindow(hoAgents, [foreign]), undefined);
});

test("matchAll assigns each window to at most one project", () => {
  const all = matchAll([dex, hoAgents, hooter], [dexWindow, hoWindow]);
  assert.equal(all.get("dex")?.window.id, "W-DEX");
  assert.equal(all.get("ho-agents")?.window.id, "W-HO");
  assert.equal(all.has("hooter"), false);
});

test("findProject resolves exact, unique-prefix and unique-substring queries", () => {
  const projects = [dex, hoAgents, hooter];
  assert.equal(findProject(projects, "ho-agents")?.name, "ho-agents");
  assert.equal(findProject(projects, "HO-AG")?.name, "ho-agents");
  assert.equal(findProject(projects, "agents")?.name, "ho-agents");
  assert.equal(findProject(projects, "ho"), undefined, "ho matches ho-agents and hooter");
  assert.equal(findProject(projects, "nope"), undefined);
});
