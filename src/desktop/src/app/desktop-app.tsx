import { useState } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { ServiceContext, type Services } from "./services.js";
import { createDesktopRouter } from "./router.js";
export function DesktopApp({
  services,
  history,
}: {
  services: Services;
  history?: Parameters<typeof createDesktopRouter>[0];
}) {
  const [router] = useState(() => createDesktopRouter(history));
  return (
    <ServiceContext.Provider value={services}>
      <QueryClientProvider client={services.queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ServiceContext.Provider>
  );
}
