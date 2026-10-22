import assert from "node:assert/strict";
import { createServer } from "node:http";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { checkInstalledRuntime } from "../installed-runtime.mjs";

async function availablePort() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

test("installed runtime requires authenticated REST and MCP and cleans up its child", async () => {
  const dir = mkdtempSync(join(tmpdir(), "alkemyst installed runtime "));
  try {
    const executable = process.execPath;
    const fixture = join(dir, "app.mjs");
    writeFileSync(
      fixture,
      `
      import {createServer} from 'node:http';
      const [port, mode] = process.argv.slice(2);
      if (mode === 'exit') process.exit(19);
      createServer((req, res) => {
        if (mode === 'silent') return;
        if (req.url === '/v1/health') {
          res.writeHead(mode === 'bad-health' ? 503 : 200).end('{}');
          return;
        }
        res.writeHead(mode === 'public' ? 200 : mode === 'bad-mcp' && req.url === '/mcp' ? 500 : 401);
        res.end('{}');
      }).listen(Number(port), '127.0.0.1');
    `,
    );
    for (const mode of ["protected", "public", "bad-mcp", "bad-health", "exit", "silent"]) {
      const port = await availablePort();
      const options = {
        executable,
        args: [fixture, String(port), mode],
        port,
        timeoutMs: mode === "silent" ? 250 : 3000,
        settleMs: 20,
      };
      if (mode === "protected") await checkInstalledRuntime(options);
      else
        await assert.rejects(
          checkInstalledRuntime(options),
          /unauthenticated|health|exited|deadline/i,
        );
      await assert.rejects(fetch(`http://127.0.0.1:${port}/v1/health`));
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a pre-existing listener cannot make an unrelated installed app appear healthy", async () => {
  const server = createServer((_req, res) => res.writeHead(401).end());
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = server.address().port;
  try {
    await assert.rejects(
      checkInstalledRuntime({ executable: process.execPath, port }),
      /already in use/i,
    );
    assert.equal((await fetch(`http://127.0.0.1:${port}/v1/health`)).status, 401);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});
