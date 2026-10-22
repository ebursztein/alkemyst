import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const desktop = fileURLToPath(new URL("../src/desktop", import.meta.url));
const tauriCli = fileURLToPath(
  new URL("../src/desktop/node_modules/@tauri-apps/cli/tauri.js", import.meta.url),
);

export function runNativeBuild({ cli = tauriCli, stdio = "inherit" } = {}) {
  // A pnpm script on Windows consumes the separator before Cargo's flags.
  // Pass the complete argv directly to the pinned public Tauri CLI entry point.
  return execFileSync(
    process.execPath,
    [cli, "build", "--debug", "--no-bundle", "--ci", "--", "--locked"],
    {
      cwd: desktop,
      encoding: "utf8",
      stdio,
    },
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) runNativeBuild();
