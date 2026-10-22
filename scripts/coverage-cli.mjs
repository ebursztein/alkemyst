import assert from "node:assert/strict";
import { startLocalService } from "./local-service.mjs";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
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
