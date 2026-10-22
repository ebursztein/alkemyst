import { useEffect, useRef } from "react";
import { useServices } from "./services.js";

/** Observe database identity revisions through the generated SDK, on every screen. */
export function useCatalogStatus() {
  const { api, queryClient } = useServices();
  const catalog = api.useQuery("get", "/v1/catalog", undefined, { refetchInterval: 2000 });
  const stamp = catalog.data
    ? `${catalog.data.snapshot_id ?? "none"}:${catalog.data.arena_alias_revision}`
    : undefined;
  const previous = useRef<string | undefined>(undefined);
  useEffect(() => {
    if (stamp === undefined || stamp === previous.current) return;
    previous.current = stamp;
    // Background native imports invalidate the SQL cache themselves. Refresh active
    // application queries too, including previously unresolved card/profile reads.
    void queryClient.invalidateQueries({
      predicate: (query) => {
        if (String(query.queryKey[0]).startsWith("native-")) return false;
        return !(query.queryKey[0] === "get" && query.queryKey[1] === "/v1/catalog");
      },
    });
  }, [stamp, queryClient]);
  return catalog;
}
