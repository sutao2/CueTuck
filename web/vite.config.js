import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  resolve: { dedupe: ['vue'] },
  server: {
    port: 5175,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
  },
});
