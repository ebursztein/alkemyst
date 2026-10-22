import { invoke } from "@tauri-apps/api/core";

const states = ["disabled", "searching", "ready", "missing", "degraded", "stopped"] as const;
export interface CardCacheHealth {
  state: (typeof states)[number];
  last_error: string | null;
}
export async function readCardCacheHealth(): Promise<CardCacheHealth> {
  try {
    const value = await invoke<unknown>("native_card_cache_status");
    if (
      !value ||
      typeof value !== "object" ||
      !("state" in value) ||
      !states.includes(value.state as CardCacheHealth["state"]) ||
      !("last_error" in value) ||
      !(value.last_error === null || typeof value.last_error === "string")
    )
      throw new Error();
    return { state: value.state as CardCacheHealth["state"], last_error: value.last_error };
  } catch {
    throw new Error("Could not read Arena card discovery. Open Alkemyst desktop and try again.");
  }
}
export async function updateCardCache(action: "retry" | "choose"): Promise<boolean> {
  try {
    const value = await invoke<unknown>(
      action === "retry" ? "retry_card_cache" : "choose_card_cache",
    );
    if (action === "retry") return true;
    if (typeof value !== "boolean") throw new Error();
    return value;
  } catch {
    throw new Error(
      "Could not update Arena card discovery. Your imported identities are unchanged.",
    );
  }
}
