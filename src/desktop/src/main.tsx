import { createRoot } from "react-dom/client";
import { Foundation } from "./foundation.js";

export function mount(element: Element) {
  createRoot(element).render(<Foundation />);
}
const root = document.getElementById("root");
if (root) mount(root);
