import { expect, test } from "@playwright/test";

test("desktop foundation loads", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Alkemyst" })).toBeVisible();
});
