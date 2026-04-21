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
  type RawRoutingConfig,
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

async function readYamlObject(filePath: string, label: string): Promise<Record<string, unknown>> {
  const contents = await readFile(filePath, "utf8");
  const parsed = parse(contents);

  if (!isObject(parsed)) {
    throw new Error(`${label} must contain an object at the top level.`);
  }

  return parsed;
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

function buildRoutingPath(policyPath: string): string {
  return path.resolve(path.dirname(policyPath), "routing.yaml");
}

async function loadSharedRouting(policyPath: string): Promise<RawRoutingConfig | undefined> {
  try {
    return (await readYamlObject(buildRoutingPath(policyPath), "Routing file")) as RawRoutingConfig;
  } catch (error: unknown) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return undefined;
    }

    throw error;
  }
}

function mergeRoutingSection(
  policySection: unknown,
  sharedSection: unknown,
  fieldName: "signals" | "cues"
): Record<string, unknown> {
  if (policySection !== undefined && !isObject(policySection)) {
    throw new Error(`Policy field "${fieldName}" must be an object.`);
  }

  if (sharedSection !== undefined && !isObject(sharedSection)) {
    throw new Error(`Routing field "${fieldName}" must be an object.`);
  }

  if (policySection === undefined && sharedSection === undefined) {
    throw new Error(`Policy field "${fieldName}" must be an object or routing.yaml must provide it.`);
  }

  return {
    ...(isObject(sharedSection) ? sharedSection : {}),
    ...(isObject(policySection) ? policySection : {})
  };
}

export function getAvailablePolicyNames(): PolicyName[] {
  return [...policyNames];
}

export function isPolicyName(value: string): value is PolicyName {
  return policyNames.includes(value as PolicyName);
}

export async function loadPolicy(policyPath: string): Promise<PolicyConfig> {
  const resolvedPath = path.resolve(process.cwd(), policyPath);
  const parsed = (await readYamlObject(resolvedPath, "Policy file")) as RawPolicyConfig;
  const sharedRouting = await loadSharedRouting(resolvedPath);

  if (!isObject(parsed.execution)) {
    throw new Error('Policy field "execution" must be an object.');
  }

  if (!isObject(parsed.behavior)) {
    throw new Error('Policy field "behavior" must be an object.');
  }

  if (!isObject(parsed.output)) {
    throw new Error('Policy field "output" must be an object.');
  }

  const signals = mergeRoutingSection(parsed.signals, sharedRouting?.signals, "signals");
  const cues = mergeRoutingSection(parsed.cues, sharedRouting?.cues, "cues");

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
      cheap: readStringList(signals.cheap, "signals.cheap"),
      balanced: readStringList(signals.balanced, "signals.balanced"),
      deep: readStringList(signals.deep, "signals.deep"),
      extradeep: readStringList(signals.extradeep, "signals.extradeep")
    },
    cues: {
      upgrade: readStringList(cues.upgrade, "cues.upgrade"),
      downgrade: readStringList(cues.downgrade, "cues.downgrade")
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
