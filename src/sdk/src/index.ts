import createClient from "openapi-fetch";
import type { paths } from "./generated/schema.js";

/** Each caller supplies the local service address and its scoped credential. */
export function createAlkemystClient(
  baseUrl: string,
  token: string,
  fetcher: typeof fetch = fetch,
) {
  return createClient<paths>({
    baseUrl,
    headers: { Authorization: `Bearer ${token}` },
    fetch: fetcher,
  });
}
