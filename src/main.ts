import { invoke } from "@tauri-apps/api/core";
import { createApp } from "vue";
import "@fontsource/zen-maru-gothic";
import "virtual:uno.css";

import App from "./App.vue";
import { router } from "./router.ts";

createApp(App).use(router).mount("#app");
