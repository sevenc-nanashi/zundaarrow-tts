import { createApp } from "vue";
import "@fontsource/zen-maru-gothic";
import "virtual:uno.css";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router.ts";

const pinia = createPinia();

createApp(App).use(router).use(pinia).mount("#app");
