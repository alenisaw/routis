import { effectiveProfiles, toRouteLabel, type EffectiveProfile, type PolicyConfig } from "../../policy/src/types.js";

export interface ClassifierMatches {
  cheap: string[];
  balanced: string[];
  deep: string[];
  extradeep: string[];
  upgrade: string[];
  downgrade: string[];
}

export interface ClassifierResult {
  recommendedProfile: EffectiveProfile;
  selectedRoute: ReturnType<typeof toRouteLabel>;
  rationale: string;
  matches: ClassifierMatches;
  baseProfile: EffectiveProfile;
}

const tiePriority: EffectiveProfile[] = ["extradeep", "deep", "balanced", "cheap"];

function collectMatches(task: string, signals: string[]): string[] {
  return signals.filter((signal) => task.includes(signal));
}

function resolveBaseProfile(matches: ClassifierMatches, baselineProfile: EffectiveProfile): EffectiveProfile {
  const counts: Record<EffectiveProfile, number> = {
    cheap: matches.cheap.length,
    balanced: matches.balanced.length,
    deep: matches.deep.length,
    extradeep: matches.extradeep.length
  };

  const highestCount = Math.max(...effectiveProfiles.map((profile) => counts[profile]));

  if (highestCount === 0) {
    return baselineProfile;
  }

  return tiePriority.find((profile) => counts[profile] === highestCount) ?? baselineProfile;
}

function applyCueAdjustment(
  baseProfile: EffectiveProfile,
  upgradeCount: number,
  downgradeCount: number
): EffectiveProfile {
  const currentIndex = effectiveProfiles.indexOf(baseProfile);

  if (upgradeCount > downgradeCount) {
    return effectiveProfiles[Math.min(currentIndex + 1, effectiveProfiles.length - 1)];
  }

  if (downgradeCount > upgradeCount) {
    return effectiveProfiles[Math.max(currentIndex - 1, 0)];
  }

  return baseProfile;
}

function pickStrongestSignal(matches: ClassifierMatches, profile: EffectiveProfile): string | undefined {
  return matches[profile][0];
}

export function classifyTask(task: string, policy: PolicyConfig): ClassifierResult {
  const normalizedTask = task.trim().toLowerCase();
  const matches: ClassifierMatches = {
    cheap: collectMatches(normalizedTask, policy.signals.cheap),
    balanced: collectMatches(normalizedTask, policy.signals.balanced),
    deep: collectMatches(normalizedTask, policy.signals.deep),
    extradeep: collectMatches(normalizedTask, policy.signals.extradeep),
    upgrade: collectMatches(normalizedTask, policy.cues.upgrade),
    downgrade: collectMatches(normalizedTask, policy.cues.downgrade)
  };

  const baseProfile = resolveBaseProfile(matches, policy.behavior.baselineProfile);
  const recommendedProfile = applyCueAdjustment(
    baseProfile,
    matches.upgrade.length,
    matches.downgrade.length
  );

  const strongestSignal = pickStrongestSignal(matches, baseProfile);
  let rationale = strongestSignal
    ? `Matched ${baseProfile} signal: "${strongestSignal}".`
    : `No strong signal matched, using the ${policy.behavior.baselineProfile} baseline profile.`;

  if (recommendedProfile !== baseProfile) {
    rationale +=
      matches.upgrade.length > matches.downgrade.length
        ? " Upgrade cues pushed it higher."
        : " Downgrade cues kept it lighter.";
  }

  return {
    recommendedProfile,
    selectedRoute: toRouteLabel(recommendedProfile),
    rationale,
    matches,
    baseProfile
  };
}
