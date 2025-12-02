import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  css: {
    postcss: "./postcss.config.js",
  },
  server: {
    port: 1420,
  },
  watch: {
    // 3. tell Vite to ignore watching `src-tauri`
    ignored: ["**/src-tauri/**"],
  },
  build: {
    outDir: "dist",
  },
});
