#!/usr/bin/env node

import { runCli } from "../dist/apps/cli/src/index.js";

runCli(process.argv.slice(2)).catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`Error: ${message}`);
  process.exitCode = 1;
});
