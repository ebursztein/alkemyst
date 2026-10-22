import { invoke } from "@tauri-apps/api/core";
export interface LocalConnection {
  base_url: string;
  token: string;
}
export function validateConnection(value: unknown): LocalConnection {
  if (
    !value ||
    typeof value !== "object" ||
    !("base_url" in value) ||
    !("token" in value) ||
    typeof value.base_url !== "string" ||
    typeof value.token !== "string"
  ) {
    throw new Error("Invalid local connection");
  }
  let url: URL;
  try {
    url = new URL(value.base_url);
  } catch {
    throw new Error("Invalid local connection");
  }
  if (
    url.protocol !== "http:" ||
    url.hostname !== "127.0.0.1" ||
    !url.port ||
    url.pathname !== "/" ||
    url.search ||
    url.hash ||
    url.username ||
    url.password ||
    value.token.length < 32 ||
    value.token.length > 256 ||
    !/^[!-~]+$/.test(value.token)
  ) {
    throw new Error("Invalid local connection");
  }
  return { base_url: url.origin, token: value.token };
}
export async function bootstrapConnection(): Promise<LocalConnection> {
  try {
    return validateConnection(await invoke<unknown>("connection_bootstrap"));
  } catch {
    throw new Error("Open Alkemyst desktop to connect to your local library.");
  }
}
