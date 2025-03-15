import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import unocss from "unocss/vite";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), unocss()],
  root: "src",

  build: {
    target: "esnext",
    outDir: "../dist",
    emptyOutDir: true,
  },
  optimizeDeps: {
    esbuildOptions: {
      target: "esnext",
    },
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: [
        "**/src-tauri/**",
        "**/zundamon-speech/**",
        "**/zundamon-speech-setup/**",
      ],
    },
  },
}));
