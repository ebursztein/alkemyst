export function metadataText(data: unknown, key: string): string | undefined {
  if (data && typeof data === "object" && key in data) {
    const value = (data as Record<string, unknown>)[key];
    if (typeof value === "string") return value;
  }
  return undefined;
}
export function cardImage(data: unknown): string | undefined {
  if (!data || typeof data !== "object") return undefined;
  const record = data as Record<string, unknown>;
  const image = metadataText(record.image_uris, "normal");
  if (!image) {
    const front = Array.isArray(record.card_faces) ? record.card_faces[0] : undefined;
    return front ? cardImage(front) : undefined;
  }
  try {
    const url = new URL(image);
    return url.protocol === "https:" && url.hostname === "cards.scryfall.io" ? url.href : undefined;
  } catch {
    return undefined;
  }
}
