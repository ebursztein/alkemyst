import AxeBuilder from "@axe-core/playwright";
import { test, expect } from "./fixture.js";

test("backup selection has accessible confirmation and keyboard cancellation", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/settings");
  await page.getByRole("button", { name: "Save backup…" }).click();
  await expect(page.getByText("Backup saved:")).toBeVisible();
  await page.getByRole("button", { name: "Restore backup…" }).click();
  const confirmation = page.getByRole("alertdialog", { name: "Restore from backup?" });
  await expect(confirmation).toBeVisible();
  await expect(page.getByRole("button", { name: "Keep current library" })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(page.getByRole("button", { name: "Restart and restore" })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("button", { name: "Keep current library" })).toBeFocused();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  const accessibility = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"])
    .analyze();
  expect(accessibility.violations).toEqual([]);
  await page.screenshot({ path: "work/backup-confirmation-dark-mobile.png" });
  await page.keyboard.press("Escape");
  await expect(confirmation).not.toBeVisible();
  await expect(page.getByRole("button", { name: "Restore backup…" })).toBeFocused();
  const before = await page.evaluate(
    () => Reflect.get(window, "__ALKEMYST_NATIVE_CALLS__") as string[],
  );
  expect(before).toContain("cancel_restore");
  expect(before).not.toContain("restore_and_restart");
  await page.getByRole("button", { name: "Restore backup…" }).click();
  await page.getByRole("button", { name: "Restart and restore" }).click();
  await expect(page.getByText("Restarting Alkemyst to restore your library…")).toBeVisible();
  const after = await page.evaluate(
    () => Reflect.get(window, "__ALKEMYST_NATIVE_CALLS__") as string[],
  );
  expect(after.filter((command) => command === "restore_and_restart")).toHaveLength(1);
});
