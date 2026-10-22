import { invoke } from "@tauri-apps/api/core";
export interface NativeSettings {
  launch_at_login: boolean;
  tracking_paused: boolean;
  tracking: { state: "waiting" | "tracking" | "degraded" | "stopped"; last_error: string | null };
}
export async function readNativeSettings(): Promise<NativeSettings> {
  try {
    const value = await invoke<unknown>("native_settings");
    if (
      !value ||
      typeof value !== "object" ||
      !("launch_at_login" in value) ||
      typeof value.launch_at_login !== "boolean" ||
      !("tracking_paused" in value) ||
      typeof value.tracking_paused !== "boolean"
    ) {
      throw new Error("Invalid native settings");
    }
    if (
      !("tracking" in value) ||
      !value.tracking ||
      typeof value.tracking !== "object" ||
      !("state" in value.tracking) ||
      !["waiting", "tracking", "degraded", "stopped"].includes(String(value.tracking.state)) ||
      !("last_error" in value.tracking) ||
      !(value.tracking.last_error === null || typeof value.tracking.last_error === "string")
    ) {
      throw new Error("Invalid tracking health");
    }
    return {
      launch_at_login: value.launch_at_login,
      tracking_paused: value.tracking_paused,
      tracking: {
        state: value.tracking.state as NativeSettings["tracking"]["state"],
        last_error: value.tracking.last_error,
      },
    };
  } catch {
    throw new Error(
      "Could not read desktop settings. Open the Alkemyst desktop app and try again.",
    );
  }
}
export async function updateNativeSetting(
  change:
    { setting: "launch_at_login"; value: boolean } | { setting: "tracking_paused"; value: boolean },
): Promise<void> {
  try {
    if (change.setting === "launch_at_login") {
      await invoke("set_launch_at_login", { enabled: change.value });
    } else {
      await invoke("set_tracking_paused", { paused: change.value });
    }
  } catch {
    throw new Error("Could not update desktop settings. Try again.");
  }
}
