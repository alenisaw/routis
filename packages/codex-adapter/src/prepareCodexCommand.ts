import { execFile } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

import type { RouteSelection } from "../../core/src/selectRoute.js";
import type { CodexExecutionPlan, ExecutionMode } from "./types.js";

const execFileAsync = promisify(execFile);

function quote(value: string): string {
  if (value === "") {
    return '""';
  }

  if (!/[\s"]/u.test(value)) {
    return value;
  }

  return `"${value.replace(/"/g, '\\"')}"`;
}

function getFallbackExecutable(): string {
  return process.platform === "win32" ? "codex.cmd" : "codex";
}

function getSourceCodexHome(): string | null {
  const configuredHome = process.env.CODEX_HOME?.trim();

  if (configuredHome) {
    return configuredHome;
  }

  const homeDirectory = os.homedir();

  if (!homeDirectory) {
    return null;
  }

  return path.join(homeDirectory, ".codex");
}

function getIsolatedCodexHome(cwd: string): string {
  return path.join(cwd, ".routis", "codex-home");
}

async function findCodexExecutable(): Promise<string | null> {
  try {
    if (process.platform === "win32") {
      const windowsCandidates = ["codex.cmd", "codex.exe", "codex"];

      for (const candidate of windowsCandidates) {
        try {
          const result = await execFileAsync("where.exe", [candidate], {
            windowsHide: true
          });
          const resolved = result.stdout
            .split(/\r?\n/)
            .map((line) => line.trim())
            .filter((line) => !line.includes("\\WindowsApps\\"))
            .find(Boolean);

          if (resolved) {
            return resolved;
          }
        } catch {
          // Try the next candidate.
        }
      }

      return null;
    }

    const result = await execFileAsync("which", ["codex"]);
    const resolved = result.stdout
      .split(/\r?\n/)
      .map((line) => line.trim())
      .find(Boolean);

    return resolved ?? null;
  } catch {
    return null;
  }
}

function createCodexPrompt(task: string, selection: RouteSelection): string {
  return [
    "Routis execution context.",
    `Requested policy: ${selection.requestedPolicy}.`,
    `Effective profile: ${selection.effectiveProfile}.`,
    `Selected route: ${selection.route}.`,
    `Model: ${selection.model}.`,
    `Reasoning: ${selection.reasoning}.`,
    `Task: ${task}`
  ].join(" ");
}

function buildCommandString(executable: string, args: string[]): string {
  return [quote(executable), ...args.map(quote)].join(" ");
}

function buildDisplayCommand(executable: string, args: string[], isolatedCodexHome: string): string {
  const baseCommand = buildCommandString(executable, args);
  const quotedHome = quote(isolatedCodexHome);

  if (process.platform === "win32") {
    return `set "CODEX_HOME=${isolatedCodexHome}" && ${baseCommand}`;
  }

  return `CODEX_HOME=${quotedHome} ${baseCommand}`;
}

export interface PrepareCodexCommandInput {
  task: string;
  selection: RouteSelection;
  executionMode: ExecutionMode;
  cwd: string;
}

export async function prepareCodexCommand(input: PrepareCodexCommandInput): Promise<CodexExecutionPlan> {
  const detectedExecutable = await findCodexExecutable();
  const executable = detectedExecutable ?? getFallbackExecutable();
  const prompt = createCodexPrompt(input.task, input.selection);
  const args = [
    "exec",
    "-m",
    input.selection.model,
    "-c",
    `reasoning_effort="${input.selection.reasoning}"`,
    "-C",
    input.cwd,
    prompt
  ];
  const isolatedCodexHome = getIsolatedCodexHome(input.cwd);
  const sourceCodexHome = getSourceCodexHome();
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    CODEX_HOME: isolatedCodexHome
  };

  return {
    executionMode: input.executionMode,
    executable,
    args,
    command: buildDisplayCommand(executable, args, isolatedCodexHome),
    prompt,
    cwd: input.cwd,
    env,
    isolatedCodexHome,
    sourceCodexHome
  };
}
