import { useState } from "react";
import { Button } from "./button.js";
export function usePages() {
  const [cursors, setCursors] = useState<(string | undefined)[]>([undefined]);
  return {
    cursor: cursors[cursors.length - 1],
    number: cursors.length,
    previous: () => setCursors((c) => c.slice(0, Math.max(1, c.length - 1))),
    next: (cursor: string) => setCursors((c) => [...c, cursor]),
    reset: () => setCursors([undefined]),
  };
}
export function Pagination({
  pages,
  next,
  busy,
}: {
  pages: ReturnType<typeof usePages>;
  next?: string | null;
  busy: boolean;
}) {
  return (
    <nav className="pagination" aria-label="Result pages">
      <Button color="secondary" isDisabled={pages.number === 1 || busy} onPress={pages.previous}>
        Previous
      </Button>
      <span>Page {pages.number}</span>
      <Button
        color="secondary"
        isDisabled={!next || busy}
        onPress={() => {
          if (next) pages.next(next);
        }}
      >
        Next
      </Button>
    </nav>
  );
}
