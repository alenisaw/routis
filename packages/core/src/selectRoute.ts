import { toRouteLabel, type PolicyConfig } from "../../policy/src/types.js";
import type { ClassifierResult } from "./classifyTask.js";

export interface RouteSelectionInput {
  requestedPolicy: PolicyConfig;
  effectivePolicy: PolicyConfig;
  classifierResult: ClassifierResult;
}

export interface RouteSelection {
  requestedPolicy: string;
  effectiveProfile: string;
  model: string;
  reasoning: string;
  route: string;
  rationale: string;
}

export function selectRoute(input: RouteSelectionInput): RouteSelection {
  const { requestedPolicy, effectivePolicy, classifierResult } = input;

  return {
    requestedPolicy: requestedPolicy.name,
    effectiveProfile: effectivePolicy.name,
    model: effectivePolicy.execution.model,
    reasoning: effectivePolicy.execution.reasoning,
    route: toRouteLabel(effectivePolicy.behavior.defaultRoute),
    rationale:
      requestedPolicy.behavior.mode === "dynamic"
        ? classifierResult.rationale
        : `Requested explicit profile "${requestedPolicy.name}".`
  };
}
