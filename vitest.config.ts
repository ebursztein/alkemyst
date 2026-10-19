import { defineConfig } from "vitest/config";
export default defineConfig({
  test: {
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/**/*.spec.{ts,tsx}",
        "src/**/generated/**",
        "src/**/vite.config.ts",
        "src/**/env.d.ts",
        "src/**/dist/**",
        "src/**/node_modules/**",
      ],
      reporter: ["text", "json-summary", "lcov", "html"],
      thresholds: { lines: 85, statements: 85, functions: 85, branches: 80 },
    },
  },
});
