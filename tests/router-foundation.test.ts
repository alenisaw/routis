import assert from "node:assert/strict";

import { runCli } from "../apps/cli/src/index.js";
import { executeCodexCommand } from "../packages/codex-adapter/src/executeCodexCommand.js";
import { prepareCodexCommand } from "../packages/codex-adapter/src/prepareCodexCommand.js";
import { classifyTask } from "../packages/core/src/classifyTask.js";
import { selectRoute } from "../packages/core/src/selectRoute.js";
import { loadPolicyPreset, loadPolicyPresetByName } from "../packages/policy/src/loadPolicy.js";
import packageJson from "../package.json" with { type: "json" };

async function main(): Promise<void> {
  const defaultPolicy = await loadPolicyPreset("default");
  const cheapPolicy = await loadPolicyPreset("cheap");
  const balancedPolicy = await loadPolicyPreset("balanced");
  const deepPolicy = await loadPolicyPreset("deep");
  const extradeepPolicy = await loadPolicyPreset("extradeep");

  assert.equal(defaultPolicy.execution.model, "dynamic");
  assert.equal(defaultPolicy.execution.reasoning, "dynamic");
  assert.equal(defaultPolicy.behavior.mode, "dynamic");
  assert.ok(defaultPolicy.signals.extradeep.includes("redesign routing"));
  assert.ok(defaultPolicy.cues.upgrade.includes("architecture"));

  assert.equal(cheapPolicy.execution.model, "gpt-5.4-mini");
  assert.equal(cheapPolicy.execution.reasoning, "none");
  assert.ok(cheapPolicy.signals.cheap.includes("typo"));

  assert.equal(balancedPolicy.execution.model, "gpt-5.4");
  assert.equal(balancedPolicy.execution.reasoning, "medium");

  assert.equal(deepPolicy.execution.model, "gpt-5.4");
  assert.equal(deepPolicy.execution.reasoning, "high");

  assert.equal(extradeepPolicy.execution.model, "gpt-5.4");
  assert.equal(extradeepPolicy.execution.reasoning, "xhigh");

  await assert.rejects(
    () => loadPolicyPresetByName("unknown"),
    /Unknown policy preset "unknown"/
  );

  const cheapTask = classifyTask("just fix typo in comment", defaultPolicy);
  assert.equal(cheapTask.recommendedProfile, "cheap");

  const balancedTask = classifyTask("implement CLI flag to load config", defaultPolicy);
  assert.equal(balancedTask.recommendedProfile, "balanced");

  const deepTask = classifyTask("debug tricky edge case in validation flow", defaultPolicy);
  assert.equal(deepTask.recommendedProfile, "deep");

  const extradeepTask = classifyTask("redesign routing architecture across files", defaultPolicy);
  assert.equal(extradeepTask.recommendedProfile, "extradeep");

  const defaultSelection = selectRoute({
    requestedPolicy: defaultPolicy,
    effectivePolicy: extradeepPolicy,
    classifierResult: extradeepTask
  });
  assert.equal(defaultSelection.requestedPolicy, "default");
  assert.equal(defaultSelection.effectiveProfile, "extradeep");
  assert.equal(defaultSelection.model, "gpt-5.4");
  assert.equal(defaultSelection.reasoning, "xhigh");
  assert.equal(defaultSelection.route, "ExtraDeep");

  const codexPlan = await prepareCodexCommand({
    task: "redesign routing architecture across files",
    selection: defaultSelection,
    executionMode: "dry-run",
    cwd: process.cwd()
  });
  assert.equal(codexPlan.executionMode, "dry-run");
  assert.match(codexPlan.command, /exec/);
  assert.match(codexPlan.command, /CODEX_HOME/);
  assert.match(codexPlan.command, /gpt-5\.4/);
  assert.match(codexPlan.command, /reasoning_effort=/);
  assert.ok(codexPlan.args.includes("exec"));
  assert.ok(codexPlan.isolatedCodexHome.includes(".routis"));

  const missingCodexExecution = await executeCodexCommand(codexPlan, {
    isExecutableAvailable: async () => false,
    prepareExecutionEnvironment: async () => {},
    runCommand: async () => ({ exitCode: 0, stderr: "" })
  });
  assert.equal(missingCodexExecution.status, "failed");
  assert.match(missingCodexExecution.message, /not found/i);

  const cheapOutput: string[] = [];
  await runCli(["--policy", "cheap", "investigate", "architecture"], {
    write: (line) => cheapOutput.push(line),
    exit: () => {
      throw new Error("cheap CLI run should not exit");
    }
  });
  assert.match(cheapOutput.join("\n"), /Requested policy: cheap/);
  assert.match(cheapOutput.join("\n"), /Effective profile: cheap/);
  assert.match(cheapOutput.join("\n"), /Model: gpt-5\.4-mini/);
  assert.match(cheapOutput.join("\n"), /Reasoning: none/);
  assert.match(cheapOutput.join("\n"), /Execution mode: dry-run/);
  assert.match(cheapOutput.join("\n"), /Codex command:/);

  const helpOutput: string[] = [];
  await runCli(["--help"], {
    write: (line) => helpOutput.push(line),
    exit: () => {
      throw new Error("help CLI run should not exit");
    }
  });
  assert.match(helpOutput.join("\n"), /CLI-first adaptive execution router for Codex/);
  assert.match(helpOutput.join("\n"), /--policy <preset>/);

  const versionOutput: string[] = [];
  await runCli(["--version"], {
    write: (line) => versionOutput.push(line),
    exit: () => {
      throw new Error("version CLI run should not exit");
    }
  });
  assert.deepEqual(versionOutput, [packageJson.version]);

  const defaultOutput: string[] = [];
  await runCli(["--policy", "default", "redesign routing architecture across files"], {
    write: (line) => defaultOutput.push(line),
    exit: () => {
      throw new Error("default CLI run should not exit");
    }
  });
  assert.match(defaultOutput.join("\n"), /Requested policy: default/);
  assert.match(defaultOutput.join("\n"), /Effective profile: extradeep/);
  assert.match(defaultOutput.join("\n"), /Selected route: ExtraDeep/);

  const executeOutput: string[] = [];
  const executeExitCodes: number[] = [];
  await runCli(
    ["--policy", "default", "--execute", "implement config loading"],
    {
      write: (line) => executeOutput.push(line),
      exit: (code) => executeExitCodes.push(code)
    },
    {
      prepareCodexCommand: async () => ({
        executionMode: "execute",
        executable: "codex.cmd",
        args: ["exec", "-m", "gpt-5.4"],
        command: 'set "CODEX_HOME=test-home" && codex.cmd exec -m gpt-5.4 -c reasoning_effort="medium" -C test -',
        prompt: "implement config loading",
        cwd: "test",
        env: { CODEX_HOME: "test-home" },
        isolatedCodexHome: "test-home",
        sourceCodexHome: "source-home"
      }),
      executeCodexCommand: async () => ({
        status: "started",
        message: "Codex CLI completed successfully using isolated CODEX_HOME at test-home."
      })
    }
  );
  assert.deepEqual(executeExitCodes, []);
  assert.match(executeOutput.join("\n"), /Execution mode: execute/);
  assert.match(executeOutput.join("\n"), /Execution status: started/);
  assert.match(executeOutput.join("\n"), /test-home/);

  const unknownOutput: string[] = [];
  const unknownExitCodes: number[] = [];
  await runCli(["--policy", "unknown", "fix typo"], {
    write: (line) => unknownOutput.push(line),
    exit: (code) => unknownExitCodes.push(code)
  });
  assert.deepEqual(unknownExitCodes, [1]);
  assert.match(unknownOutput.join("\n"), /Unknown policy preset "unknown"/);

  console.log("router-foundation smoke test passed");
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
