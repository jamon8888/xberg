// vitest.config.ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react-oxc";

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./tests/setup.ts"],
    include: ["tests/**/*.test.{ts,tsx}"],
    exclude: ["e2e/**", "node_modules/**", ".next/**", "out/**"],
  },
  resolve: {
    alias: {
      "@": new URL("./src", import.meta.url).pathname,
      "@xberg-io/xberg-wasm": new URL("./tests/__mocks__/xberg-wasm.ts", import.meta.url).pathname,
      "xberg-wasm-runtime": new URL("./tests/__mocks__/xberg-wasm-runtime.ts", import.meta.url).pathname,
    },
  },
});
