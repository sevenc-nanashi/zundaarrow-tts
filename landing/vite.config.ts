import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import unocss from "unocss/vite";
import { imagetools } from "vite-imagetools";

// https://vite.dev/config/
export default defineConfig({
  base: "/zundaarrow-tts/",
  plugins: [
    vue({
      template: {
        compilerOptions: {
          isCustomElement: (tag) => tag === "budoux-ja",
        },
      },
    }),
    unocss({
      configFile: "../uno.config.ts",
    }),
    imagetools(),
  ],
});
