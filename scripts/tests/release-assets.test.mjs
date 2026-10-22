import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { assembleReleaseAssets } from "../release-assets.mjs";

function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), "alkemyst release assets "));
  const source = { tag: "v0.1.0-preview.1", version: "0.1.0-preview.1", commit: "a".repeat(40) };
  const artifacts = join(root, "artifacts");
  const output = join(root, "output");
  const targets = ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"];
  const manifests = targets.map((target, index) => {
    const file = `Alkemyst_${source.version}_${index === 0 ? "aarch64.dmg" : "x64-setup.exe"}`;
    const dir = join(artifacts, `preview-${target}`);
    mkdirSync(dir, { recursive: true });
    const bytes = Buffer.from(`package fixture ${target}`);
    writeFileSync(join(dir, file), bytes);
    const manifest = {
      ...source,
      target,
      file,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      distribution: "preview",
      trusted_signing: false,
    };
    writeFileSync(join(dir, "manifest.json"), JSON.stringify(manifest));
    return {
      dir,
      manifest,
      save: () => writeFileSync(join(dir, "manifest.json"), JSON.stringify(manifest)),
    };
  });
  try {
    run({ source, artifacts, output, manifests });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("both platform artifacts become one source-bound release with verified checksums", () =>
  fixture(({ source, artifacts, output, manifests }) => {
    const release = assembleReleaseAssets({ source, artifacts, output });
    assert.equal(release.commit, source.commit);
    assert.equal(release.tag, source.tag);
    assert.equal(release.platforms.length, 2);
    for (const { manifest } of manifests) {
      assert.equal(
        createHash("sha256")
          .update(readFileSync(join(output, manifest.file)))
          .digest("hex"),
        manifest.sha256,
      );
      assert.ok(
        readFileSync(join(output, "SHA256SUMS"), "utf8").includes(
          `${manifest.sha256}  ${manifest.file}\n`,
        ),
      );
    }
    assert.deepEqual(JSON.parse(readFileSync(join(output, "manifest.json"), "utf8")), release);
    assert.match(readFileSync(join(output, "RELEASE_NOTES.md"), "utf8"), /ad-hoc|unsigned/);
    assert.match(readFileSync(join(output, "RELEASE_NOTES.md"), "utf8"), new RegExp(source.commit));
  }));

test("tampered, mismatched, missing or unsafe artifacts fail before publication", () => {
  const mutations = [
    ({ manifests }) =>
      writeFileSync(join(manifests[0].dir, manifests[0].manifest.file), "tampered"),
    ({ manifests }) => {
      manifests[0].manifest.commit = "b".repeat(40);
      manifests[0].save();
    },
    ({ manifests }) => {
      manifests[0].manifest.version = "0.1.0-preview.2";
      manifests[0].save();
    },
    ({ manifests }) => {
      manifests[0].manifest.file = "../escape.dmg";
      manifests[0].save();
    },
    ({ manifests }) => {
      manifests[0].manifest.trusted_signing = true;
      manifests[0].save();
    },
    ({ manifests }) => rmSync(manifests[0].dir, { recursive: true }),
  ];
  for (const mutate of mutations)
    fixture((context) => {
      mutate(context);
      assert.throws(() => assembleReleaseAssets(context), /artifact|manifest|package|signing/i);
    });
});

test("invalid input creates no output and stale release directories are never reused", () => {
  fixture((context) => {
    rmSync(context.manifests[1].dir, { recursive: true });
    assert.throws(() => assembleReleaseAssets(context), /artifact/i);
    assert.equal(existsSync(context.output), false);
  });
  fixture((context) => {
    mkdirSync(context.output);
    writeFileSync(join(context.output, "stale.exe"), "previous attempt");
    assert.throws(() => assembleReleaseAssets(context), /new directory|stale/i);
    assert.equal(readFileSync(join(context.output, "stale.exe"), "utf8"), "previous attempt");
  });
});
