import "@fontsource/zen-maru-gothic/400.css";
import "@unocss/reset/tailwind.css";
import "./styles/element.scss";
import "./style.css";
import "virtual:uno.css";

import { ViteSSG } from "vite-ssg/single-page";
import ElementPlus from "element-plus";
import App from "./App.vue";

export const createApp = ViteSSG(App, ({ app }) => {
  app.use(ElementPlus);
});
