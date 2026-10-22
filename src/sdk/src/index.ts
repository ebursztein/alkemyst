import createClient from "openapi-fetch";
import createQueryClient from "openapi-react-query";
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

/** Typed TanStack Query helpers use the exact same authenticated generated client. */
export function createAlkemystQueryClient(
  baseUrl: string,
  token: string,
  fetcher: typeof fetch = fetch,
) {
  return createQueryClient(createAlkemystClient(baseUrl, token, fetcher));
}

export type { paths, components, operations } from "./generated/schema.js";
