import { createContext, useContext } from "react";
import { QueryClient } from "@tanstack/react-query";
import { createAlkemystClient, createAlkemystQueryClient } from "@alkemyst/sdk";
import type { LocalConnection } from "./connection.js";
export function createServices(connection: LocalConnection, fetcher: typeof fetch = fetch) {
  return {
    connection,
    client: createAlkemystClient(connection.base_url, connection.token, fetcher),
    api: createAlkemystQueryClient(connection.base_url, connection.token, fetcher),
    queryClient: new QueryClient({
      defaultOptions: { queries: { staleTime: 15_000, retry: false }, mutations: { retry: false } },
    }),
  };
}
export type Services = ReturnType<typeof createServices>;
export const ServiceContext = createContext<Services | null>(null);
export function useServices(): Services {
  const services = useContext(ServiceContext);
  if (!services) throw new Error("Local service provider is missing");
  return services;
}
