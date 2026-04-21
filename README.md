<p align="center">
  <img src="https://em-content.zobj.net/source/apple/391/compass_1f9ed.png" width="120" />
</p>

<h1 align="center">Routis</h1>

<p align="center">
  <strong>adaptive client and execution policy layer for Codex</strong>
</p>

<p align="center">
  <a href="https://github.com/alenisaw/routis/stargazers"><img src="https://img.shields.io/github/stars/alenisaw/routis?style=flat&color=yellow" alt="Stars"></a>
  <a href="https://github.com/alenisaw/routis/commits/main"><img src="https://img.shields.io/github/last-commit/alenisaw/routis?style=flat" alt="Last Commit"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/alenisaw/routis?style=flat" alt="License"></a>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> &bull;
  <a href="#install">Install</a> &bull;
  <a href="#run">Run</a> &bull;
  <a href="#verify">Verify</a> &bull;
  <a href="#profiles">Profiles</a> &bull;
  <a href="#routing">Routing</a>
</p>

---

Routis is an adaptive client and execution policy layer for Codex. It routes tasks into the right execution depth, reduces unnecessary token burn, keeps responses lean, and stays explainable.

Right now it is intentionally narrow:

- CLI-first
- policy-driven
- rule-based
- explainable
- dry-run first
- thin Codex adapter

No desktop app, no telemetry, and no app-server integration.

## Quickstart

```bash
npm install -g @alenisaw/routis
routis --policy default "redesign routing architecture across files"
```

Example output:

```text
Requested policy: default
Effective profile: extradeep
Model: gpt-5.4
Reasoning: xhigh
Selected route: ExtraDeep
Why: Matched extradeep signal: "redesign routing".
Execution mode: dry-run
Codex command: set "CODEX_HOME=..." && codex.cmd exec -m gpt-5.4 -c "reasoning_effort=\"xhigh\"" -C ...
```

## What It Does

- accepts a task from the CLI
- supports `--policy default|cheap|balanced|deep|extradeep`
- supports `--dry-run` and `--execute`
- supports `--help` and `--version`
- loads policy presets from YAML
- uses a rule-based classifier to recommend an effective profile
- auto-selects an effective profile when `default` is requested
- prepares a Codex execution plan from the selected effective profile
- generates the exact Codex CLI command Routis would run
- can optionally execute Codex CLI through a thin adapter

## Install

Requirements:

- Node.js 20+
- npm

Install from npm:

```bash
npm install -g @alenisaw/routis
```

Check the installation:

```bash
routis --version
routis --help
```

For local development from the repository:

```bash
npm install
npm run install:local
```

That builds the project and runs `npm link`, so `routis` is available in your shell.

On Windows, the install step also removes the blocked `routis.ps1` shim so PowerShell resolves bare `routis` directly to the linked command launcher.

If you prefer to keep the generated PowerShell shim enabled for the current session instead, run:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

Remove the local development link later with:

```bash
npm run uninstall:local
```

## Run

Basic run:

```bash
routis "fix typo in readme"
```

Explicit profile:

```bash
routis --policy deep "debug tricky edge case in validation flow"
```

Dynamic default:

```bash
routis --policy default "redesign routing architecture across files"
```

Execute mode:

```bash
routis --policy balanced --execute "implement CLI flag to load config"
```

Help:

```bash
routis --help
routis --version
```

Unknown profile handling:

```bash
routis --policy unknown "fix typo"
```

If you prefer not to link it globally, you can still run the development entrypoint:

```bash
npm run cli -- --policy default "review the current task summary"
```

## Verify

Smoke test:

```bash
npm test
```

Build:

```bash
npm run build
```

Release candidate check:

```bash
npm run verify:release
```

## Profiles

| Profile | Model | Reasoning | Behavior |
|--------|-------|-----------|----------|
| `cheap` | `gpt-5.4-mini` | `none` | fixed |
| `balanced` | `gpt-5.4` | `medium` | fixed |
| `deep` | `gpt-5.4` | `high` | fixed |
| `extradeep` | `gpt-5.4` | `xhigh` | fixed |
| `default` | `dynamic` | `dynamic` | classifier chooses the effective profile |

## Routing

The classifier is rule-based and meant to stay readable.

It looks at:

- cheap signals like `typo`, `rename`, `format`, `small edit`
- balanced signals like `implement`, `update`, `review`, `load config`
- deep signals like `debug`, `architecture`, `migration`, `security`
- extradeep signals like `redesign routing`, `major architecture change`, `large refactor across layers`
- upgrade and downgrade cues that can move the result one level up or down

It is not trying to be clever. It is trying to be easy to inspect and easy to tune.

## Repository Layout

```text
routis/
  apps/cli/                CLI entrypoint
  bin/                     linked CLI wrapper
  configs/policies/        YAML policy presets and shared routing rules
  packages/codex-adapter/  thin Codex command planning and execution
  packages/core/           classification and route selection
  packages/policy/         policy types and loading
  tests/                   smoke-test script
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache License 2.0. See [LICENSE](LICENSE).
