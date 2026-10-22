import { useRef } from "react";
/** Keep the key after a failed request; clear only after success or a changed body. */
export function useRequestKey() {
  const previous = useRef<{ body: string; key: string } | null>(null);
  return {
    forBody(body: unknown) {
      const text = JSON.stringify(body);
      if (previous.current?.body !== text)
        previous.current = { body: text, key: crypto.randomUUID() };
      return previous.current.key;
    },
    clear() {
      previous.current = null;
    },
  };
}
