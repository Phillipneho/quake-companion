import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  // Tauri expects a deterministic port in dev.
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: "127.0.0.1",
    // Proxy is not required (IPC uses the Tauri bridge), but keep HMR working
    // when launched through `tauri dev`.
    hmr: { protocol: "ws", host: "127.0.0.1", port: 5174 },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // Tauri embeds the dist folder; relative paths are required.
    target: "es2021",
    // Keep source maps off for the production bundle (smaller, the panel is
    // resource-constrained).
    sourcemap: false,
    outDir: "dist",
    emptyOutDir: true,
  },
});