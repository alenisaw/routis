import { spawn } from "node:child_process";
import { access, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import type { CodexExecutionPlan, CodexExecutionResult } from "./types.js";

export interface ExecuteCodexRuntime {
  isExecutableAvailable: (executable: string) => Promise<boolean>;
  prepareExecutionEnvironment: (plan: CodexExecutionPlan) => Promise<void>;
  runCommand: (
    executable: string,
    args: string[],
    env: NodeJS.ProcessEnv,
    cwd: string
  ) => Promise<{ exitCode: number; stderr: string }>;
}

async function defaultIsExecutableAvailable(executable: string): Promise<boolean> {
  if (executable === "codex") {
    return true;
  }

  return pathExists(executable);
}

async function pathExists(filePath: string): Promise<boolean> {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function defaultPrepareExecutionEnvironment(plan: CodexExecutionPlan): Promise<void> {
  await mkdir(plan.isolatedCodexHome, { recursive: true });
  await mkdir(path.join(plan.isolatedCodexHome, "rules"), { recursive: true });
  await writeFile(path.join(plan.isolatedCodexHome, "config.toml"), "[features]\nmemories = false\n", "utf8");

  const sourceCodexHome = plan.sourceCodexHome;

  if (!sourceCodexHome) {
    return;
  }

  const filesToSeed = ["auth.json"];

  for (const fileName of filesToSeed) {
    const sourcePath = path.join(sourceCodexHome, fileName);
    const targetPath = path.join(plan.isolatedCodexHome, fileName);

    if (!(await pathExists(sourcePath))) {
      continue;
    }

    await copyFile(sourcePath, targetPath);
  }
}

async function resolveWindowsCmdExecutable(
  executable: string,
  args: string[]
): Promise<{ executable: string; args: string[] }> {
  const wrapperContents = await readFile(executable, "utf8");
  const match = wrapperContents.match(/"%dp0%\\([^"]+\.js)"/i);

  if (!match) {
    return { executable, args };
  }

  const cmdDirectory = path.dirname(executable);
  const relativeScriptPath = match[1].replace(/\\/g, path.sep);
  const scriptPath = path.join(cmdDirectory, relativeScriptPath);

  return {
    executable: process.execPath,
    args: [scriptPath, ...args]
  };
}

async function defaultRunCommand(
  executable: string,
  args: string[],
  env: NodeJS.ProcessEnv,
  cwd: string
): Promise<{ exitCode: number; stderr: string }> {
  const lowerExecutable = executable.toLowerCase();

  return await new Promise<{ exitCode: number; stderr: string }>((resolve, reject) => {
    void (async () => {
      const resolvedCommand =
        process.platform === "win32" && lowerExecutable.endsWith(".cmd")
          ? await resolveWindowsCmdExecutable(executable, args)
          : { executable, args };

      const child = spawn(resolvedCommand.executable, resolvedCommand.args, {
        cwd,
        env,
        shell: false,
        stdio: ["ignore", "pipe", "pipe"],
        windowsHide: true
      });
      let stderr = "";

      child.on("error", (error) => {
        reject(error);
      });

      child.on("close", (code) => {
        resolve({
          exitCode: code ?? 0,
          stderr
        });
      });

      child.stdout?.on("data", (chunk) => {
        process.stdout.write(chunk);
      });

      child.stderr?.on("data", (chunk) => {
        const text = chunk.toString();
        stderr += text;
      });
    })().catch(reject);
  });
}

const defaultRuntime: ExecuteCodexRuntime = {
  isExecutableAvailable: defaultIsExecutableAvailable,
  prepareExecutionEnvironment: defaultPrepareExecutionEnvironment,
  runCommand: defaultRunCommand
};

export async function executeCodexCommand(
  plan: CodexExecutionPlan,
  runtime: ExecuteCodexRuntime = defaultRuntime
): Promise<CodexExecutionResult> {
  const isAvailable = await runtime.isExecutableAvailable(plan.executable);

  if (!isAvailable) {
    return {
      status: "failed",
      message: "Codex CLI was not found on PATH."
    };
  }

  try {
    await runtime.prepareExecutionEnvironment(plan);
    const result = await runtime.runCommand(plan.executable, plan.args, plan.env, plan.cwd);

    if (result.exitCode === 0) {
      return {
        status: "started",
        message: `Codex CLI completed successfully using isolated CODEX_HOME at ${plan.isolatedCodexHome}.`
      };
    }

    const stderrSnippet = result.stderr.trim();
    const details = stderrSnippet ? ` ${stderrSnippet}` : "";

    return {
      status: "failed",
      message: `Codex CLI exited with code ${result.exitCode}.${details}`
    };
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);

    return {
      status: "failed",
      message: `Failed to execute Codex CLI: ${message}`
    };
  }
}
