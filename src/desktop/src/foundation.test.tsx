// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";
import { Foundation } from "./foundation.js";

it("exposes a named main heading", () => {
  render(<Foundation />);
  expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Alkemyst");
  expect(screen.getByRole("main")).toBeTruthy();
});
