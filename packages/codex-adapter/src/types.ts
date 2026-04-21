export type ExecutionMode = "dry-run" | "execute";
export type ExecutionStatus = "dry-run" | "started" | "failed";

export interface CodexExecutionPlan {
  executionMode: ExecutionMode;
  executable: string;
  args: string[];
  command: string;
  prompt: string;
  cwd: string;
  env: NodeJS.ProcessEnv;
  isolatedCodexHome: string;
  sourceCodexHome: string | null;
}

export interface CodexExecutionResult {
  status: ExecutionStatus;
  message: string;
}
