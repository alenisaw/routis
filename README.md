# Routis

Routis is an early-stage adaptive execution layer for Codex.

The project is focused on a narrow problem: choosing a reasonable execution depth for a task, keeping token usage under control, and making the choice explainable.

Routis is CLI-first for now. Desktop support is only a later idea.

This repository is still in bootstrap stage. There is no working CLI yet.

## Why It Exists

Not every Codex task needs the same level of effort. Some requests can be handled quickly. Some need more depth. Some only need a short answer and a clear reason for the route that was chosen.

Routis exists to explore that layer in a small, inspectable codebase.

## What Is In The Repo Now

Right now the repository mostly contains:

-   project documentation
-   package and app placeholders
-   early policy preset files
-   a local `.agent/` workspace for planning and notes

These files describe intended structure more than implemented behavior.

## Next Milestone

The first meaningful milestone is a working CLI router that can:

-   load a policy preset
-   classify a task at a basic level
-   choose fast, balanced, or deep
-   return the chosen route with a short rationale

## Repository Structure

```text
routis/
  .agent/             Local planning and scratch space
  apps/
    cli/              Planned CLI app
    desktop/          Future desktop app
  configs/
    policies/         Early routing presets
  docs/               Project documentation
  packages/           Planned module boundaries
```

## License

Routis is licensed under the Apache License 2.0. See [LICENSE](LICENSE).