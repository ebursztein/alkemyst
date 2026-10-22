import astro from "eslint-plugin-astro";
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";
export default tseslint.config(
  {
    ignores: [
      "**/generated/**",
      "**/dist/**",
      "**/node_modules/**",
      "**/.astro/**",
      "**/.wrangler/**",
      "target/**",
      "work/**",
      ".sprinty/**",
      "coverage/**",
      "playwright-report/**",
      "test-results/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...astro.configs["flat/recommended"],
  {
    languageOptions: { globals: { ...globals.node, ...globals.browser } },
    rules: { "@typescript-eslint/no-explicit-any": "error" },
  },
);
