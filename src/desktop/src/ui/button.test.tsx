// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Button } from "./button.js";
afterEach(cleanup);
it("runs presses, preserves explicit form types and blocks pending or disabled actions", () => {
  const press = vi.fn();
  const { rerender } = render(<Button onPress={press}>Run simulation</Button>);
  fireEvent.click(screen.getByRole("button"));
  expect(press).toHaveBeenCalledOnce();
  expect(screen.getByRole("button").getAttribute("type")).toBe("button");
  rerender(
    <Button onPress={press} isLoading type="submit">
      Run simulation
    </Button>,
  );
  fireEvent.click(screen.getByRole("button"));
  expect(press).toHaveBeenCalledOnce();
  expect(screen.getByRole("button").getAttribute("data-loading")).toBe("true");
  expect((screen.getByRole("button") as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByRole("status").textContent).toBe("Working…");
  rerender(
    <Button onPress={press} isDisabled color="secondary">
      Run simulation
    </Button>,
  );
  fireEvent.click(screen.getByRole("button"));
  expect(press).toHaveBeenCalledOnce();
});
