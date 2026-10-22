import { spawn, execFileSync } from "node:child_process";
import { createConnection } from "node:net";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";

function listenerExists(port) {
  return new Promise((resolve) => {
    const socket = createConnection({ host: "127.0.0.1", port });
    const finish = (open) => {
      socket.destroy();
      resolve(open);
    };
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
    socket.setTimeout(500, () => finish(false));
  });
}

async function status(port, path, method = "GET") {
  try {
    const response = await fetch(`http://127.0.0.1:${port}${path}`, {
      method,
      signal: AbortSignal.timeout(500),
    });
    await response.body?.cancel();
    return response.status;
  } catch {
    return null;
  }
}

export async function checkInstalledRuntime({
  executable,
  args = ["--background"],
  port = 47831,
  timeoutMs = 30000,
  settleMs = 1500,
}) {
  if (await listenerExists(port)) throw new Error("Installed runtime port is already in use");
  const child = spawn(executable, args, { stdio: "inherit" });
  let launchError;
  child.on("error", (error) => {
    launchError = error;
  });
  const assertRunning = () => {
    if (launchError) throw launchError;
    if (child.exitCode !== null || child.signalCode !== null)
      throw new Error(
        `Installed app exited before verification (${child.exitCode ?? child.signalCode})`,
      );
  };
  try {
    const deadline = Date.now() + timeoutMs;
    for (;;) {
      assertRunning();
      const health = await status(port, "/v1/health");
      if (health !== null) {
        if (health !== 200) throw new Error(`Installed build-health request failed (${health})`);
        const decks = await status(port, "/v1/decks");
        if (decks !== 401)
          throw new Error(`Installed deck data accepted an unauthenticated request (${decks})`);
        const mcp = await status(port, "/mcp", "POST");
        if (mcp !== 401)
          throw new Error(`Installed MCP failed the unauthenticated-request check (${mcp})`);
        await delay(settleMs);
        assertRunning();
        break;
      }
      if (Date.now() >= deadline)
        throw new Error("Installed app did not start its local service before the deadline");
      await delay(100);
    }
  } finally {
    if (child.exitCode === null && child.signalCode === null && child.pid) {
      const exited = new Promise((resolve) => child.once("exit", resolve));
      if (process.platform === "win32")
        execFileSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      else child.kill("SIGTERM");
      await Promise.race([exited, delay(5000)]);
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
    }
  }
  if (await listenerExists(port))
    throw new Error("Installed runtime left its local listener running after process cleanup");
  console.info(
    "Installed runtime started; REST and MCP enforce authentication. Process cleanup passed.",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const executable = process.argv[2];
  if (!executable) throw new Error("Pass the installed application executable");
  await checkInstalledRuntime({ executable });
}
