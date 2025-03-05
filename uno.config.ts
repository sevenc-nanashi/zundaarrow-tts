import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetWind3,
  transformerDirectives,
} from "unocss";

export default defineConfig({
  presets: [
    presetAttributify({
      prefixedOnly: true,
    }),
    presetWind3(),
    presetIcons(),
  ],
  transformers: [transformerDirectives()],
  rules: [["align-content-center", { "align-content": "center" }]],
});
