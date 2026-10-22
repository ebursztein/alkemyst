import { expect, test } from "@playwright/test";

test("static delivery preserves real 404s, headers and source provenance", async ({ request }) => {
  const home = await request.get("/");
  expect(home.status()).toBe(200);
  expect(home.headers()["x-content-type-options"]).toBe("nosniff");
  expect(home.headers()["x-frame-options"]).toBe("DENY");
  for (const path of ["/missing-page-for-delivery-test/", "/mcp", "/v1/decks"]) {
    expect((await request.get(path)).status(), path).toBe(404);
  }
  const manifestResponse = await request.get("/deployment.json");
  expect(manifestResponse.status()).toBe(200);
  const manifest = await manifestResponse.json();
  expect(manifest.commit).toMatch(/^[a-f0-9]{40}$/);
  expect(manifest.tree).toMatch(/^[a-f0-9]{40}$/);
  expect(manifest.origin).toBe(process.env.SITE_URL);
  const expected = process.env.ALKEMYST_EXPECTED_SITE_COMMIT ?? process.env.GITHUB_SHA;
  if (expected) expect(manifest.commit).toBe(expected);
});
