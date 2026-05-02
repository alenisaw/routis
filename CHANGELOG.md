# Changelog

## 0.2.1 - TUI CI Smoke Fix - 2026-05-03

This patch release fixes TUI smoke testing in non-interactive CI environments.

### Fixed

- Fixed `ROUTIS_TUI_SMOKE_EXIT` so the binary exits before raw mode and alternate-screen terminal initialization.
- Fixed CI smoke coverage for Ubuntu, macOS, and Windows runners without an attached TTY.
- Fixed the Dependabot `lru` advisory by updating `ratatui` to `0.30.0`, which resolves `lru` to `0.16.4`.
- Kept normal interactive TUI startup unchanged for local terminal use.

## 0.2.0 - Ratatui TUI Shell - 2026-05-03

This release adds the first Routis terminal UI shell while keeping the 0.1.0 one-shot CLI routing path unchanged. The CLI still routes tasks, applies policy files, prints Codex command previews, supports dry-run by default, and executes only when explicitly requested.

### Added

- Added a full interactive TUI shell that starts when `routis` is run without a task.
- Added the main TUI workspace: dashboard header, recent updates, recent sessions, provider/model/reasoning summary, metrics, timeline, input row, and status badge.
- Added first-run setup for local display name, provider selection, Codex CLI check, theme selection, review, and config saving to `~/.routis/config.toml`.
- Added keyboard-driven command handling for `/help`, `/status`, `/setup`, `/doctor`, `/provider`, `/theme`, `/sessions`, `/history`, `/clear`, `/config`, and `/quit`.
- Added an inline command palette with live filtering while typing slash commands.
- Added a dedicated sessions window with searchable local prompt history and keyboard selection.
- Added dedicated theme and provider picker windows outside setup, so theme/provider changes can be made from the shell.
- Added Codex provider diagnostics that search the system PATH, resolve a runnable Codex executable, show binary/version/auth/config information, and run `codex --version`.
- Added local shell history storage under `~/.routis/shell_history` and recent-session titles derived from prompts.
- Added TUI timeline events for command results, task previews, confirmation prompts, cancellation, and cleared views.
- Added confirmation flow for prepared tasks with `proceed`, `edit`, and `cancel`.
- Added responsive terminal layouts for minimal, compact, and wide widths.
- Added five dark terminal themes: Routis Cyan, Routis Violet, Neon Magenta, Midnight Blue, and Monochrome.
- Added dashboard metrics with horizontal terminal bars for context, input, output, total, and saved values.

### Changed

- Kept 0.1.0 one-shot CLI behavior unchanged for `routis --task "..."`, positional tasks, policy selection, explain output, and `--execute`.
- Updated CI to run format, clippy, tests, and release builds across Ubuntu, Windows, and macOS.
- Added an explicit read-only `contents: read` permission block to the CI workflow.

### Fixed

- Fixed Windows provider detection by preferring executable shims such as `.cmd`, `.exe`, and `.bat` over PowerShell `.ps1` wrappers that can be blocked by execution policy.
- Fixed provider setup continuation after a successful Codex CLI check.


### Quality

- Added TUI contract tests for setup, command filtering, command results, session picker behavior, provider diagnostics, layout bounds, clear behavior, exit handling, and render smoke coverage.
- Added CLI smoke tests that continue to cover the 0.1.0 routing surface.

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
