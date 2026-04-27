# Contributing to Routis

We welcome contributions that improve routing quality, context control, policy behavior, documentation, tests, and developer experience.

We want contributions to stay focused, reviewable, and easy to reason about.

## Before you start

We prefer small pull requests with a clear purpose.

Please open an issue first for changes that affect:

- routing behavior
- CLI flags or command semantics
- interactive commands
- policy schema
- trace or session formats
- dependency choices
- storage paths
- CI or release behavior
- broad UI or architecture changes

Small documentation fixes, typo fixes, focused tests, and clearly scoped bug fixes can go directly to a pull request.

> [!TIP]
> If a change affects routing behavior, explain which tasks will behave differently after the change.

## Development setup

Requirements:

- Rust stable
- Cargo
- Git

Clone the repository:

```bash
git clone https://github.com/alenisaw/routis.git
cd routis
```

Build:

```bash
cargo build
```

Run checks:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Run locally:

```bash
cargo run -- "fix typo in README"
cargo run -- --explain "debug failing config loader"
cargo run -- tui
```

## Pull requests

We review pull requests faster when they are small, intentional, and easy to review.

A good pull request usually:

- solves one problem
- explains the user-facing behavior
- includes tests for changed behavior
- updates documentation when needed
- avoids unrelated formatting churn
- keeps routing behavior deterministic
- keeps persistent formats versioned

We prefer not to mix refactors with behavior changes. Split them when possible.

## Commit style

We use Conventional Commits.

Examples:

```text
feat: add policy file loading
fix: handle empty task input
docs: tighten README overview
test: cover deep routing signals
refactor: split trace writer
chore: update CI checks
```

Keep the subject short and specific. Use the body when the change needs context.

## AI-assisted contributions

We accept contributions made with AI assistance when the final work is reviewed and owned by the contributor.

If AI tools were used substantially, mention that in the pull request description. Every generated line should be checked for correctness, relevance, license compatibility, and project fit.

We strongly prefer smaller, reviewed AI-assisted pull requests over large unreviewed dumps.

## Routing behavior changes

Routing changes require extra care because small rule changes can affect many tasks.

When changing routing logic, include:

- examples of tasks affected by the change
- tests for the new behavior
- a note about changed profiles or risk signals
- updates to documentation if user-facing behavior changes

> [!IMPORTANT]
> Routing should remain explainable. If a decision cannot be explained clearly, the implementation is not ready.

## Persistent formats

Trace files, sessions, policies, and config files are persistent formats.

When changing them:

- keep a schema version
- avoid silent breaking changes
- document new fields
- preserve compatibility where practical
- add tests for read/write behavior

## Code style

We keep code explicit and boring where possible.

Prefer:

- typed errors over stringly-typed failures
- deterministic rules over hidden behavior
- clear data structures over clever abstractions
- small modules with narrow responsibility
- tests that describe real routing cases

Avoid:

- `unwrap()` in library code
- hidden network calls
- storing raw task text in traces
- broad dependency additions without discussion
- formatting-only churn inside behavior PRs

## Documentation

We value documentation that explains behavior directly.

Good docs should answer:

- what the command does
- when to use it
- what output to expect
- where data is stored
- what decisions Routis makes automatically

## Reporting issues

When reporting a bug, include:

- operating system
- Routis version or commit hash
- command used
- expected behavior
- actual behavior
- relevant output or trace summary

Do not include private repository content, secrets, tokens, or sensitive task text.

## Security

Please do not report security issues through public issues.

Use the repository security policy if available. If there is no published security policy yet, contact the maintainer privately before disclosing details.
