# CLI App

Home for the Routis CLI.

The CLI accepts a task string, supports optional policy selection, resolves an effective profile, prepares a Codex execution plan, and can either print it or execute it.

Local development entrypoint:

```bash
npm run cli -- --policy default "review the current task summary"
```

Local shell command installation:

```bash
npm run install:local
routis --help
```
