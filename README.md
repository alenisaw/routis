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
  | <a href="#before--after">Before / After</a>
  | <a href="#run-routis">Run Routis</a>
  | <a href="#how-it-works">How it works</a>
  | <a href="#decision-trace">Decision Trace</a>
  | <a href="#policy">Policy</a>
</p>

---

Routis is a local routing and audit layer for AI coding workflows.

It adds a deliberate routing step before execution. Given a task, Routis classifies the request, reads repository signals, applies local policy, selects an execution profile, and explains why that route was chosen.

The point is simple: AI-assisted development should not use the same execution path for every task. Small edits should stay lightweight. Risky changes should receive stronger handling. Model choice, reasoning depth, and command shape should be explicit instead of hidden in repeated shell habits.

## Why Routis

AI coding tasks do not all deserve the same execution depth. A typo fix, a focused implementation task, and a risky debugging session should not be routed through identical model and reasoning settings.

Routis makes that decision explicit:

- **Classify the task** into a clear execution profile.
- **Read repository context** such as branch, changed files, manifests, tests, workflows, and sensitive areas.
- **Apply local policy** so model and reasoning choices stay reviewable.
- **Explain the route** with a compact Decision Trace.
- **Keep local state privacy-aware** by avoiding raw task text in traces and sessions by default.

## Before / After

Without Routis, developers often choose model, reasoning depth, and command shape manually for every AI coding task. Small and risky changes can end up using the same execution path because the routing decision lives in memory, habit, or ad-hoc shell commands.

With Routis, the routing decision becomes a local, inspectable step before the AI agent runs.

| Task | Without Routis | With Routis |
|---|---|---|
| `fix typo in README` | Manually decide whether the task deserves a full agent run | `cheap / low` |
| `add parser tests` | Guess the required reasoning depth | `balanced / medium` |
| `debug auth flow` | Remember to raise effort for sensitive code | `deep / high` |
| `redesign routing architecture` | Manually switch to the strongest execution setup | `extradeep / xhigh` |

Example route:

```bash
routis route --explain "debug auth flow"
```

```text
selected: deep / gpt-5.5 / high
intent: debug
area: auth
risk: high

Routis Decision Trace
├─ Input Analysis
│  ├─ Intent: debug
│  ├─ Area: auth
│  └─ Risk: high
└─ Route Decision
   ├─ Selected profile: deep
   ├─ Model: gpt-5.5
   └─ Reasoning: high
```

Instead of relying on shell habits, Routis gives the route, the reason, and the selected execution profile before work begins.

## Run Routis

Install from crates.io:

```bash
cargo install routis
```

Install from source:

```bash
git clone https://github.com/alenisaw/routis.git
cd routis
cargo install --path .
```

Open the interactive terminal UI:

```bash
routis
```

Preview a route from the CLI:

```bash
routis route "debug auth flow"
```

Print the route explanation tree:

```bash
routis route --explain "debug auth flow"
```

Inspect recent local traces:

```bash
routis traces
routis traces --latest
```

Cargo installs the binary into Cargo's bin directory, usually `~/.cargo/bin` on Linux/macOS and `%USERPROFILE%\.cargo\bin` on Windows. Make sure that directory is on `PATH`.

## How it works

Routis combines task classification, repository context, local policy, and execution profiles.

| Step | What happens |
|---|---|
| Classify | Detects intent, area, scope, risk, confidence, and target hints |
| Inspect | Reads branch, changed files, manifests, docs, tests, workflows, and risk zones |
| Apply policy | Uses local rules to raise or cap the selected profile |
| Select route | Chooses profile, model, and reasoning depth |
| Explain | Builds a local Decision Trace for the route |

Profiles are intentionally simple:

| Profile | Typical use |
|---|---|
| `cheap` | Typos, formatting, comments, small documentation edits |
| `balanced` | Ordinary implementation, tests, focused refactors |
| `deep` | Debugging, migrations, edge cases, security-sensitive work |
| `extradeep` | Redesigns, rewrites, architecture-level changes |
| `default` | Automatic selection from task wording and repository context |

## Decision Trace

Routis can store routing decisions as local JSONL traces under:

```text
~/.routis/traces/<session_id>.jsonl
```

Decision Trace is designed for local auditability: it records the selected route, matched signals, repository facts, policy source, selected model, and reasoning level.

By default, Routis avoids raw task text in trace and session records. CLI traces store an HMAC-SHA256 task hash using a per-install local secret at:

```text
~/.routis/secret
```

Runtime files live under `~/.routis` by default. Set `ROUTIS_HOME` to override this location.

## Policy

Routis behavior is controlled through local policy.

Default policy path:

```text
~/.routis/policies/default.yaml
```

| Policy control | Example |
|---|---|
| Profile model | `cheap → gpt-5.4-mini` |
| Reasoning depth | `deep → high` |
| Risk escalation | `auth → at least deep` |
| Path cap | `README.md → at most cheap` |

Policy rules keep routing behavior local, reviewable, and project-specific. The default policy can be replaced with a custom YAML file when a repository needs different routing rules.

## License

Routis is released under the Apache-2.0 License. See [LICENSE](LICENSE).
