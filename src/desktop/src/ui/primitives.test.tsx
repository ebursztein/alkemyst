// @vitest-environment jsdom
import { cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { Pagination, usePages } from "./pagination.js";
import { Empty, errorMessage, Failure, Loading } from "./states.js";
import { Field } from "./field.js";
import { ManaCurve } from "../decks/curve.js";
import { useRequestKey } from "../app/request-key.js";
import { cardImage, metadataText } from "../decks/card-data.js";
afterEach(cleanup);
it("keeps failed-request keys stable and creates new keys only for changed bodies or success", () => {
  const { result, rerender } = renderHook(useRequestKey);
  const first = result.current.forBody({ version: "one" });
  rerender();
  expect(result.current.forBody({ version: "one" })).toBe(first);
  expect(result.current.forBody({ version: "two" })).not.toBe(first);
  result.current.clear();
  expect(result.current.forBody({ version: "one" })).not.toBe(first);
});
it("pages with opaque cursors and prevents navigation past the first page", () => {
  function Pages() {
    const pages = usePages();
    return (
      <>
        <span data-testid="cursor">{pages.cursor ?? "first"}</span>
        <Pagination pages={pages} next={pages.number < 2 ? "opaque-cursor" : null} busy={false} />
        <button onClick={pages.reset}>Reset</button>
      </>
    );
  }
  render(<Pages />);
  fireEvent.click(screen.getByRole("button", { name: "Next" }));
  expect(screen.getByTestId("cursor").textContent).toBe("opaque-cursor");
  expect((screen.getByRole("button", { name: "Next" }) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Previous" }));
  expect(screen.getByTestId("cursor").textContent).toBe("first");
  fireEvent.click(screen.getByRole("button", { name: "Next" }));
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  expect(screen.getByText("Page 1")).toBeTruthy();
});
it("labels unknown curve values, empty curves and recoverable errors", () => {
  const { rerender } = render(
    <ManaCurve
      bins={[
        { mana_value: null, quantity: 2 },
        { mana_value: 1, quantity: 0 },
      ]}
    />,
  );
  expect(screen.getByText("Unknown")).toBeTruthy();
  rerender(<ManaCurve bins={[]} />);
  expect(screen.getByText("No nonland cards.")).toBeTruthy();
  rerender(
    <>
      <Empty title="Empty">No records.</Empty>
      <Loading />
      <Failure error={{ message: "Local failure" }} />
      <Field label="Seed" hint="Saved locally" />
    </>,
  );
  expect(screen.getByRole("alert").textContent).toBe("Local failure");
  expect(screen.getByLabelText("Seed")).toBeTruthy();
  expect(errorMessage(undefined)).toContain("local service");
});
it("only displays HTTPS Scryfall artwork and preserves missing metadata", () => {
  const image = "https://cards.scryfall.io/normal/a.jpg";
  expect(cardImage({ card_faces: [{ image_uris: { normal: image } }] })).toBe(image);
  for (const value of [
    null,
    {},
    { image_uris: { normal: "javascript:bad" } },
    { image_uris: { normal: "not a URL" } },
    { image_uris: { normal: "https://untrusted.example/card.jpg" } },
  ])
    expect(cardImage(value)).toBeUndefined();
  expect(metadataText({ name: "Cloud" }, "name")).toBe("Cloud");
  expect(metadataText({ name: 1 }, "name")).toBeUndefined();
});
