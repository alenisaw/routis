# Contributing

Thanks for taking a look.

Routis is still small, so the best contributions are usually straightforward ones: a clear bug fix, a better routing rule, a cleaner CLI behavior, a tighter explanation, or a doc fix that removes ambiguity.

---

## Good Contributions

- improve routing behavior without adding a lot of machinery
- make policy files easier to understand or maintain
- tighten CLI output or error handling
- add focused tests for real routing behavior
- fix docs that no longer match the code

## Before You Open a PR

- read the current README first
- keep the change narrow
- avoid unrelated cleanup
- avoid adding dependencies unless they clearly earn their keep

Small focused change beats big rewrite here.

## Local Workflow

```bash
npm install
npm test
npm run build
```

Then open a PR with a short explanation of what changed and why.

If docs changed, make sure they still match actual behavior.

---

## Branch Names

Examples:

- `feat/policy-selection`
- `feat/better-route-rationale`
- `fix/default-profile-resolution`
- `docs/readme-accuracy`

## Commit Messages

Keep them direct.

Examples:

- `feat(cli): support explicit policy selection`
- `feat(core): add extradeep routing profile`
- `fix(policy): reject unknown policy names`
- `docs: update readme examples`

## Scope

Routis is currently about routing, policy selection, token economy, and explainability in a CLI workflow.

If a change pulls the project toward a generic chat app, desktop UI work, telemetry, or real model execution, it is probably out of scope for now.
