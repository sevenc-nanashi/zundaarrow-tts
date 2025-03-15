import { createApp } from "vue";
import "@fontsource/zen-maru-gothic/index.css";
import "virtual:uno.css";
import { createPinia } from "pinia";
import ElementPlus from "element-plus";
import "@unocss/reset/tailwind.css";
import "./styles/element.scss";
import "./styles.css";
import App from "./App.vue";
import { router } from "./router.ts";

const pinia = createPinia();

createApp(App).use(ElementPlus).use(router).use(pinia).mount("#app");
