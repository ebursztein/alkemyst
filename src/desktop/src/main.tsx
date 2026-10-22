import "./styles/editor.css";
import "./styles/theme.css";
import "./styles/navigation.css";
import "./styles/layout.css";
import "./styles/controls.css";
import "./styles/content.css";
import "./styles/practice.css";
import { createRoot } from "react-dom/client";
import { Foundation } from "./foundation.js";

export function mount(element: Element) {
  createRoot(element).render(<Foundation />);
}
const root = document.getElementById("root");
if (root) mount(root);
