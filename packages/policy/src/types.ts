export const policyNames = ["default", "cheap", "balanced", "deep", "extradeep"] as const;
export const effectiveProfiles = ["cheap", "balanced", "deep", "extradeep"] as const;

export type PolicyName = (typeof policyNames)[number];
export type EffectiveProfile = (typeof effectiveProfiles)[number];
export type BehaviorMode = "fixed" | "dynamic";
export type ModelName = "gpt-5.4-mini" | "gpt-5.4" | "dynamic";
export type ReasoningLevel = "none" | "medium" | "high" | "xhigh" | "dynamic";
export type RouteLabel = "Cheap" | "Balanced" | "Deep" | "ExtraDeep";

export interface PolicyExecutionConfig {
  model: ModelName;
  reasoning: ReasoningLevel;
}

export interface PolicyBehaviorConfig {
  mode: BehaviorMode;
  baselineProfile: EffectiveProfile;
  defaultRoute: EffectiveProfile;
}

export interface PolicySignalGroups {
  cheap: string[];
  balanced: string[];
  deep: string[];
  extradeep: string[];
}

export interface PolicyCueGroups {
  upgrade: string[];
  downgrade: string[];
}

export interface PolicyOutputConfig {
  leanOutput: boolean;
  silentSuccess: boolean;
  patchFirst: boolean;
}

export interface PolicyConfig {
  name: PolicyName;
  description: string;
  execution: PolicyExecutionConfig;
  behavior: PolicyBehaviorConfig;
  signals: PolicySignalGroups;
  cues: PolicyCueGroups;
  output: PolicyOutputConfig;
}

export interface RawPolicyConfig {
  name?: unknown;
  description?: unknown;
  execution?: {
    model?: unknown;
    reasoning?: unknown;
  };
  behavior?: {
    mode?: unknown;
    baseline_profile?: unknown;
    default_route?: unknown;
  };
  signals?: {
    cheap?: unknown;
    balanced?: unknown;
    deep?: unknown;
    extradeep?: unknown;
  };
  cues?: {
    upgrade?: unknown;
    downgrade?: unknown;
  };
  output?: {
    lean_output?: unknown;
    silent_success?: unknown;
    patch_first?: unknown;
  };
}

export interface RawRoutingConfig {
  signals?: RawPolicyConfig["signals"];
  cues?: RawPolicyConfig["cues"];
}

export function toRouteLabel(profile: EffectiveProfile): RouteLabel {
  switch (profile) {
    case "cheap":
      return "Cheap";
    case "balanced":
      return "Balanced";
    case "deep":
      return "Deep";
    case "extradeep":
      return "ExtraDeep";
  }
}
