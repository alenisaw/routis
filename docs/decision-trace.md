# Decision Trace

Decision Trace is the v0.4.0 audit layer for Routis routing decisions.

It records why a route was selected without storing raw task text by default. The raw task is represented as an HMAC-SHA256 hash using a per-install secret stored under Routis home. Full prompts remain out of trace files unless a later explicit privacy mode enables them.

## Storage

Traces are stored as JSONL files:

```text
~/.routis/traces/<session_id>.jsonl
```

Each line is one `DecisionTrace` record.

`ROUTIS_HOME` overrides the default `~/.routis` location. If the user home directory cannot be resolved, Routis returns `could not determine user home; set ROUTIS_HOME` instead of writing runtime state beside the executable or current directory. On Unix, Routis creates sensitive local directories with `0700` permissions and trace/secret/session/config files with `0600` permissions. On Windows, strict ACL hardening is not implemented in this pass; Routis uses normal filesystem creation and relies on the user's profile ACLs.

## Schema

Core fields:

```text
schema_version
session_id
task_hash
timestamp_unix_ms
language
intent
area
scope
risk
confidence
matched_signals
risk_zones
repo_facts
requested_profile
selected_profile
selected_model
selected_reasoning
prompt_mode
execution_mode
policy_source
policy_overrides
provider_command_preview
route_tree
```

## CLI behavior

`routis route` stores a local trace by default. `--explain` additionally prints a compact tree:

```text
Routis Decision Trace
├─ Input Analysis
│  ├─ Language: english
│  ├─ Intent: debug
│  ├─ Area: auth
│  ├─ Scope: focused
│  ├─ Risk: high
│  ├─ Confidence: high
│  └─ Target: -
├─ Matched Signals
│  ├─ intent: debug
│  └─ area: auth
├─ Repo Context
│  ├─ Policy source: embedded default policy
│  └─ risk-zone: auth
└─ Route Decision
   ├─ Requested profile: default
   ├─ Selected profile: deep
   ├─ Model: gpt-5.5
   ├─ Reasoning: high
   ├─ Prompt mode: raw
   ├─ Execution mode: preview
   └─ Provider command preview: codex ["exec", "-m", "gpt-5.5", "--reasoning", "high", "--", "<task-redacted>"]
```

## Commands

```text
routis traces
routis traces --latest
```

`traces` prints recent trace summaries. `traces --latest` prints the last full explain tree.

## Privacy rule

Default trace and session files must not include raw task text, raw source code, `.env` values, private keys, tokens, or provider output.
