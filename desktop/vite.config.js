import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [vue()],
  resolve: { dedupe: ['vue'] },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        launcher: resolve(import.meta.dirname, "launcher.html"),
      },
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ['./src/test/setup.js'],
  },
});
