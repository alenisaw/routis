import path from "node:path";
import { fileURLToPath } from "node:url";

import { executeCodexCommand } from "../../../packages/codex-adapter/src/executeCodexCommand.js";
import { prepareCodexCommand } from "../../../packages/codex-adapter/src/prepareCodexCommand.js";
import type { ExecutionMode } from "../../../packages/codex-adapter/src/types.js";
import { classifyTask } from "../../../packages/core/src/classifyTask.js";
import { selectRoute } from "../../../packages/core/src/selectRoute.js";
import {
  getAvailablePolicyNames,
  loadPolicyPreset,
  loadPolicyPresetByName
} from "../../../packages/policy/src/loadPolicy.js";
import type { EffectiveProfile, PolicyConfig, PolicyName } from "../../../packages/policy/src/types.js";

const CLI_VERSION = "0.1.1";

export interface CliRuntime {
  write: (line: string) => void;
  exit: (code: number) => void;
}

export interface ParsedCliArgs {
  kind: "run" | "help" | "version";
  task: string;
  requestedPolicyName: PolicyName;
  executionMode: ExecutionMode;
}

export interface CliDependencies {
  prepareCodexCommand: typeof prepareCodexCommand;
  executeCodexCommand: typeof executeCodexCommand;
}

const defaultRuntime: CliRuntime = {
  write: (line) => console.log(line),
  exit: (code) => {
    process.exitCode = code;
  }
};

const defaultDependencies: CliDependencies = {
  prepareCodexCommand,
  executeCodexCommand
};

function usageLine(): string {
  return 'Usage: routis [--policy <preset>] [--dry-run|--execute] "task text"';
}

function availablePoliciesLine(): string {
  return `Available policies: ${getAvailablePolicyNames().join(", ")}`;
}

function helpLines(): string[] {
  return [
    "Routis",
    "CLI-first adaptive execution router for Codex.",
    "",
    usageLine(),
    "",
    "Options:",
    "  --policy <preset>  Select default, cheap, balanced, deep, or extradeep.",
    "  --dry-run          Print the execution plan and Codex command.",
    "  --execute          Run Codex CLI with the selected profile.",
    "  -h, --help         Show this help message.",
    "  -v, --version      Show the current CLI version.",
    "",
    availablePoliciesLine()
  ];
}

function parseCliArgs(args: string[]): ParsedCliArgs {
  const taskParts: string[] = [];
  let requestedPolicyName = "default";
  let executionMode: ExecutionMode = "dry-run";
  let modeWasSet = false;

  for (let index = 0; index < args.length; index += 1) {
    const currentArg = args[index];

    if (currentArg === "--help" || currentArg === "-h") {
      return {
        kind: "help",
        task: "",
        requestedPolicyName: "default",
        executionMode: "dry-run"
      };
    }

    if (currentArg === "--version" || currentArg === "-v") {
      return {
        kind: "version",
        task: "",
        requestedPolicyName: "default",
        executionMode: "dry-run"
      };
    }

    if (currentArg === "--policy") {
      const nextArg = args[index + 1];

      if (!nextArg) {
        throw new Error('Missing value for "--policy".');
      }

      requestedPolicyName = nextArg.toLowerCase();
      index += 1;
      continue;
    }

    if (currentArg === "--dry-run" || currentArg === "--execute") {
      if (modeWasSet && executionMode !== currentArg.replace("--", "") as ExecutionMode) {
        throw new Error('Use either "--dry-run" or "--execute", not both.');
      }

      executionMode = currentArg === "--execute" ? "execute" : "dry-run";
      modeWasSet = true;
      continue;
    }

    taskParts.push(currentArg);
  }

  const task = taskParts.join(" ").trim();

  if (!task) {
    throw new Error("Task input is required.");
  }

  return {
    kind: "run",
    task,
    requestedPolicyName: requestedPolicyName as PolicyName,
    executionMode
  };
}

function resolveEffectiveProfile(requestedPolicy: PolicyConfig, classifierProfile: EffectiveProfile): EffectiveProfile {
  if (requestedPolicy.behavior.mode === "dynamic") {
    return classifierProfile;
  }

  return requestedPolicy.behavior.baselineProfile;
}

export async function runCli(
  args: string[],
  runtime: CliRuntime = defaultRuntime,
  dependencies: CliDependencies = defaultDependencies
): Promise<void> {
  try {
    const parsedArgs = parseCliArgs(args);

    if (parsedArgs.kind === "help") {
      for (const line of helpLines()) {
        runtime.write(line);
      }

      return;
    }

    if (parsedArgs.kind === "version") {
      runtime.write(CLI_VERSION);
      return;
    }

    const requestedPolicy = await loadPolicyPresetByName(parsedArgs.requestedPolicyName);
    const classifierResult = classifyTask(parsedArgs.task, requestedPolicy);
    const effectiveProfile = resolveEffectiveProfile(requestedPolicy, classifierResult.recommendedProfile);
    const effectivePolicy = await loadPolicyPreset(effectiveProfile);
    const selection = selectRoute({
      requestedPolicy,
      effectivePolicy,
      classifierResult
    });
    const codexPlan = await dependencies.prepareCodexCommand({
      task: parsedArgs.task,
      selection,
      executionMode: parsedArgs.executionMode,
      cwd: process.cwd()
    });

    runtime.write(`Requested policy: ${selection.requestedPolicy}`);
    runtime.write(`Effective profile: ${selection.effectiveProfile}`);
    runtime.write(`Model: ${selection.model}`);
    runtime.write(`Reasoning: ${selection.reasoning}`);
    runtime.write(`Selected route: ${selection.route}`);
    runtime.write(`Why: ${selection.rationale}`);
    runtime.write(`Execution mode: ${codexPlan.executionMode}`);
    runtime.write(`Codex command: ${codexPlan.command}`);

    if (parsedArgs.executionMode === "execute") {
      const executionResult = await dependencies.executeCodexCommand(codexPlan);
      runtime.write(`Execution status: ${executionResult.status}`);
      runtime.write(`Execution message: ${executionResult.message}`);

      if (executionResult.status === "failed") {
        runtime.exit(1);
      }
    }
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);

    runtime.write(`Error: ${message}`);
    runtime.write(usageLine());
    runtime.write(availablePoliciesLine());
    runtime.exit(1);
  }
}

const isEntrypoint =
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url));

if (isEntrypoint) {
  runCli(process.argv.slice(2)).catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Error: ${message}`);
    process.exitCode = 1;
  });
}
