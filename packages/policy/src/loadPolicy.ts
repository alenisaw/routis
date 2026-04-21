import { readFile } from "node:fs/promises";
import path from "node:path";

import { parse } from "yaml";

import {
  effectiveProfiles,
  policyNames,
  type BehaviorMode,
  type EffectiveProfile,
  type ModelName,
  type PolicyConfig,
  type PolicyName,
  type RawPolicyConfig,
  type ReasoningLevel
} from "./types.js";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function readString(value: unknown, fieldName: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`Policy field "${fieldName}" must be a non-empty string.`);
  }

  return value;
}

function readBoolean(value: unknown, fieldName: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Policy field "${fieldName}" must be a boolean.`);
  }

  return value;
}

function readStringList(value: unknown, fieldName: string): string[] {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string" || entry.trim() === "")) {
    throw new Error(`Policy field "${fieldName}" must be a list of non-empty strings.`);
  }

  return value.map((entry) => entry.toLowerCase());
}

function normalizePolicyName(value: unknown): PolicyName {
  const name = readString(value, "name").toLowerCase();

  if (!policyNames.includes(name as PolicyName)) {
    throw new Error(`Policy field "name" must be one of: ${policyNames.join(", ")}.`);
  }

  return name as PolicyName;
}

function normalizeModel(value: unknown): ModelName {
  const model = readString(value, "execution.model");

  if (model !== "gpt-5.4-mini" && model !== "gpt-5.4" && model !== "dynamic") {
    throw new Error('Policy field "execution.model" must be one of: gpt-5.4-mini, gpt-5.4, dynamic.');
  }

  return model;
}

function normalizeReasoning(value: unknown): ReasoningLevel {
  const reasoning = readString(value, "execution.reasoning").toLowerCase();

  if (
    reasoning !== "none" &&
    reasoning !== "medium" &&
    reasoning !== "high" &&
    reasoning !== "xhigh" &&
    reasoning !== "dynamic"
  ) {
    throw new Error(
      'Policy field "execution.reasoning" must be one of: none, medium, high, xhigh, dynamic.'
    );
  }

  return reasoning;
}

function normalizeBehaviorMode(value: unknown): BehaviorMode {
  const mode = readString(value, "behavior.mode").toLowerCase();

  if (mode !== "fixed" && mode !== "dynamic") {
    throw new Error('Policy field "behavior.mode" must be one of: fixed, dynamic.');
  }

  return mode;
}

function normalizeEffectiveProfile(value: unknown, fieldName: string): EffectiveProfile {
  const profile = readString(value, fieldName).toLowerCase();

  if (!effectiveProfiles.includes(profile as EffectiveProfile)) {
    throw new Error(`Policy field "${fieldName}" must be one of: ${effectiveProfiles.join(", ")}.`);
  }

  return profile as EffectiveProfile;
}

function buildPolicyPath(policyName: PolicyName): string {
  return path.resolve(process.cwd(), "configs", "policies", `${policyName}.yaml`);
}

export function getAvailablePolicyNames(): PolicyName[] {
  return [...policyNames];
}

export function isPolicyName(value: string): value is PolicyName {
  return policyNames.includes(value as PolicyName);
}

export async function loadPolicy(policyPath: string): Promise<PolicyConfig> {
  const resolvedPath = path.resolve(process.cwd(), policyPath);
  const contents = await readFile(resolvedPath, "utf8");
  const parsed = parse(contents) as RawPolicyConfig;

  if (!isObject(parsed)) {
    throw new Error("Policy file must contain an object at the top level.");
  }

  if (!isObject(parsed.execution)) {
    throw new Error('Policy field "execution" must be an object.');
  }

  if (!isObject(parsed.behavior)) {
    throw new Error('Policy field "behavior" must be an object.');
  }

  if (!isObject(parsed.signals)) {
    throw new Error('Policy field "signals" must be an object.');
  }

  if (!isObject(parsed.cues)) {
    throw new Error('Policy field "cues" must be an object.');
  }

  if (!isObject(parsed.output)) {
    throw new Error('Policy field "output" must be an object.');
  }

  return {
    name: normalizePolicyName(parsed.name),
    description: readString(parsed.description, "description"),
    execution: {
      model: normalizeModel(parsed.execution.model),
      reasoning: normalizeReasoning(parsed.execution.reasoning)
    },
    behavior: {
      mode: normalizeBehaviorMode(parsed.behavior.mode),
      baselineProfile: normalizeEffectiveProfile(parsed.behavior.baseline_profile, "behavior.baseline_profile"),
      defaultRoute: normalizeEffectiveProfile(parsed.behavior.default_route, "behavior.default_route")
    },
    signals: {
      cheap: readStringList(parsed.signals.cheap, "signals.cheap"),
      balanced: readStringList(parsed.signals.balanced, "signals.balanced"),
      deep: readStringList(parsed.signals.deep, "signals.deep"),
      extradeep: readStringList(parsed.signals.extradeep, "signals.extradeep")
    },
    cues: {
      upgrade: readStringList(parsed.cues.upgrade, "cues.upgrade"),
      downgrade: readStringList(parsed.cues.downgrade, "cues.downgrade")
    },
    output: {
      leanOutput: readBoolean(parsed.output.lean_output, "output.lean_output"),
      silentSuccess: readBoolean(parsed.output.silent_success, "output.silent_success"),
      patchFirst: readBoolean(parsed.output.patch_first, "output.patch_first")
    }
  };
}

export async function loadPolicyPreset(policyName: PolicyName): Promise<PolicyConfig> {
  return loadPolicy(buildPolicyPath(policyName));
}

export async function loadPolicyPresetByName(policyName: string): Promise<PolicyConfig> {
  const normalized = policyName.trim().toLowerCase();

  if (!isPolicyName(normalized)) {
    throw new Error(
      `Unknown policy preset "${policyName}". Available presets: ${getAvailablePolicyNames().join(", ")}.`
    );
  }

  return loadPolicyPreset(normalized);
}
