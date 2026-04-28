# Changelog

## 0.1.0 - Rust CLI Foundation - 2026-04-28

Routis starts as a focused Rust command-line tool for routing AI coding tasks before execution. This release introduces the core routing loop: classify a task, choose an execution profile, build a Codex command from policy, and keep dry-run as the default path.

### Highlights

- Introduced the `routis` Rust CLI binary.
- Added rule-based routing across five profiles: `default`, `cheap`, `balanced`, `deep`, and `extradeep`.
- Added Codex command planning with model and reasoning selection driven by YAML policy.
- Made dry-run the default execution mode.
- Added explicit execution through `--execute`.
- Added explain output through `--explain`.

### CLI

- `routis "fix typo in README"` routes a positional task.
- `routis --task "debug auth flow"` routes a task passed by flag.
- `routis --policy deep "debug auth flow"` forces a profile.
- `routis --policy-file ./configs/policies/default.yaml "update config loader"` loads a custom execution policy.

### Policy

- Added `configs/policies/default.yaml`.
- Moved profile-to-model and profile-to-reasoning mappings out of Rust code.
- Added validation for missing profile configuration, empty policy fields, unsupported policy versions, and unknown YAML fields.

### Quality

- Added unit tests for routing and policy behavior.
- Added CLI smoke tests with `assert_cmd`.
- Added GitHub Actions CI for Ubuntu, Windows, and macOS.
- Added tag-triggered release workflow for binary artifacts.
