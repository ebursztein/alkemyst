import { execFile, spawn } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { promisify } from "node:util";
import { gzipSync } from "node:zlib";

const execute = promisify(execFile);
const token = "test-only-local-api-credential-with-at-least-32-characters";

export async function startLocalService({
  database,
  coverage = false,
  history = false,
  analysis = false,
  deferCatalog = false,
  failFirstCatalog = false,
} = {}) {
  const directory = database ? undefined : await mkdtemp(join(tmpdir(), "alkemyst-sdk-"));
  const path = database ?? join(directory, "catalog.sqlite3");
  let child;
  let pid;
  let deferredArchive;
  try {
    if (directory) {
      const archive = join(directory, "cards.jsonl.gz");
      const fixture = analysis
        ? JSON.parse(await readFile("src/core/tests/fixtures/boros-equipment.json", "utf8"))
            .cards.map(({ card }) =>
              JSON.stringify({
                ...card,
                id: `test-${card.oracle_id}`,
                object: "card",
                lang: "en",
                released_at: "2026-01-01",
                color_identity: card.colors ?? [],
              }),
            )
            .join("\n")
        : await readFile("src/core/tests/fixtures/cards.jsonl");
      await writeFile(archive, gzipSync(fixture));
      if (deferCatalog) deferredArchive = archive;
      else {
        await execute(
          "cargo",
          [
            "run",
            "--quiet",
            "--locked",
            "--example",
            "catalog",
            "--",
            "import",
            path,
            archive,
            "https://data.scryfall.io/default-cards/fixture.jsonl.gz",
            "2026-10-21T21:00:00Z",
          ],
          { timeout: 180_000 },
        );
      }
    }
    if (history) {
      await execute(
        "cargo",
        [
          "run",
          "--quiet",
          "--locked",
          "--example",
          "capture",
          "--",
          "import",
          path,
          "src/core/tests/fixtures/arena-game.txt",
        ],
        { timeout: 180_000 },
      );
    }
    const args = coverage
      ? ["llvm-cov", "run", "--no-report", "--locked"]
      : ["run", "--quiet", "--locked"];
    child = spawn("cargo", [...args, "--example", "local_service", "--", path, "0"], {
      env: {
        ...process.env,
        ALKEMYST_TOKEN: token,
        ALKEMYST_FIXTURE_CATALOG: deferredArchive ?? "",
        ALKEMYST_FIXTURE_FAIL_FIRST: deferredArchive && failFirstCatalog ? "1" : "0",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let diagnostics = "";
    child.stderr.on("data", (chunk) => {
      diagnostics = (diagnostics + chunk.toString()).slice(-16_000);
    });
    const info = await new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error(`Local service startup timeout: ${diagnostics}`)),
        180_000,
      );
      const fail = (error) => {
        clearTimeout(timer);
        reject(error);
      };
      child.once("error", fail);
      child.once("exit", () =>
        fail(new Error(`Local service exited during startup: ${diagnostics}`)),
      );
      createInterface({ input: child.stdout }).once("line", (line) => {
        try {
          const info = JSON.parse(line);
          pid = info.pid;
          clearTimeout(timer);
          resolve(info);
        } catch (error) {
          fail(error);
        }
      });
    });
    return {
      baseUrl: info.base_url,
      token,
      async stop() {
        const exited = new Promise((resolve) => child.once("exit", resolve));
        process.kill(pid, "SIGINT");
        let timer;
        try {
          await Promise.race([
            exited,
            new Promise((_, reject) => {
              timer = setTimeout(() => {
                try {
                  process.kill(pid, "SIGKILL");
                } catch {
                  /* already stopped */
                }
                reject(new Error("Local service did not stop"));
              }, 10_000);
            }),
          ]);
        } finally {
          clearTimeout(timer);
          if (directory) await rm(directory, { recursive: true, force: true });
        }
      },
    };
  } catch (error) {
    if (pid) process.kill(pid, "SIGKILL");
    child?.kill();
    if (directory) await rm(directory, { recursive: true, force: true });
    throw error;
  }
}
