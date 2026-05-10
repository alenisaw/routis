# Changelog

## 0.4.0 - Auditable Routing - 2026-05-11

This release makes Routis routing decisions auditable, explainable, and safer to inspect locally. Routis now records structured Decision Trace data, renders compact explain trees, protects raw task text from trace/session surfaces by default, and adds eval coverage so future routing changes can be checked against explicit expectations.

### Added

- Added `DecisionTrace` schema in `routis-core` with schema version, session id, task hash, timestamp, language, intent, area, scope, risk, confidence, matched signals, repository facts, risk zones, policy source, policy overrides, selected profile, selected model, selected reasoning, prompt mode, execution mode, provider command preview, and route tree.
- Added compact route tree rendering for CLI explain output.
- Added HMAC-SHA256 task hashing for CLI trace records with a per-install secret.
- Added JSONL trace storage under `~/.routis/traces/<session_id>.jsonl`.
- Added automatic trace creation for `routis route <task>`.
- Added `routis route --explain <task>` to print the decision tree after the normal route summary.
- Added `routis traces` for recent local trace summaries.
- Added `routis traces --latest` for printing the latest full decision tree.
- Added live TUI Decision Trace state and panel wiring for route decisions.
- Added TUI route failure state so routing errors are shown explicitly instead of being converted into a fabricated fallback plan.
- Added private local file helpers for sensitive Routis runtime state.
- Added recognition eval fixtures for routing classification behavior.
- Added explain-tree snapshot fixtures for stable auditable routing output.
- Added CLI smoke coverage for `route --explain` trace creation and `traces --latest`.

### Fixed

- Fixed Routis home resolution so persistent runtime data defaults to `~/.routis`, honors `ROUTIS_HOME`, and fails explicitly when the user home cannot be resolved.
- Fixed trace secret handling so empty or malformed secret files fail safely instead of weakening task hashes.
- Fixed CLI session ids so trace files use generated filesystem-safe ids instead of merging unrelated route decisions into one `cli` trace.
- Fixed trace file and secret creation to use restrictive Unix permissions where supported.
- Fixed session persistence so route sessions no longer store raw task text by default.
- Fixed session persistence so session hashes are not derived directly from raw task text.
- Fixed legacy session deserialization so old raw task values are not restored into visible task previews.
- Fixed CLI route output so raw task text is not printed by default.
- Fixed TUI task events so raw task text is not rendered in the local timeline by default.
- Fixed TUI `/route` command output so task text is redacted in command results.
- Fixed history-backed session picker display so raw prompts are not rendered while in-memory resume behavior remains available.
- Fixed persisted shell history handling by writing it through private local file helpers.
- Fixed trace sanitization for common API keys, bearer tokens, GitHub tokens, npm tokens, AWS key markers, `.env` paths, JWT-like values, private key blocks, OpenSSH private key blocks, and raw task arguments.
- Fixed provider command preview to use an argv-style redacted preview instead of unsafe shell-style command text.
- Fixed `git status --porcelain=v1 -z` rename and copy parsing so the new target path is used for changed-file context.
- Fixed repository context collection so nested working directories are resolved through the repository root.
- Fixed policy path matching so explicit target hints such as `README.md` can trigger path rules even before the file is modified.
- Fixed provider detection so missing non-Windows `codex` binaries are not reported as found.
- Fixed default display-name fallback to use `User` instead of an author-specific name.
- Fixed route explanation text after policy rules are applied, so `reason` no longer describes the pre-policy profile when a policy rule changes the selected profile.
- Fixed routing error handling in the TUI so failed route planning is displayed as an error instead of producing a fake balanced plan.
- Fixed broad debug detection by avoiding `trace` as a standalone debug signal, preserving `stack trace` as the intended debug phrase.
- Fixed Russian and mixed-language recognition fixtures for docs, UI, release, security, and auth-related prompts.
- Fixed TUI contract tests to match privacy-preserving task display behavior.

### Changed

- Changed local runtime storage from executable-local `.routis` behavior to `~/.routis` by default.
- Replaced custom config serialization with TOML and schema versioning.
- Replaced synthetic internal percentage metric names with explicit routing hint fields.
- Replaced shell-style provider command formatting in policy tests with argv-list assertions.
- Updated default policy path text to `~/.routis/policies/default.yaml`.
- Updated session picker behavior so displayed history labels are redacted while task execution can still use the selected in-memory task.
- Updated eval coverage to include both routing expectations and explain-tree output fragments.

### Quality

- Added tests for HMAC task hashing stability and invalid trace secret handling.
- Added tests proving trace JSON does not contain raw task text or representative secret values.
- Added tests for sanitizer redaction of common token and secret formats.
- Added tests for private session persistence without raw task text.
- Added tests for repository-root context collection from nested directories.
- Added tests for target path parsing for renamed and copied files.
- Added tests for corrupt JSONL trace handling.
- Added tests for explicit routing error behavior instead of silent fallback.
- Added recognition eval cases for Russian and mixed-language prompts.
- Added explain-tree snapshot fragment checks for docs, auth, architecture, release, and Russian UI fix scenarios.
- Expanded TUI contract coverage for privacy-preserving task display, redacted session picker output, and live trace panel behavior.

### Known limitations

- Windows runtime-file privacy currently relies on user-profile ACLs. Strict Windows ACL hardening is planned as a separate platform-specific pass.
- Live TUI execution still keeps the active task in memory until routing/execution preparation finishes. Persistent trace and session records do not store raw task text by default.
- Configurable history modes are not included yet. Persisted history is privacy-hardened by default.

## 0.3.0 - Repo-Aware Routing and Local Sessions - 2026-05-08

This release turns Routis into a repository-aware routing layer. Routing decisions now use local project context, policy rules, risk zones, and persisted session records instead of relying only on task wording.

### Added

- Added Routing IR for task classification, including intent, area, scope, risk level, confidence, language hint, target hints, and routing evidence.
- Added repository context collection for branch name, changed files, repository markers, manifests, documentation files, tests, workflows, instruction files, and detected risk zones.
- Added repo-aware route planning that combines task classification, repository context, policy rules, selected profile, model, reasoning level, and explain output into one structured route plan.
- Added `routis route <task>` for CLI route preview.
- Added `routis context` for printing a compact repository context summary from the current working directory.
- Added `/route <task>` in the TUI to preview routing decisions interactively.
- Added `/context` in the TUI to inspect branch, changed files, repository markers, manifests, docs, tests, workflows, and instruction files.
- Added `/policy-file <path>` in the TUI for switching the active routing policy file during a shell session.
- Added local route session storage with JSON records containing task, branch, policy, effective profile, model, reasoning level, timestamps, and routing count.
- Added session lookup by id prefix or generated session title.
- Added legacy session record reading for older local session formats.
- Added install-local Routis runtime paths through `.routis`, with support for `ROUTIS_HOME`.
- Added embedded fallback loading for the default policy when no installed local policy exists.
- Added CodeQL workflow for Rust security analysis.

### Changed

- Made automatic routing repository-aware instead of relying only on task wording.
- Policy rules can now raise or cap routing profiles based on repository risk zones and path matches.
- High-risk repository areas such as auth, schema, workflow, and package files can automatically raise default routing to a deeper profile.
- Large changed-file sets can raise the route to `deep` or `extradeep`.
- README-focused tasks can be capped to cheaper routing when no stronger non-documentation intent is detected.
- Reworked TUI task planning around prompt-first route preview, visible runtime context, confirmation choices, and clearer route explanation.
- Updated `/status`, `/context`, `/sessions`, and task planning events to show repository and session context.
- Moved runtime config, history, sessions, and default policy handling toward install-local `.routis` storage.
- Updated CI structure into separate format, clippy, test, and build jobs.
- Aligned workspace crate versions for `routis`, `routis-core`, `routis-context`, and `routis-policy`.

### Fixed

- Fixed routing behavior for longer prompts where documentation-related words could override stronger routing, context, refactor, or implementation intent.
- Fixed profile selection for repository-sensitive work by adding repository context to the final routing decision.
- Fixed policy loading so a broken installed default policy reports an actual loading error instead of silently falling back to the embedded default policy.
- Fixed repeated session title collisions by using timestamp-based session ids.
- Fixed session ordering so newest or most recently updated sessions appear first.
- Fixed session lookup for both title-based and id-prefix-based selection.
- Fixed literal `\n` handling in stored session task text.

### Quality

- Added tests for routing classification, repo-aware profile escalation, explicit policy override behavior, README caps, longer mixed-intent prompts, and UI-area classification.
- Added tests for installed default policy error handling.
- Added tests for session save/list/find behavior, ordering, id uniqueness, and literal escape preservation.
- Expanded TUI contract coverage around route previews, repo context rendering, session storage, command behavior, and shell state expectations.

---

## 0.2.2 - TUI Command and Layout Polish - 2026-05-05

This patch release improves TUI command behavior, session rendering, command history, and layout polish.

### Fixed

- Fixed slash command output so `/status`, `/doctor`, `/config`, `/history`, `/provider`, `/theme`, `/sessions`, and `/clear` render visible timeline or session events instead of only mutating internal state.
- Fixed command history persistence so submitted commands are saved immediately.
- Fixed command history persistence for palette-selected slash commands.
- Fixed provider picker confirmation so selecting Codex CLI runs diagnostics, accepts the provider, and closes the picker when Codex is available.
- Fixed session picker keyboard-selection coverage.
- Fixed accumulation of command-result events across consecutive slash commands.

### Changed

- Updated TUI version labels to use the package version automatically instead of hard-coded release text.
- Reduced dashboard header density for a cleaner home view.
- Removed the `more CHANGELOG` prompt from the home screen.
- Restored the home dashboard `Recent Sessions` section.

### Quality

- Expanded TUI contract tests for command output, provider selection, history rendering, session selection, command palette limits, autoscroll behavior, layout expectations, and version display.

---

## 0.2.1 - TUI CI Smoke Fix - 2026-05-03

This patch release fixes non-interactive TUI testing and resolves a dependency advisory.

### Fixed

- Fixed `ROUTIS_TUI_SMOKE_EXIT` so the binary exits before raw mode and alternate-screen terminal initialization.
- Fixed CI smoke coverage for Ubuntu, macOS, and Windows runners without an attached TTY.
- Fixed the Dependabot `lru` advisory by updating `ratatui` to `0.30.0`, resolving `lru` to `0.16.4`.
- Kept normal interactive TUI startup unchanged for local terminal use.

---

## 0.2.0 - Ratatui TUI Shell - 2026-05-03

This release adds the first interactive terminal UI for Routis while keeping the one-shot CLI routing flow available.

### Added

- Added a full Ratatui-based interactive TUI shell that starts when `routis` is run without a task.
- Added the main TUI workspace with dashboard header, recent updates, recent sessions, provider/model/reasoning summary, metrics, timeline, input row, and status badge.
- Added first-run setup for local display name, provider selection, Codex CLI check, theme selection, review, and config saving.
- Added keyboard-driven slash command handling for `/help`, `/status`, `/setup`, `/doctor`, `/provider`, `/theme`, `/sessions`, `/history`, `/clear`, `/config`, and `/quit`.
- Added inline command palette with live filtering while typing slash commands.
- Added dedicated sessions window with searchable local prompt history and keyboard selection.
- Added dedicated theme picker outside setup.
- Added dedicated provider picker outside setup.
- Added Codex provider diagnostics that search the system PATH, resolve a runnable Codex executable, show binary/version/auth/config information, and run `codex --version`.
- Added local shell history storage and recent-session titles derived from submitted prompts.
- Added TUI timeline events for command results, task previews, confirmation prompts, cancellation, and cleared views.
- Added confirmation flow for prepared tasks with `proceed`, `edit`, and `cancel`.
- Added responsive terminal layouts for minimal, compact, and wide terminal widths.
- Added five dark terminal themes: Routis Cyan, Routis Violet, Neon Magenta, Midnight Blue, and Monochrome.
- Added dashboard metrics with horizontal terminal bars for context, input, output, total, and saved values.

### Changed

- Preserved one-shot CLI behavior for `routis --task "..."`, positional tasks, policy selection, explain output, dry-run mode, and `--execute`.
- Updated CI to run format checks, clippy, tests, and release builds across Ubuntu, Windows, and macOS.
- Added explicit read-only `contents: read` permissions to the CI workflow.
- Updated `.gitignore` for local Routis runtime data, local agent files, generated graph artifacts, archives, and environment files.

### Fixed

- Fixed Windows provider detection by preferring executable shims such as `.cmd`, `.exe`, and `.bat` over PowerShell `.ps1` wrappers that may be blocked by execution policy.
- Fixed provider setup continuation after a successful Codex CLI check.

### Quality

- Added TUI contract tests for setup flow, command filtering, command result rendering, session picker behavior, provider diagnostics, layout bounds, clear behavior, exit handling, and render smoke coverage.
- Added CLI smoke tests to ensure the routing CLI remains covered after adding the TUI.

---

## 0.1.0 - Rust CLI Foundation - 2026-04-28

This release introduces Routis as a focused Rust CLI for routing AI coding tasks before execution.

### Added

- Added the initial `routis` Rust CLI binary.
- Added rule-based routing across five profiles: `default`, `cheap`, `balanced`, `deep`, and `extradeep`.
- Added task classification and profile resolution for small edits, implementation tasks, debugging, refactoring, architecture work, and explicit policy overrides.
- Added Codex command planning with model and reasoning selection driven by policy configuration.
- Added dry-run as the default execution mode.
- Added explicit execution through `--execute`.
- Added explain output through `--explain`.
- Added positional task routing, for example `routis "fix typo in README"`.
- Added `--task` support for passing a task by flag.
- Added `--policy <profile>` for forcing a routing profile.
- Added `--policy-file <path>` for loading a custom execution policy.
- Added default YAML policy configuration.
- Moved profile-to-model and profile-to-reasoning mappings out of Rust code and into policy configuration.
- Added policy validation for missing profile configuration, empty policy fields, unsupported policy versions, and unknown YAML fields.

### Quality

- Added unit tests for routing and policy behavior.
- Added CLI smoke tests with `assert_cmd`.
- Added GitHub Actions CI for Ubuntu, Windows, and macOS.
- Added tag-triggered release workflow for binary artifacts.
