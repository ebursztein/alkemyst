import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
const pages = [
  "/",
  "/download/",
  "/docs/",
  "/docs/setup/",
  "/docs/guides/tracking/",
  "/docs/guides/analysis/",
  "/docs/guides/mcp/",
  "/docs/guides/development/",
];
test("marketing leads to truthful preview status with keyboard access and accessible layouts", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Brew with evidence." })).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to content" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("#main-content")).toBeFocused();
  expect(
    (await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze())
      .violations,
  ).toEqual([]);
  await page.getByRole("link", { name: "Preview status", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Download previews" })).toBeVisible();
  await expect(
    page.getByText("Preview installers are not published yet.", { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "View GitHub Releases" })).toHaveAttribute(
    "href",
    "https://github.com/ebursztein/alkemyst/releases",
  );
  await expect(page.locator('a[href$=".dmg"], a[href$=".exe"], a[href$=".msi"]')).toHaveCount(0);
  expect(
    (await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze())
      .violations,
  ).toEqual([]);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
test("built documentation search finds analysis assumptions and remains accessible", async ({
  page,
}) => {
  await page.goto("/docs/");
  await page.getByRole("button", { name: /Search/ }).click();
  const search = page.getByRole("dialog");
  await search.getByRole("textbox").fill("legendary");
  await search.locator('a[href*="/docs/guides/analysis/"]').first().click();
  await expect(page.getByRole("heading", { level: 1, name: "Analysis assumptions" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Legendary duplicates" })).toBeVisible();
  expect(
    (await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze())
      .violations,
  ).toEqual([]);
});
test("every published page and its local link destinations resolve", async ({ page }) => {
  const checked = new Set<string>();
  for (const path of pages) {
    const response = await page.goto(path);
    expect(response?.status(), path).toBe(200);
    const origin = new URL(page.url()).origin;
    const links = await page
      .locator("a[href]")
      .evaluateAll((anchors) => anchors.map((anchor) => (anchor as HTMLAnchorElement).href));
    for (const href of links) {
      const url = new URL(href);
      if (url.origin !== origin || checked.has(url.pathname)) continue;
      checked.add(url.pathname);
      expect((await page.request.get(url.pathname)).status(), url.pathname).toBe(200);
    }
  }
});
