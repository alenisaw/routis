<p align="center">
  <img src="https://em-content.zobj.net/source/apple/391/compass_1f9ed.png" width="92" alt="Routis compass" />
</p>

<h1 align="center">Routis</h1>

<p align="center">
  <strong>Adaptive execution routing for AI coding workflows.</strong>
</p>

<p align="center">
  <a href="https://github.com/alenisaw/routis/releases/latest"><img src="https://img.shields.io/github/v/release/alenisaw/routis?style=flat&label=release&color=58A6FF" alt="Release"></a>
  <a href="https://github.com/alenisaw/routis/stargazers"><img src="https://img.shields.io/github/stars/alenisaw/routis?style=flat&color=F4C430" alt="Stars"></a>
  <a href="https://github.com/alenisaw/routis/commits/main"><img src="https://img.shields.io/github/last-commit/alenisaw/routis?style=flat" alt="Last Commit"></a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/alenisaw/routis?style=flat&color=97CA00" alt="License"></a>
  <a href="https://github.com/alenisaw/routis/actions/workflows/ci.yml"><img src="https://github.com/alenisaw/routis/actions/workflows/ci.yml/badge.svg?branch=main&event=push" alt="CI"></a>
</p>

<p align="center">
  <a href="#why-routis">Why Routis</a>
  | <a href="#capabilities">Capabilities</a>
  | <a href="#installation">Installation</a>
  | <a href="#usage">Usage</a>
  | <a href="#policy-file">Policy File</a>
  | <a href="#development">Development</a>
</p>

---

Routis is an execution-intelligence layer for AI coding workflows.

It adds a deliberate routing step before execution. Given a task, Routis classifies the request, reads repository signals, applies local policy, selects an execution profile, and explains why that route was chosen.

The point is simple: AI-assisted development should not use the same execution path for every task. Small edits should stay lightweight. Risky changes should receive stronger handling. Model choice, reasoning depth, and command shape should be explicit instead of hidden in shell habits.

## Why Routis

AI coding tasks do not all deserve the same execution depth. A typo fix, a focused implementation task, and a risky debugging session should not be routed through identical model and reasoning settings.

Routis makes that decision explicit:

- **Classify the task** into a clear execution profile.
- **Keep simple tasks lightweight** with cheaper routing.
- **Raise effort for risky work** such as debugging, migrations, security, and architecture changes.
- **Keep command generation inspectable** before execution.
- **Move model and reasoning choices into policy files** instead of hard-coding them into shell habits.

## Capabilities

Routis combines routing, context control, policy, explainability, and local records into one workflow layer.

| Capability | What it does |
|---|---|
| Adaptive routing | Selects a fitting execution profile for the task |
| Routing IR | Classifies English prompts by intent, area, scope, risk, confidence, and target hints |
| Context control | Summarizes branch, changed files, manifests, docs, tests, workflows, and instruction files |
| Risk detection | Recognizes sensitive zones such as config, auth, schema, workflow, and package files |
| Policy control | Applies local routing rules and project-specific overrides |
| Dry run | Shows the route and command preview before execution |
| Explain mode | Shows a compact Decision Trace tree for the selected route |
| Sessions | Keeps continuity across related tasks |
| Traces | Records routing decisions as local JSONL audit artifacts |
| Token economy | Reduces unnecessary context, reasoning depth, and repeated work |

## Profiles

| Profile | Typical use |
|---|---|
| `cheap` | Typos, formatting, comments, small documentation edits |
| `balanced` | Ordinary implementation, tests, focused refactors |
| `deep` | Debugging, migrations, edge cases, security-sensitive work |
| `extradeep` | Redesigns, rewrites, architecture-level changes |
| `default` | Automatic selection from task wording |

## Installation

Build from source:

```bash
git clone https://github.com/alenisaw/routis.git
cd routis
cargo build --release
```

Run the compiled binary:

```bash
./target/release/routis --help
```

Install locally from the repository:

```bash
cargo install --path .
```

After local installation, the binary is available as `routis`:

```bash
routis
```

Running `routis` without a task opens the interactive TUI console. Cargo installs the binary into Cargo's bin directory, usually `~/.cargo/bin` on Linux/macOS and `%USERPROFILE%\.cargo\bin` on Windows. Make sure that directory is on `PATH`.

## Usage

Open the interactive terminal shell:

```bash
routis
```

The TUI starts when no task is passed. It provides:

- a responsive dashboard with provider, model, reasoning, metrics, updates, and recent sessions;
- a timeline for command results and local execution previews;
- a slash command palette;
- searchable session selection;
- inline theme and provider pickers;
- provider diagnostics for the local Codex CLI.

Useful TUI keys:

| Key | Action |
|---|---|
| `/` | Open the command palette |
| `Enter` | Submit input or confirm the selected item |
| `Esc` | Close the current palette, picker, or session view |
| `Ctrl+C` | Cancel the current task or clear input |
| `Ctrl+D` | Exit Routis |
| `F1` | Toggle keyboard shortcuts |

Useful TUI commands:

| Command | What it does |
|---|---|
| `/help` | Show keyboard shortcuts |
| `/status` | Show provider, model, reasoning, and theme |
| `/setup` | Open the local setup flow |
| `/doctor` | Check Codex CLI availability, version, auth status, and config path |
| `/provider` | Open the provider picker and diagnostics |
| `/theme` | Open the theme picker with live preview |
| `/sessions` | Open searchable recent-session selection |
| `/history` | Show local history status |
| `/context` | Show branch, changed files, area, and repo map markers |
| `/route <task>` | Preview the selected route without executing |
| `/policy-file <path>` | Set the routing policy file for this shell |
| `/clear` | Clear the current TUI timeline |
| `/config` | Show the local config path |
| `/quit` | Exit Routis |

Launch the TUI:

```bash
routis
```

Enter a task in the input row. Routis plans locally, shows the prompt, provider, model and reasoning, selected area, branch, changed file count, and confidence, then waits for `proceed`, `edit`, or `cancel`.

The TUI stores Routis runtime files next to the installed `routis` executable under `.routis`: config in `.routis/config.toml`, prompt history in `.routis/shell_history`, route sessions in `.routis/sessions`, and default policies in `.routis/policies`. Set `ROUTIS_HOME` to override this location. Provider diagnostics locate `codex` from the system PATH and run `codex --version`; on Windows, Routis prefers executable shims such as `.cmd` or `.exe` over blocked PowerShell scripts.

Decision traces are stored as JSONL files under `.routis/traces`. The trace stores a task hash instead of raw task text by default.

## Command Reference

```text
routis

Options:
  -h, --help                Print help
  -V, --version             Print version

Commands:
  route <task>              Preview the selected route without opening the TUI
    --explain               Print and store the decision trace tree
  context                   Print the repository context summary
  traces                    Print recent decision trace summaries
    --latest                Print the latest full decision trace tree
```

## Policy File

Default policy file: the install-local `.routis/policies/default.yaml`.

```yaml
version: 1

profiles:
  cheap:
    model: gpt-5.4-mini
    reasoning: low

  balanced:
    model: gpt-5.5
    reasoning: medium

  deep:
    model: gpt-5.5
    reasoning: high

  extradeep:
    model: gpt-5.5
    reasoning: xhigh

rules:
  - if_risk_zone: auth
    min_profile: deep
  - if_risk_zone: schema
    min_profile: deep
  - if_risk_zone: workflow
    min_profile: deep
  - if_risk_zone: package
    min_profile: deep
  - if_path: README.md
    max_profile: cheap
```

Policy rules apply to the automatic TUI route planner. A rule must define `if_risk_zone` or `if_path`; empty matchers are rejected.

## Development

Run checks:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Run locally:

```bash
cargo run
```

## License

Routis is released under the Apache-2.0 License. See [LICENSE](LICENSE).
