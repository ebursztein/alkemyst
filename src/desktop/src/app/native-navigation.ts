import { listen } from "@tauri-apps/api/event";
export function watchSettingsNavigation(navigate: () => void): () => void {
  let closed = false;
  let unsubscribe: (() => void) | undefined;
  void listen("navigate-settings", () => {
    if (!closed) navigate();
  })
    .then((stop) => {
      if (closed) stop();
      else unsubscribe = stop;
    })
    .catch(() => {
      // Browser previews have no native event bridge; the Settings link still works.
    });
  return () => {
    closed = true;
    unsubscribe?.();
  };
}
