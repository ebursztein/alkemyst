import { invoke } from "@tauri-apps/api/core";

export type RestoreStatus =
  | { state: "none" }
  | { state: "restored"; safety_backup: string }
  | { state: "failed"; safety_backup: string | null };

export async function readRestoreStatus(): Promise<RestoreStatus> {
  try {
    const result = await invoke<unknown>("native_restore_status");
    if (!result || typeof result !== "object" || !("state" in result)) throw new Error();
    if (result.state === "none") return { state: "none" };
    if (
      (result.state === "restored" || result.state === "failed") &&
      "safety_backup" in result &&
      (typeof result.safety_backup === "string" ||
        (result.state === "failed" && result.safety_backup === null))
    )
      return result as RestoreStatus;
    throw new Error();
  } catch {
    throw new Error("Could not read backup status. Open Alkemyst desktop and try again.");
  }
}

export type BackupAction = "save" | "select" | "cancel" | "restore";
const actions = {
  save: [
    "create_backup",
    "Could not save the backup. Choose a new file name and check available disk space.",
  ],
  select: [
    "select_restore",
    "Could not prepare this backup. Select a valid backup from the same Alkemyst version and check available disk space.",
  ],
  cancel: ["cancel_restore", "Could not cancel the selection. Try again before quitting."],
  restore: [
    "restore_and_restart",
    "Could not schedule the restore. Your current library is unchanged.",
  ],
} as const;

export async function runBackupAction(action: BackupAction): Promise<string | null> {
  const [command, message] = actions[action];
  try {
    const value = await invoke<unknown>(command);
    if (action === "cancel" || action === "restore") return null;
    if (value === null || (typeof value === "string" && value.length > 0)) return value;
    throw new Error();
  } catch {
    throw new Error(message);
  }
}
