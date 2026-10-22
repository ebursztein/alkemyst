import assert from "node:assert/strict";
import { startLocalService } from "./local-service.mjs";
import { execFileSync, spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { gzipSync } from "node:zlib";

export async function coverRustCli() {
  const directory = mkdtempSync(join("work", "coverage-cli-"));
  const run = (example, args) =>
    JSON.parse(
      execFileSync(
        "cargo",
        ["llvm-cov", "run", "--no-report", "--locked", "--example", example, "--", ...args],
        { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
      ),
    );
  try {
    const schema = run("export_openapi", []);
    assert.equal(schema.openapi, "3.1.0");
    const capture = run("inspect_arena", ["src/core/tests/fixtures/arena-game.txt"]);
    assert.equal(capture.main_deck_versions, 3);
    assert.equal(capture.numbered_games, 1);
    const captureDatabase = join(directory, "capture.sqlite3");
    const imported = run("capture", [
      "import",
      captureDatabase,
      "src/core/tests/fixtures/arena-game.txt",
    ]);
    assert.equal(imported.games, 1);
    assert.equal(imported.deck_versions, 3);
    assert.equal(run("capture", ["status", captureDatabase]).events, imported.events);
    const watched = await coverCaptureWatch(captureDatabase, directory);
    assert.equal(watched.games, 1);
    assert.equal(watched.events, imported.events);
    const archive = join(directory, "cards.jsonl.gz");
    const database = join(directory, "cards.sqlite3");
    writeFileSync(archive, gzipSync(readFileSync("src/core/tests/fixtures/cards.jsonl")));
    const result = run("catalog", [
      "import",
      database,
      archive,
      "https://data.scryfall.io/default-cards/fixture.jsonl.gz",
      "2026-10-21T21:00:00Z",
    ]);
    assert.equal(result.cards, 3);
    assert.equal(run("catalog", ["status", database]).printings, 4);
    assert.equal(run("catalog", ["search", database, "equipment"]).items.length, 2);
    const resolution = run("catalog", ["resolve", database, "42", "999"]);
    assert.equal(resolution[0].name, "Forgeblade");
    assert.equal(resolution[1].name, null);
    const service = await startLocalService({ database, coverage: true });
    try {
      const response = await fetch(`${service.baseUrl}/v1/decks`, {
        headers: { Authorization: `Bearer ${service.token}` },
      });
      assert.equal(response.status, 200);
      assert.deepEqual((await response.json()).items, []);
    } finally {
      await service.stop();
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

async function coverCaptureWatch(database, directory) {
  const child = spawn(
    "cargo",
    [
      "llvm-cov",
      "run",
      "--no-report",
      "--locked",
      "--example",
      "capture",
      "--",
      "watch",
      database,
      "src/core/tests/fixtures/arena-game.txt",
      join(directory, "missing.log"),
    ],
    { stdio: ["ignore", "pipe", "inherit"] },
  );
  let pid;
  let result;
  let signaled = false;
  const exited = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (code) =>
      code === 0 ? resolve() : reject(new Error(`Capture process exited: ${code}`)),
    );
  });
  // Attach immediately so an early process error cannot become unhandled.
  const failure = exited.catch(() => {});
  const timer = setTimeout(() => {
    if (pid) {
      try {
        process.kill(pid, "SIGKILL");
      } catch {
        /* already stopped */
      }
    }
    child.kill();
  }, 60_000);
  try {
    for await (const line of createInterface({ input: child.stdout })) {
      const value = JSON.parse(line);
      if (value.pid) pid = value.pid;
      if (value.state === "tracking" && !signaled) {
        signaled = true;
        process.kill(pid, "SIGINT");
      }
      if (value.games !== undefined) result = value;
    }
    await exited;
    assert.equal(signaled, true, "capture must process retained history before shutdown");
    assert.ok(result, "capture must report final counts");
    return result;
  } finally {
    clearTimeout(timer);
    if (child.exitCode === null && child.signalCode === null) {
      if (pid) {
        try {
          process.kill(pid, "SIGKILL");
        } catch {
          /* already stopped */
        }
      }
      child.kill();
    }
    await failure;
  }
}
