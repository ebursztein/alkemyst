import AxeBuilder from "@axe-core/playwright";
import { test, expect } from "./fixture.js";
test("keyboard navigation, deck analysis, native API bootstrap and saved Monte Carlo run", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/decks");
  await expect(page.getByRole("link", { name: /Boros Forge/ })).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to content" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();
  await page.getByRole("link", { name: /Boros Forge/ }).click();
  await expect(page.getByRole("main")).toBeFocused();
  await expect(page.getByRole("figure", { name: "Printed mana curve", exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Analyze deck" }).click();
  await page.getByLabel("Trials", { exact: true }).fill("1000");
  await page.getByRole("button", { name: "Run simulation" }).click();
  await expect(page.getByText("Completed · 1,000 / 1,000 trials")).toBeVisible();
  await expect(
    page.getByRole("table", { name: "Simulation probabilities by turn" }).getByRole("row"),
  ).toHaveCount(9);
  const accessibility = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(accessibility.violations).toEqual([]);
  expect(errors).toEqual([]);
});
test("manual practice and system themes remain usable on a narrow window", async ({
  page,
  backend,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page.goto("/practice");
  await page.getByRole("combobox", { name: "Deck", exact: true }).selectOption(backend.versionId);
  await page.getByRole("button", { name: "Draw opening hand" }).click();
  await expect(page.getByRole("heading", { name: "Practice table" })).toBeVisible();
  await page.getByRole("button", { name: "Mulligan", exact: true }).click();
  await expect(page.getByText("Choose 1 to bottom · 0 selected")).toBeVisible();
  await page.getByRole("checkbox").first().check();
  await page.getByRole("button", { name: "Keep hand" }).click();
  await expect(page.getByRole("heading", { name: "Hand · 6" })).toBeVisible();
  await page.getByRole("button", { name: "Next turn" }).click();
  await expect(page.getByRole("heading", { name: "Hand · 7" })).toBeVisible();
  await page.getByRole("button", { name: "Finish practice" }).click();
  await expect(page.getByText("Practice session finished.")).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  const accessibility = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(accessibility.violations).toEqual([]);
});
