import { execFile } from "node:child_process";
import { unlink } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

async function getGlobalBinDirectory() {
  if (process.platform === "win32" && process.env.APPDATA) {
    return path.join(process.env.APPDATA, "npm");
  }

  try {
    const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
    const { stdout } = await execFileAsync(npmCommand, ["prefix", "-g"], {
      windowsHide: true
    });
    const prefix = stdout.trim();

    if (!prefix) {
      return null;
    }

    return process.platform === "win32" ? prefix : path.join(prefix, "bin");
  } catch {
    return null;
  }
}

async function removePowerShellShim() {
  if (process.platform !== "win32") {
    console.log("Linked Routis CLI is ready.");
    return;
  }

  const globalBinDirectory = await getGlobalBinDirectory();

  if (!globalBinDirectory) {
    console.log("Linked Routis CLI is ready, but the PowerShell shim could not be located.");
    return;
  }

  const powershellShimPath = path.join(globalBinDirectory, "routis.ps1");

  try {
    await unlink(powershellShimPath);
    console.log(`Removed ${powershellShimPath} so PowerShell resolves "routis" to the linked command shim.`);
  } catch (error) {
    const code = error && typeof error === "object" && "code" in error ? error.code : undefined;

    if (code === "ENOENT") {
      console.log(`No PowerShell shim found at ${powershellShimPath}.`);
      return;
    }

    throw error;
  }

  console.log("Linked Routis CLI is ready.");
}

removePowerShellShim().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`Failed to finalize linked Routis CLI: ${message}`);
  process.exitCode = 1;
});
