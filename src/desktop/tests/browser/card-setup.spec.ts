import AxeBuilder from "@axe-core/playwright";
import { createAlkemystClient } from "@alkemyst/sdk";
import { test, expect } from "./fixture.js";
test.use({ coldCatalog: true });

test("a cold library retries metadata acquisition and unlocks resolved deck analysis", async ({
  page,
  backend,
}) => {
  const client = createAlkemystClient(backend.baseUrl, backend.token);
  const before = await client.GET("/v1/capture");
  expect(before.data?.games).toBe(1);
  await page.goto("/decks");
  await page.getByRole("link", { name: "Set up card library" }).click();
  await page.getByRole("button", { name: "Download card metadata" }).click();
  await expect(page.getByRole("alert")).toContainText("Card refresh failed");
  expect((await client.GET("/v1/catalog")).data?.cards).toBe(0);
  // Keep intermediate color states observable across the refresh response.
  // Contrast must hold during a variant change, not only after animations end.
  await page.addStyleTag({ content: ".button { transition-duration: 10s !important; }" });
  await page.getByRole("button", { name: "Download card metadata" }).click();
  await expect(page.getByRole("button", { name: "Update card metadata" })).toBeVisible();
  expect((await client.GET("/v1/catalog")).data?.cards).toBeGreaterThan(20);
  expect((await client.GET("/v1/capture")).data).toEqual(before.data);
  const accessibility = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(accessibility.violations).toEqual([]);
  await page.screenshot({ path: "work/card-setup-settings.png" });
  await page.getByRole("link", { name: "Decks", exact: true }).click();
  await expect(page.getByRole("link", { name: "Set up card library" })).toHaveCount(0);
  await page.getByRole("link", { name: /Boros Forge/ }).click();
  await expect(page.getByRole("figure", { name: "Printed mana curve", exact: true })).toBeVisible();
  await expect(page.getByText("Unresolved card", { exact: true })).toHaveCount(0);
});
