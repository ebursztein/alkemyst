import type { LocalService } from "./local-service.mjs";
export function startDesktopFixture(options?: {
  history?: boolean;
  coldCatalog?: boolean;
}): Promise<LocalService & { versionId: string; deckId: string }>;
