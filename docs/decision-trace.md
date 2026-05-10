# Decision Trace

Decision Trace is the v0.4.0 audit layer for Routis routing decisions.

It records why a route was selected without storing raw task text by default. The raw task is represented as a stable FNV-1a hash. Full prompts remain out of trace files unless a later explicit privacy mode enables them.

## Storage

Traces are stored as JSONL files:

```text
.routis/traces/<session_id>.jsonl
```

Each line is one `DecisionTrace` record.

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
provider_command
route_tree
```

## CLI behavior

`--explain` should print a compact tree:

```text
Routis Decision Trace
├─ Input Analysis
│  ├─ Language: english
│  ├─ Intent: debug
│  ├─ Area: auth
│  ├─ Scope: focused
│  ├─ Risk: high
│  └─ Confidence: high
├─ Matched Signals
│  ├─ intent: debug
│  └─ area: auth
├─ Repo Context
└─ Route Decision
   ├─ Requested profile: default
   ├─ Selected profile: deep
   ├─ Model: gpt-5.5
   └─ Reasoning: high
```

## Commands

```text
routis traces
routis traces --latest
```

`traces` prints recent trace summaries. `traces --latest` prints the last full explain tree.

## Privacy rule

Default trace files must not include raw task text, raw source code, `.env` values, private keys, tokens, or provider output.
