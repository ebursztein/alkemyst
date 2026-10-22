import { expect, test } from "@playwright/test";

test("a browser without native bootstrap explains how to connect and can retry", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Alkemyst" })).toBeVisible();
  await expect(
    page.getByText("Open Alkemyst desktop to connect to your local library."),
  ).toBeVisible();
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByRole("alert")).toContainText("Open Alkemyst desktop");
  expect(errors).toEqual([]);
});
