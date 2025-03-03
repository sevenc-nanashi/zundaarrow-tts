import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetWind4,
  transformerDirectives,
} from "unocss";

export default defineConfig({
  presets: [
    presetAttributify({
      prefixedOnly: true,
    }),
    presetWind4(),
    presetIcons(),
  ],
  transformers: [transformerDirectives()],
  rules: [["align-content-center", { "align-content": "center" }]],
});
